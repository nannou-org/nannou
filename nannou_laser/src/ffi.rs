//! Expose a C compatible interface.

use std::ffi::{CStr, CString};
use std::os::raw;

#[repr(C)]
pub struct Api {
    inner: crate::Api,
    last_error: Option<CString>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DetectedDac {
    pub kind: DetectedDacKind,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union DetectedDacKind {
    pub ether_dream: DacEtherDream,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DacEtherDream {
    pub broadcast: ether_dream::protocol::DacBroadcast,
    pub source_addr: *const raw::c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StreamConfig {
    pub detected_dac: *const DetectedDac,
    pub point_hz: raw::c_uint,
    pub latency_points: raw::c_uint,
}

#[repr(C)]
pub struct FrameStream {
    stream: FrameStreamInner,
}

#[repr(C)]
pub struct RawStream {
    stream: RawStreamInner,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Result {
    Success = 0,
    FailedToDetectDac,
    FailedToBuildStream,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct FrameStreamConfig {
    pub stream_conf: StreamConfig,
    pub frame_hz: u32,
    pub interpolation_conf: crate::stream::frame::opt::InterpolationConfig,
}

#[repr(C)]
pub struct Frame<'a> {
    frame: &'a mut crate::stream::frame::Frame,
}

#[repr(C)]
pub struct Buffer<'a> {
    buffer: &'a mut crate::stream::raw::Buffer,
}

struct FrameStreamModel(*mut raw::c_void, FrameRenderCallback, RawRenderCallback);
struct RawStreamModel(*mut raw::c_void, RawRenderCallback);

unsafe impl Send for FrameStreamModel {}
unsafe impl Send for RawStreamModel {}

type FrameStreamInner = crate::FrameStream<FrameStreamModel>;
type RawStreamInner = crate::RawStream<RawStreamModel>;

/// Cast to `extern fn(*mut raw::c_void, *mut Frame)` internally.
//pub type FrameRenderCallback = *const raw::c_void;
pub type FrameRenderCallback = extern "C" fn(*mut raw::c_void, *mut Frame);
/// Cast to `extern fn(*mut raw::c_void, *mut Buffer)` internally.
//pub type RawRenderCallback = *const raw::c_void;
pub type RawRenderCallback = extern "C" fn(*mut raw::c_void, *mut Buffer);

/// Given some uninitialized pointer to an `Api` struct, fill it with a new Api instance.
#[no_mangle]
pub unsafe extern "C" fn api_new(api_ptr: *mut *mut Api) {
    let inner = crate::Api::new();
    let last_error = None;
    let api = Api { inner, last_error };
    let boxed_api = Box::new(api);
    let new_api_ptr = Box::into_raw(boxed_api);
    *api_ptr = new_api_ptr;
}

/// Block the current thread until a new DAC is detected and return it.
#[no_mangle]
pub unsafe extern "C" fn detect_dac(api: *const Api, detected_dac: *mut DetectedDac) -> Result {
    let api: &Api = &*api;
    let mut iter = match api.inner.detect_dacs() {
        Err(_err) => {
            // TODO: Store error
            return Result::FailedToDetectDac;
        }
        Ok(iter) => iter,
    };
    match iter.next() {
        None => return Result::FailedToDetectDac,
        Some(res) => match res {
            Ok(crate::DetectedDac::EtherDream {
                broadcast,
                source_addr,
            }) => {
                let string = format!("{}", source_addr);
                let bytes = string.into_bytes();
                let source_addr = bytes.as_ptr() as *const raw::c_char;
                std::mem::forget(bytes);
                let ether_dream = DacEtherDream {
                    broadcast,
                    source_addr,
                };
                let kind = DetectedDacKind { ether_dream };
                *detected_dac = DetectedDac { kind };
                return Result::Success;
            }
            Err(_err) => {
                // TODO: Store error
                return Result::FailedToDetectDac;
            }
        },
    }
}

fn default_stream_config() -> StreamConfig {
    let detected_dac = std::ptr::null();
    let point_hz = crate::stream::DEFAULT_POINT_HZ;
    let latency_points = crate::stream::raw::default_latency_points(point_hz);
    StreamConfig {
        detected_dac,
        point_hz,
        latency_points,
    }
}

/// Initialise the given frame stream configuration with default values.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_config_default(conf: *mut FrameStreamConfig) {
    let stream_conf = default_stream_config();
    let frame_hz = crate::stream::DEFAULT_FRAME_HZ;
    let interpolation_conf = crate::stream::frame::opt::InterpolationConfig::start().build();
    *conf = FrameStreamConfig {
        stream_conf,
        frame_hz,
        interpolation_conf,
    };
}

/// Initialise the given raw stream configuration with default values.
#[no_mangle]
pub unsafe extern "C" fn stream_config_default(conf: *mut StreamConfig) {
    *conf = default_stream_config();
}

/// Spawn a new frame rendering stream.
#[no_mangle]
pub unsafe extern "C" fn new_frame_stream(
    api: *const Api,
    stream: *mut *mut FrameStream,
    config: *const FrameStreamConfig,
    callback_data: *mut raw::c_void,
    frame_render_callback: FrameRenderCallback,
    process_raw_callback: RawRenderCallback,
) -> Result {
    let api: &Api = &*api;
    let model = FrameStreamModel(callback_data, frame_render_callback, process_raw_callback);

    fn render_fn(model: &mut FrameStreamModel, frame: &mut crate::stream::frame::Frame) {
        let FrameStreamModel(callback_data_ptr, frame_render_callback, _) = *model;
        let mut frame = Frame { frame };
        frame_render_callback(callback_data_ptr, &mut frame);
    }

    let mut builder = api
        .inner
        .new_frame_stream(model, render_fn)
        .point_hz((*config).stream_conf.point_hz as _)
        .latency_points((*config).stream_conf.latency_points as _)
        .frame_hz((*config).frame_hz as _);

    fn process_raw_fn(model: &mut FrameStreamModel, buffer: &mut crate::stream::raw::Buffer) {
        let FrameStreamModel(callback_data_ptr, _, process_raw_callback) = *model;
        let mut buffer = Buffer { buffer };
        process_raw_callback(callback_data_ptr, &mut buffer);
    }

    builder = builder.process_raw(process_raw_fn);

    if (*config).stream_conf.detected_dac != std::ptr::null() {
        let ffi_dac = (*(*config).stream_conf.detected_dac).clone();
        let broadcast = ffi_dac.kind.ether_dream.broadcast.clone();
        let source_addr_ptr = ffi_dac.kind.ether_dream.source_addr;
        let source_addr = CStr::from_ptr(source_addr_ptr)
            .to_string_lossy()
            .parse()
            .expect("failed to parse `source_addr`");
        let detected_dac = crate::DetectedDac::EtherDream {
            broadcast,
            source_addr,
        };
        builder = builder.detected_dac(detected_dac);
    }

    let frame_stream = match builder.build() {
        Err(_err) => {
            // TODO: Store error
            return Result::FailedToBuildStream;
        }
        Ok(stream) => Box::new(FrameStream { stream }),
    };
    *stream = Box::into_raw(frame_stream);
    Result::Success
}

/// Spawn a new frame rendering stream.
#[no_mangle]
pub unsafe extern "C" fn new_raw_stream(
    api: *const Api,
    stream: *mut *mut RawStream,
    config: *const StreamConfig,
    callback_data: *mut raw::c_void,
    process_raw_callback: RawRenderCallback,
) -> Result {
    let api: &Api = &*api;
    let model = RawStreamModel(callback_data, process_raw_callback);

    fn render_fn(model: &mut RawStreamModel, buffer: &mut crate::stream::raw::Buffer) {
        let RawStreamModel(callback_data_ptr, raw_render_callback) = *model;
        let mut buffer = Buffer { buffer };
        raw_render_callback(callback_data_ptr, &mut buffer);
    }

    let mut builder = api
        .inner
        .new_raw_stream(model, render_fn)
        .point_hz((*config).point_hz as _)
        .latency_points((*config).latency_points as _);

    if (*config).detected_dac != std::ptr::null() {
        let ffi_dac = (*(*config).detected_dac).clone();
        let broadcast = ffi_dac.kind.ether_dream.broadcast.clone();
        let source_addr_ptr = ffi_dac.kind.ether_dream.source_addr;
        let source_addr = CStr::from_ptr(source_addr_ptr)
            .to_string_lossy()
            .parse()
            .expect("failed to parse `source_addr`");
        let detected_dac = crate::DetectedDac::EtherDream {
            broadcast,
            source_addr,
        };
        builder = builder.detected_dac(detected_dac);
    }

    let raw_stream = match builder.build() {
        Err(_err) => {
            // TODO: Store error
            return Result::FailedToBuildStream;
        }
        Ok(stream) => Box::new(RawStream { stream }),
    };
    *stream = Box::into_raw(raw_stream);

    Result::Success
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
    frame.frame.add_points(points.iter().cloned());
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
    frame.frame.add_lines(points.iter().cloned());
}

pub unsafe extern "C" fn frame_hz(frame: *const Frame) -> u32 {
    (*frame).frame.frame_hz()
}

pub unsafe extern "C" fn frame_point_hz(frame: *const Frame) -> u32 {
    (*frame).frame.point_hz()
}

pub unsafe extern "C" fn frame_latency_points(frame: *const Frame) -> u32 {
    (*frame).frame.latency_points()
}

pub unsafe extern "C" fn points_per_frame(frame: *const Frame) -> u32 {
    (*frame).frame.latency_points()
}

/// Must be called in order to correctly clean up the frame stream.
#[no_mangle]
pub unsafe extern "C" fn frame_stream_drop(stream_ptr: *mut FrameStream) {
    if stream_ptr != std::ptr::null_mut() {
        Box::from_raw(stream_ptr);
    }
}

/// Must be called in order to correctly clean up the raw stream.
#[no_mangle]
pub unsafe extern "C" fn raw_stream_drop(stream_ptr: *mut RawStream) {
    if stream_ptr != std::ptr::null_mut() {
        Box::from_raw(stream_ptr);
    }
}

/// Must be called in order to correctly clean up the API resources.
#[no_mangle]
pub unsafe extern "C" fn api_drop(api_ptr: *mut Api) {
    if api_ptr != std::ptr::null_mut() {
        Box::from_raw(api_ptr);
    }
}
