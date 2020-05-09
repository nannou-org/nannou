//! Expose a C compatible interface.

use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::io;
use std::os::raw;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Allows for detecting and enumerating laser DACs on a network and establishing new streams of
/// communication with them.
#[repr(C)]
pub struct Api {
    inner: *mut ApiInner,
}

/// A handle to a non-blocking DAC detection thread.
#[repr(C)]
pub struct DetectDacsAsync {
    inner: *mut DetectDacsAsyncInner,
}

/// Represents a DAC that has been detected on the network along with any information collected
/// about the DAC in the detection process.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DetectedDac {
    pub kind: DetectedDacKind,
}

/// A union for distinguishing between the kind of LASER DAC that was detected. Currently, only
/// EtherDream is supported, however this will gain more variants as more protocols are added (e.g.
/// AVB).
#[repr(C)]
#[derive(Clone, Copy)]
pub union DetectedDacKind {
    pub ether_dream: DacEtherDream,
}

/// An Ether Dream DAC that was detected on the network.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DacEtherDream {
    pub broadcast: ether_dream::protocol::DacBroadcast,
    pub source_addr: SocketAddr,
}

/// A set of stream configuration parameters applied to the initialisation of both `Raw` and
/// `Frame` streams.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StreamConfig {
    /// A valid pointer to a `DetectedDac` that should be targeted.
    pub detected_dac: *const DetectedDac,
    /// The rate at which the DAC should process points per second.
    ///
    /// This value should be no greater than the detected DAC's `max_point_hz`.
    pub point_hz: raw::c_uint,
    /// The maximum latency specified as a number of points.
    ///
    /// Each time the laser indicates its "fullness", the raw stream will request enough points
    /// from the render function to fill the DAC buffer up to `latency_points`.
    ///
    /// This value should be no greaterthan the DAC's `buffer_capacity`.
    pub latency_points: raw::c_uint,
    /// The timeout duration of the stream in seconds.
    ///
    /// A negative value indicates that the stream should never timeout. This is the default case.
    pub tcp_timeout_secs: raw::c_float,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum IpAddrVersion {
    V4,
    V6,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct IpAddr {
    pub version: IpAddrVersion,
    /// 4 bytes used for `V4`, 16 bytes used for `V6`.
    pub bytes: [raw::c_uchar; 16],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SocketAddr {
    pub ip: IpAddr,
    pub port: raw::c_ushort,
}

/// A handle to a stream that requests frames of LASER data from the user.
///
/// Each "frame" has an optimisation pass applied that optimises the path for inertia, minimal
/// blanking, point de-duplication and segment order.
#[repr(C)]
pub struct FrameStream {
    inner: *mut FrameStreamInner,
}

/// A handle to a raw LASER stream that requests the exact number of points that the DAC is
/// awaiting in each call to the user's callback.
#[repr(C)]
pub struct RawStream {
    inner: *mut RawStreamInner,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Result {
    Success = 0,
    DetectDacFailed,
    BuildStreamFailed,
    DetectDacsAsyncFailed,
    CloseStreamFailed,
    NullPointer,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StreamError {
    inner: *const StreamErrorInner,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum StreamErrorKind {
    EtherDreamFailedToDetectDacs,
    EtherDreamFailedToConnectStream,
    EtherDreamFailedToPrepareStream,
    EtherDreamFailedToBeginStream,
    EtherDreamFailedToSubmitData,
    EtherDreamFailedToSubmitPointRate,
    EtherDreamFailedToStopStream,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StreamErrorAction {
    inner: *mut StreamErrorActionInner,
}

/// An owned instance of a raw C string.
#[repr(C)]
pub struct RawString {
    inner: *mut raw::c_char,
}

/// A set of stream configuration parameters unique to `Frame` streams.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct FrameStreamConfig {
    pub stream_conf: StreamConfig,
    /// The rate at which the stream will attempt to present images via the DAC. This value is used
    /// in combination with the DAC's `point_hz` in order to determine how many points should be
    /// used to draw each frame. E.g.
    ///
    /// ```ignore
    /// let points_per_frame = point_hz / frame_hz;
    /// ```
    ///
    /// This is simply used as a minimum value. E.g. if some very simple geometry is submitted, this
    /// allows the DAC to spend more time creating the path for the image. However, if complex geometry
    /// is submitted that would require more than the ideal `points_per_frame`, the DAC may not be able
    /// to achieve the desired `frame_hz` when drawing the path while also taking the
    /// `distance_per_point` and `radians_per_point` into consideration.
    pub frame_hz: u32,
    /// Enable or disable frame optimisations.
    ///
    /// By default, optimisations are enabled. This includes path re-interpolation, as
    /// re-interpolation is only possible using a euler circuit which is created during the
    /// optimisation pass.
    ///
    /// Read more about the kinds of optimisations applied at the
    /// [**lasy**](https://docs.rs/lasy/0.3.0/lasy/) API documentation.
    ///
    /// Returns `true` on success or `false` if the communication channel was closed.
    pub enable_optimisations: bool,
    /// Configuration options for eulerian circuit interpolation.
    pub interpolation_conf: lasy::InterpolationConfig,
}

#[repr(C)]
pub struct Frame {
    inner: *mut FrameInner,
}

#[repr(C)]
pub struct Buffer {
    inner: *mut BufferInner,
}

struct FrameInner(*mut crate::stream::frame::Frame);

struct BufferInner(*mut crate::stream::raw::Buffer);

struct ApiInner {
    inner: crate::Api,
    last_error: Option<CString>,
}

struct DetectDacsAsyncInner {
    _inner: crate::DetectDacsAsync,
    dacs: Arc<Mutex<HashMap<crate::DacId, (Instant, crate::DetectedDac)>>>,
    last_error: Arc<Mutex<Option<CString>>>,
}

struct StreamErrorInner(*const crate::StreamError);

struct StreamErrorActionInner(*mut crate::StreamErrorAction);

struct FrameStreamModel(
    *mut raw::c_void,
    FrameRenderCallback,
    RawRenderCallback,
    StreamErrorCallback,
);
struct RawStreamModel(*mut raw::c_void, RawRenderCallback, StreamErrorCallback);

unsafe impl Send for FrameStreamModel {}
unsafe impl Send for RawStreamModel {}

struct FrameStreamInner(crate::FrameStream<FrameStreamModel>);
struct RawStreamInner(crate::RawStream<RawStreamModel>);

pub type FrameRenderCallback = extern "C" fn(*mut raw::c_void, *mut Frame);
pub type RawRenderCallback = extern "C" fn(*mut raw::c_void, *mut Buffer);
pub type StreamErrorCallback =
    extern "C" fn(*mut raw::c_void, *const StreamError, *mut StreamErrorAction);

/// Given some uninitialized pointer to an `Api` struct, fill it with a new Api instance.
#[no_mangle]
pub unsafe extern "C" fn api_new(api: *mut Api) {
    let inner = crate::Api::new();
    let last_error = None;
    let boxed_inner = Box::new(ApiInner { inner, last_error });
    (*api).inner = Box::into_raw(boxed_inner);
}

/// Given some uninitialised pointer to a `DetectDacsAsync` struct, fill it with a new instance.
///
/// If the given `timeout_secs` is `0`, DAC detection will never timeout and detected DACs that no
/// longer broadcast will remain accessible in the device map.
#[no_mangle]
pub unsafe extern "C" fn detect_dacs_async(
    api: *mut Api,
    timeout_secs: raw::c_float,
    detect_dacs: *mut DetectDacsAsync,
) -> Result {
    let api: &mut ApiInner = &mut (*(*api).inner);
    let duration = if timeout_secs == 0.0 {
        None
    } else {
        Some(duration_from_secs_f32(timeout_secs))
    };
    let boxed_detect_dacs_inner = match detect_dacs_async_inner(&api.inner, duration) {
        Ok(detector) => Box::new(detector),
        Err(err) => {
            api.last_error = Some(err_to_cstring(&err));
            return Result::DetectDacsAsyncFailed;
        }
    };
    (*detect_dacs).inner = Box::into_raw(boxed_detect_dacs_inner);
    Result::Success
}

/// Retrieve a list of the currently available DACs.
///
/// Calling this function should never block, and simply provide the list of DACs that have
/// broadcast their availability within the last specified DAC timeout duration.
#[no_mangle]
pub unsafe extern "C" fn available_dacs(
    detect_dacs_async: *mut DetectDacsAsync,
    first_dac: *mut *mut DetectedDac,
    len: *mut raw::c_uint,
) {
    let detect_dacs_async: &mut DetectDacsAsyncInner = &mut (*(*detect_dacs_async).inner);
    *first_dac = std::ptr::null_mut();
    *len = 0;
    if let Ok(dacs) = detect_dacs_async.dacs.lock() {
        if !dacs.is_empty() {
            let mut dacs: Box<[_]> = dacs
                .values()
                .map(|&(_, ref dac)| detected_dac_to_ffi(dac.clone()))
                .collect();
            *len = dacs.len() as _;
            *first_dac = dacs.as_mut_ptr();
            std::mem::forget(dacs);
        }
    }
}

/// Block the current thread until a new DAC is detected and return it.
#[no_mangle]
pub unsafe extern "C" fn detect_dac(api: *mut Api, detected_dac: *mut DetectedDac) -> Result {
    let api: &mut ApiInner = &mut (*(*api).inner);
    let mut iter = match api.inner.detect_dacs() {
        Err(err) => {
            api.last_error = Some(err_to_cstring(&err));
            return Result::DetectDacFailed;
        }
        Ok(iter) => iter,
    };
    match iter.next() {
        None => return Result::DetectDacFailed,
        Some(res) => match res {
            Ok(dac) => {
                *detected_dac = detected_dac_to_ffi(dac);
                return Result::Success;
            }
            Err(err) => {
                api.last_error = Some(err_to_cstring(&err));
                return Result::DetectDacFailed;
            }
        },
    }
}

/// Initialise the given frame stream configuration with default values.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_config_default(conf: *mut FrameStreamConfig) {
    let stream_conf = default_stream_config();
    let frame_hz = crate::stream::DEFAULT_FRAME_HZ;
    let interpolation_conf = lasy::InterpolationConfig::default();
    let enable_optimisations = crate::stream::DEFAULT_ENABLE_OPTIMISATIONS;
    *conf = FrameStreamConfig {
        stream_conf,
        frame_hz,
        interpolation_conf,
        enable_optimisations,
    };
}

/// Initialise the given raw stream configuration with default values.
#[no_mangle]
pub unsafe extern "C" fn stream_config_default(conf: *mut StreamConfig) {
    *conf = default_stream_config();
}

/// Spawn a new frame rendering stream.
///
/// The `frame_render_callback` is called each time the stream is ready for a new `Frame` of laser
/// points. Each "frame" has an optimisation pass applied that optimises the path for inertia,
/// minimal blanking, point de-duplication and segment order.
///
/// The `process_raw_callback` allows for optionally processing the raw points before submission to
/// the DAC. This might be useful for:
///
/// - applying post-processing effects onto the optimised, interpolated points.
/// - monitoring the raw points resulting from the optimisation and interpolation processes.
/// - tuning brightness of colours based on safety zones.
///
/// The given function will get called right before submission of the optimised, interpolated
/// buffer.
#[no_mangle]
pub unsafe extern "C" fn new_frame_stream(
    api: *mut Api,
    stream: *mut FrameStream,
    config: *const FrameStreamConfig,
    callback_data: *mut raw::c_void,
    frame_render_callback: FrameRenderCallback,
    process_raw_callback: RawRenderCallback,
    stream_error_callback: StreamErrorCallback,
) -> Result {
    let api: &mut ApiInner = &mut (*(*api).inner);
    let model = FrameStreamModel(
        callback_data,
        frame_render_callback,
        process_raw_callback,
        stream_error_callback,
    );

    fn render_fn(model: &mut FrameStreamModel, frame: &mut crate::stream::frame::Frame) {
        let FrameStreamModel(callback_data_ptr, frame_render_callback, _, _) = *model;
        let mut inner = FrameInner(frame);
        let mut frame = Frame { inner: &mut inner };
        frame_render_callback(callback_data_ptr, &mut frame);
    }

    fn process_raw_fn(model: &mut FrameStreamModel, buffer: &mut crate::stream::raw::Buffer) {
        let FrameStreamModel(callback_data_ptr, _, process_raw_callback, _) = *model;
        let mut inner = BufferInner(buffer);
        let mut buffer = Buffer { inner: &mut inner };
        process_raw_callback(callback_data_ptr, &mut buffer);
    }

    fn stream_error_fn(
        model: &mut FrameStreamModel,
        err: &crate::stream::raw::StreamError,
        action: &mut crate::stream::raw::StreamErrorAction,
    ) {
        let FrameStreamModel(callback_data_ptr, _, _, stream_error_callback) = *model;
        let mut inner = StreamErrorActionInner(action);
        let mut action = StreamErrorAction {
            inner: &mut inner as *mut _,
        };
        let inner = StreamErrorInner(err);
        let err = StreamError {
            inner: &inner as *const _,
        };
        stream_error_callback(callback_data_ptr, &err, &mut action);
    }

    let tcp_timeout = tcp_timeout_from_float((*config).stream_conf.tcp_timeout_secs);

    let mut builder = api
        .inner
        .new_frame_stream(model, render_fn)
        .point_hz((*config).stream_conf.point_hz as _)
        .latency_points((*config).stream_conf.latency_points as _)
        .tcp_timeout(tcp_timeout)
        .frame_hz((*config).frame_hz as _)
        .enable_optimisations((*config).enable_optimisations as _)
        .process_raw(process_raw_fn)
        .stream_error(stream_error_fn);

    if (*config).stream_conf.detected_dac != std::ptr::null() {
        let ffi_dac = (*(*config).stream_conf.detected_dac).clone();
        let detected_dac = detected_dac_from_ffi(ffi_dac);
        builder = builder.detected_dac(detected_dac);
    }

    let inner = match builder.build() {
        Err(err) => {
            api.last_error = Some(err_to_cstring(&err));
            return Result::BuildStreamFailed;
        }
        Ok(stream) => Box::new(FrameStreamInner(stream)),
    };
    (*stream).inner = Box::into_raw(inner);
    Result::Success
}

/// Spawn a new frame rendering stream.
///
/// A raw LASER stream requests the exact number of points that the DAC is awaiting in each call to
/// the user's `process_raw_callback`. Keep in mind that no optimisation passes are applied. When
/// using a raw stream, this is the responsibility of the user.
#[no_mangle]
pub unsafe extern "C" fn new_raw_stream(
    api: *mut Api,
    stream: *mut RawStream,
    config: *const StreamConfig,
    callback_data: *mut raw::c_void,
    process_raw_callback: RawRenderCallback,
    stream_error_callback: StreamErrorCallback,
) -> Result {
    let api: &mut ApiInner = &mut (*(*api).inner);
    let model = RawStreamModel(callback_data, process_raw_callback, stream_error_callback);

    fn render_fn(model: &mut RawStreamModel, buffer: &mut crate::stream::raw::Buffer) {
        let RawStreamModel(callback_data_ptr, raw_render_callback, _) = *model;
        let mut inner = BufferInner(buffer);
        let mut buffer = Buffer { inner: &mut inner };
        raw_render_callback(callback_data_ptr, &mut buffer);
    }

    fn stream_error_fn(
        model: &mut RawStreamModel,
        err: &crate::stream::raw::StreamError,
        action: &mut crate::stream::raw::StreamErrorAction,
    ) {
        let RawStreamModel(callback_data_ptr, _, stream_error_callback) = *model;
        let mut inner = StreamErrorActionInner(action);
        let mut action = StreamErrorAction {
            inner: &mut inner as *mut _,
        };
        let inner = StreamErrorInner(err);
        let err = StreamError {
            inner: &inner as *const _,
        };
        stream_error_callback(callback_data_ptr, &err, &mut action);
    }

    let tcp_timeout = tcp_timeout_from_float((*config).tcp_timeout_secs);

    let mut builder = api
        .inner
        .new_raw_stream(model, render_fn)
        .point_hz((*config).point_hz as _)
        .latency_points((*config).latency_points as _)
        .tcp_timeout(tcp_timeout)
        .stream_error(stream_error_fn);

    if (*config).detected_dac != std::ptr::null() {
        let ffi_dac = (*(*config).detected_dac).clone();
        let detected_dac = detected_dac_from_ffi(ffi_dac);
        builder = builder.detected_dac(detected_dac);
    }

    let inner = match builder.build() {
        Err(err) => {
            api.last_error = Some(err_to_cstring(&err));
            return Result::BuildStreamFailed;
        }
        Ok(stream) => Box::new(RawStreamInner(stream)),
    };
    (*stream).inner = Box::into_raw(inner);

    Result::Success
}

/// Enable or disable frame optimisations.
///
/// By default, optimisations are enabled. This includes path re-interpolation, as re-interpolation
/// is only possible using a euler circuit which is created during the optimisation pass.
///
/// Read more about the kinds of optimisations applied at the
/// [**lasy**](https://docs.rs/lasy/0.3.0/lasy/) API documentation.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_enable_optimisations(
    stream: *const FrameStream,
    enable: bool,
) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner).0.enable_optimisations(enable).is_ok()
}

/// Update the rate at which the DAC should process points per second.
///
/// This value should be no greater than the detected DAC's `max_point_hz`.
///
/// By default this value is `stream::DEFAULT_POINT_HZ`.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_set_point_hz(
    stream: *const FrameStream,
    point_hz: u32,
) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner).0.set_point_hz(point_hz).is_ok()
}

/// The maximum latency specified as a number of points.
///
/// Each time the laser indicates its "fullness", the raw stream will request enough points
/// from the render function to fill the DAC buffer up to `latency_points`.
///
/// This value should be no greaterthan the DAC's `buffer_capacity`.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_set_latency_points(
    stream: *const FrameStream,
    points: u32,
) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner).0.set_latency_points(points).is_ok()
}

/// Update the `distance_per_point` field of the interpolation configuration used within the
/// optimisation pass for frames. This represents the minimum distance the interpolator can travel
/// along an edge before a new point is required.
///
/// The value will be updated on the laser thread prior to requesting the next frame.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_set_distance_per_point(
    stream: *const FrameStream,
    distance_per_point: f32,
) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner)
        .0
        .set_distance_per_point(distance_per_point)
        .is_ok()
}

/// Update the `blank_delay_points` field of the interpolation configuration. This represents the
/// number of points to insert at the end of a blank to account for light modulator delay.
///
/// The value will be updated on the laser thread prior to requesting the next frame.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_set_blank_delay_points(
    stream: *const FrameStream,
    points: u32,
) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner).0.set_blank_delay_points(points).is_ok()
}

/// Update the `radians_per_point` field of the interpolation configuration. This represents the
/// amount of delay to add based on the angle of the corner in radians.
///
/// The value will be updated on the laser thread prior to requesting the next frame.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_set_radians_per_point(
    stream: *const FrameStream,
    radians_per_point: f32,
) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner)
        .0
        .set_radians_per_point(radians_per_point)
        .is_ok()
}

/// Update the rate at which the stream will attempt to present images via the DAC. This value is
/// used in combination with the DAC's `point_hz` in order to determine how many points should be
/// used to draw each frame. E.g.
///
/// ```ignore
/// let points_per_frame = point_hz / frame_hz;
/// ```
///
/// This is simply used as a minimum value. E.g. if some very simple geometry is submitted, this
/// allows the DAC to spend more time creating the path for the image. However, if complex geometry
/// is submitted that would require more than the ideal `points_per_frame`, the DAC may not be able
/// to achieve the desired `frame_hz` when drawing the path while also taking the
/// `distance_per_point` and `radians_per_point` into consideration.
///
/// The value will be updated on the laser thread prior to requesting the next frame.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_set_frame_hz(
    stream: *const FrameStream,
    frame_hz: u32,
) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner).0.set_frame_hz(frame_hz).is_ok()
}

/// Returns whether or not the communication thread has closed.
///
/// A stream may be closed if an error has occurred and the stream error callback indicated to
/// close the thread. A stream might also be closed if another `close` was called on another handle
/// to the stream.
///
/// In this case, the `Stream` should be closed or dropped and a new one should be created to
/// replace it.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_is_closed(stream: *const FrameStream) -> bool {
    let stream: &FrameStream = &*stream;
    (*stream.inner).0.is_closed()
}

/// Close the TCP communication thread and wait for the thread to join.
///
/// This consumes and drops the `Stream`, returning the result produced by joining the thread.
///
/// This method will block until the associated thread has been joined.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_close(api: *mut Api, stream: FrameStream) -> Result {
    if stream.inner != std::ptr::null_mut() {
        match Box::from_raw(stream.inner).0.close() {
            Some(Ok(Ok(()))) => Result::Success,
            Some(Ok(Err(err))) => {
                (*(*api).inner).last_error = Some(err_to_cstring(&err));
                return Result::CloseStreamFailed;
            }
            Some(Err(_err)) => {
                let string = format!("failed to join stream thread");
                (*(*api).inner).last_error = Some(string_to_cstring(string));
                return Result::CloseStreamFailed;
            }
            None => Result::Success,
        }
    } else {
        Result::NullPointer
    }
}

/// Update the rate at which the DAC should process points per second.
///
/// This value should be no greater than the detected DAC's `max_point_hz`.
///
/// By default this value is `stream::DEFAULT_POINT_HZ`.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn raw_stream_set_point_hz(stream: *const RawStream, point_hz: u32) -> bool {
    let stream: &RawStream = &*stream;
    (*stream.inner).0.set_point_hz(point_hz).is_ok()
}

/// The maximum latency specified as a number of points.
///
/// Each time the laser indicates its "fullness", the raw stream will request enough points
/// from the render function to fill the DAC buffer up to `latency_points`.
///
/// This value should be no greaterthan the DAC's `buffer_capacity`.
///
/// Returns `true` on success or `false` if the communication channel was closed.
#[no_mangle]
pub unsafe extern "C" fn raw_stream_set_latency_points(
    stream: *const RawStream,
    points: u32,
) -> bool {
    let stream: &RawStream = &*stream;
    (*stream.inner).0.set_latency_points(points).is_ok()
}

/// Returns whether or not the communication thread has closed.
///
/// A stream may be closed if an error has occurred and the stream error callback indicated to
/// close the thread. A stream might also be closed if another `close` was called on another handle
/// to the stream.
///
/// In this case, the `Stream` should be closed or dropped and a new one should be created to
/// replace it.
#[no_mangle]
pub unsafe extern "C" fn raw_stream_is_closed(stream: *const RawStream) -> bool {
    let stream: &RawStream = &*stream;
    (*stream.inner).0.is_closed()
}

/// Close the TCP communication thread and wait for the thread to join.
///
/// This consumes and drops the `Stream`, returning the result produced by joining the thread.
///
/// This method will block until the associated thread has been joined.
#[no_mangle]
pub unsafe extern "C" fn raw_stream_close(api: *mut Api, stream: RawStream) -> Result {
    if stream.inner != std::ptr::null_mut() {
        match Box::from_raw(stream.inner).0.close() {
            Some(Ok(Ok(()))) => Result::Success,
            Some(Ok(Err(err))) => {
                (*(*api).inner).last_error = Some(err_to_cstring(&err));
                return Result::CloseStreamFailed;
            }
            Some(Err(_err)) => {
                let string = format!("failed to join stream thread");
                (*(*api).inner).last_error = Some(string_to_cstring(string));
                return Result::CloseStreamFailed;
            }
            None => Result::Success,
        }
    } else {
        Result::NullPointer
    }
}

/// Add a sequence of consecutive points separated by blank space.
///
/// If some points already exist in the frame, this method will create a blank segment between the
/// previous point and the first point before appending this sequence.
#[no_mangle]
pub unsafe extern "C" fn frame_add_points(
    frame: *mut Frame,
    points: *const crate::Point,
    len: usize,
) {
    let frame = &mut *frame;
    let points = std::slice::from_raw_parts(points, len);
    (*(*frame.inner).0).add_points(points.iter().cloned());
}

/// Add a sequence of consecutive lines.
///
/// If some points already exist in the frame, this method will create a blank segment between the
/// previous point and the first point before appending this sequence.
#[no_mangle]
pub unsafe extern "C" fn frame_add_lines(
    frame: *mut Frame,
    points: *const crate::Point,
    len: usize,
) {
    let frame = &mut *frame;
    let points = std::slice::from_raw_parts(points, len);
    (*(*frame.inner).0).add_lines(points.iter().cloned());
}

/// Retrieve the current `frame_hz` at the time of rendering this `Frame`.
#[no_mangle]
pub unsafe extern "C" fn frame_hz(frame: *const Frame) -> u32 {
    (*(*(*frame).inner).0).frame_hz()
}

/// Retrieve the current `point_hz` at the time of rendering this `Frame`.
#[no_mangle]
pub unsafe extern "C" fn frame_point_hz(frame: *const Frame) -> u32 {
    (*(*(*frame).inner).0).point_hz()
}

/// Retrieve the current `latency_points` at the time of rendering this `Frame`.
#[no_mangle]
pub unsafe extern "C" fn frame_latency_points(frame: *const Frame) -> u32 {
    (*(*(*frame).inner).0).latency_points()
}

/// Retrieve the current ideal `points_per_frame` at the time of rendering this `Frame`.
#[no_mangle]
pub unsafe extern "C" fn points_per_frame(frame: *const Frame) -> u32 {
    (*(*(*frame).inner).0).latency_points()
}

/// Allocate a new C string containing the error message.
#[no_mangle]
pub unsafe extern "C" fn stream_error_message(err: *const StreamError) -> RawString {
    let string = format!("{}", *(*(*err).inner).0);
    raw_string_from_str(&string[..])
}

/// Returns the pointer to the beginning of the C string for reading.
#[no_mangle]
pub unsafe extern "C" fn raw_string_ref(msg: *const RawString) -> *const raw::c_char {
    (*msg).inner as *const raw::c_char
}

/// Must be called in order to correctly clean up a raw string.
#[no_mangle]
pub unsafe extern "C" fn raw_string_drop(msg: RawString) {
    if msg.inner != std::ptr::null_mut() {
        CString::from_raw(msg.inner);
    }
}

/// Must be called in order to correctly clean up the frame stream.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_drop(stream: FrameStream) {
    if stream.inner != std::ptr::null_mut() {
        Box::from_raw(stream.inner);
    }
}

/// Must be called in order to correctly clean up the raw stream.
#[no_mangle]
pub unsafe extern "C" fn raw_stream_drop(stream: RawStream) {
    if stream.inner != std::ptr::null_mut() {
        Box::from_raw(stream.inner);
    }
}

/// Must be called in order to correctly clean up the `DetectDacsAsync` resources.
#[no_mangle]
pub unsafe extern "C" fn detect_dacs_async_drop(detect: DetectDacsAsync) {
    if detect.inner != std::ptr::null_mut() {
        Box::from_raw(detect.inner);
    }
}

/// Must be called in order to correctly clean up the API resources.
#[no_mangle]
pub unsafe extern "C" fn api_drop(api: Api) {
    if api.inner != std::ptr::null_mut() {
        Box::from_raw(api.inner);
    }
}

/// Used for retrieving the last error that occurred from the API.
#[no_mangle]
pub unsafe extern "C" fn api_last_error(api: *const Api) -> *const raw::c_char {
    let api: &ApiInner = &(*(*api).inner);
    match api.last_error {
        None => std::ptr::null(),
        Some(ref cstring) => cstring.as_ptr(),
    }
}

/// Used for retrieving the last error that occurred from the API.
#[no_mangle]
pub unsafe extern "C" fn detect_dacs_async_last_error(
    detect: *const DetectDacsAsync,
) -> *const raw::c_char {
    let detect_dacs_async: &DetectDacsAsyncInner = &(*(*detect).inner);
    let mut s = std::ptr::null();
    if let Ok(last_error) = detect_dacs_async.last_error.lock() {
        if let Some(ref cstring) = *last_error {
            s = cstring.as_ptr();
        }
    }
    s
}

/// Retrieve the kind of the stream error.
#[no_mangle]
pub unsafe extern "C" fn stream_error_kind(err: *const StreamError) -> StreamErrorKind {
    let err: &crate::StreamError = &*(*(*err).inner).0;
    stream_error_to_kind(err)
}

/// Retrieve the number of attempts from the stream error.
///
/// If the error is `EtherDreamFailedToConnectStream`, this refers to the consecutive number of
/// failed attempts to establish a TCP connection with the DAC.
///
/// If the error is `EtherDreamFailedToDetectDac`, this refers to the consecutive number of failed
/// attempts to detect the requested DAC.
#[no_mangle]
pub unsafe extern "C" fn stream_error_attempts(err: *const StreamError) -> u32 {
    let err: &crate::StreamError = &*(*(*err).inner).0;
    stream_error_to_attempts(err)
}

/// Set the error action to reattempt the TCP stream connection.
///
/// This action attempts to reconnect to the specified DAC in the case that one was provided, or
/// any DAC in the case that `None` was provided.
#[no_mangle]
pub unsafe extern "C" fn stream_error_action_set_reattempt_connect(action: *mut StreamErrorAction) {
    let target = crate::StreamErrorAction::ReattemptConnect;
    stream_error_action_set(action, target);
}

/// Set the error action to redetect the DAC.
///
/// This action attempts to re-detect the same DAC in the case that one was specified, or any DAC in the
/// case that `None` was provided.
///
/// This can be useful in the case where the DAC has dropped from the network and may have
/// re-appeared broadcasting from a different IP address.
#[no_mangle]
pub unsafe extern "C" fn stream_error_action_set_redetect_dacs(
    action: *mut StreamErrorAction,
    timeout_secs: f32,
) {
    let timeout = if timeout_secs < 0.0 {
        None
    } else {
        Some(duration_from_secs_f32(timeout_secs))
    };
    let target = crate::StreamErrorAction::RedetectDac { timeout };
    stream_error_action_set(action, target);
}

/// Set the error action to close the TCP communication thread.
#[no_mangle]
pub unsafe extern "C" fn stream_error_action_set_close_thread(action: *mut StreamErrorAction) {
    let target = crate::StreamErrorAction::CloseThread;
    stream_error_action_set(action, target);
}

/// Begin asynchronous DAC detection.
///
/// The given timeout corresponds to the duration of time since the last DAC broadcast was received
/// before a DAC will be considered to be unavailable again.
fn detect_dacs_async_inner(
    api: &crate::Api,
    dac_timeout: Option<Duration>,
) -> io::Result<DetectDacsAsyncInner> {
    let dacs = Arc::new(Mutex::new(HashMap::new()));
    let last_error = Arc::new(Mutex::new(None));
    let dacs2 = dacs.clone();
    let last_error2 = last_error.clone();
    let _inner = api.detect_dacs_async(
        dac_timeout,
        move |res: io::Result<crate::DetectedDac>| match res {
            Ok(dac) => {
                let now = Instant::now();
                let mut dacs = dacs2.lock().unwrap();
                dacs.insert(dac.id(), (now, dac));
                if let Some(timeout) = dac_timeout {
                    dacs.retain(|_, (ref last_bc, _)| now.duration_since(*last_bc) < timeout);
                }
            }
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {
                    dacs2.lock().unwrap().clear();
                }
                _ => {
                    *last_error2.lock().unwrap() = Some(err_to_cstring(&err));
                }
            },
        },
    )?;
    Ok(DetectDacsAsyncInner {
        _inner,
        dacs,
        last_error,
    })
}

fn tcp_timeout_from_float(tcp_timeout_secs: raw::c_float) -> Option<Duration> {
    // A negative value is considered to mean no timeout.
    if tcp_timeout_secs < 0.0 {
        None
    } else {
        let duration = duration_from_secs_f32(tcp_timeout_secs);
        Some(duration)
    }
}

unsafe fn stream_error_action_set(
    action: *mut StreamErrorAction,
    target: crate::StreamErrorAction,
) {
    let action: &mut crate::StreamErrorAction = &mut *(*(*action).inner).0;
    *action = target;
}

fn raw_string_from_str(s: &str) -> RawString {
    let cstring = CString::new(&s[..]).unwrap();
    let inner = cstring.into_raw();
    RawString { inner }
}

fn duration_from_secs_f32(secs: f32) -> Duration {
    let whole_secs = secs as u64;
    let nanos = ((secs - whole_secs as raw::c_float) * 1_000_000_000.0) as u32;
    std::time::Duration::new(whole_secs, nanos)
}

fn err_to_cstring(err: &dyn fmt::Display) -> CString {
    string_to_cstring(format!("{}", err))
}

fn string_to_cstring(string: String) -> CString {
    CString::new(string.into_bytes()).expect("`string` contained null bytes")
}

fn default_stream_config() -> StreamConfig {
    let detected_dac = std::ptr::null();
    let point_hz = crate::stream::DEFAULT_POINT_HZ;
    let latency_points = crate::stream::raw::default_latency_points(point_hz);
    let tcp_timeout_secs = -1.0;
    StreamConfig {
        detected_dac,
        point_hz,
        latency_points,
        tcp_timeout_secs,
    }
}

fn socket_addr_to_ffi(addr: std::net::SocketAddr) -> SocketAddr {
    let port = addr.port();
    let mut bytes = [0u8; 16];
    let ip = match addr.ip() {
        std::net::IpAddr::V4(ref ip) => {
            for (byte, octet) in bytes.iter_mut().zip(&ip.octets()) {
                *byte = *octet;
            }
            let version = IpAddrVersion::V4;
            IpAddr { version, bytes }
        }
        std::net::IpAddr::V6(ref ip) => {
            for (byte, octet) in bytes.iter_mut().zip(&ip.octets()) {
                *byte = *octet;
            }
            let version = IpAddrVersion::V6;
            IpAddr { version, bytes }
        }
    };
    SocketAddr { ip, port }
}

fn socket_addr_from_ffi(addr: SocketAddr) -> std::net::SocketAddr {
    let ip = match addr.ip.version {
        IpAddrVersion::V4 => {
            let b = &addr.ip.bytes;
            std::net::IpAddr::from([b[0], b[1], b[2], b[3]])
        }
        IpAddrVersion::V6 => std::net::IpAddr::from(addr.ip.bytes),
    };
    std::net::SocketAddr::new(ip, addr.port)
}

fn detected_dac_to_ffi(dac: crate::DetectedDac) -> DetectedDac {
    match dac {
        crate::DetectedDac::EtherDream {
            broadcast,
            source_addr,
        } => {
            let source_addr = socket_addr_to_ffi(source_addr);
            let ether_dream = DacEtherDream {
                broadcast,
                source_addr,
            };
            let kind = DetectedDacKind { ether_dream };
            DetectedDac { kind }
        }
    }
}

fn detected_dac_from_ffi(ffi_dac: DetectedDac) -> crate::DetectedDac {
    unsafe {
        let broadcast = ffi_dac.kind.ether_dream.broadcast.clone();
        let source_addr = socket_addr_from_ffi(ffi_dac.kind.ether_dream.source_addr);
        crate::DetectedDac::EtherDream {
            broadcast,
            source_addr,
        }
    }
}

fn stream_error_to_kind(err: &crate::StreamError) -> StreamErrorKind {
    use crate::stream::raw::EtherDreamStreamError;
    match *err {
        crate::StreamError::EtherDreamStream { ref err } => match *err {
            EtherDreamStreamError::FailedToDetectDacs { .. } => {
                StreamErrorKind::EtherDreamFailedToDetectDacs
            }
            EtherDreamStreamError::FailedToConnectStream { .. } => {
                StreamErrorKind::EtherDreamFailedToConnectStream
            }
            EtherDreamStreamError::FailedToPrepareStream { .. } => {
                StreamErrorKind::EtherDreamFailedToPrepareStream
            }
            EtherDreamStreamError::FailedToBeginStream { .. } => {
                StreamErrorKind::EtherDreamFailedToBeginStream
            }
            EtherDreamStreamError::FailedToSubmitData { .. } => {
                StreamErrorKind::EtherDreamFailedToSubmitData
            }
            EtherDreamStreamError::FailedToSubmitPointRate { .. } => {
                StreamErrorKind::EtherDreamFailedToSubmitPointRate
            }
            EtherDreamStreamError::FailedToStopStream { .. } => {
                StreamErrorKind::EtherDreamFailedToStopStream
            }
        },
    }
}

fn stream_error_to_attempts(err: &crate::StreamError) -> u32 {
    use crate::stream::raw::EtherDreamStreamError;
    match *err {
        crate::StreamError::EtherDreamStream { ref err } => match *err {
            EtherDreamStreamError::FailedToDetectDacs { attempts, .. }
            | EtherDreamStreamError::FailedToConnectStream { attempts, .. } => attempts,
            _ => 0,
        },
    }
}
