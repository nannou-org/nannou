//! Expose a C compatible interface.

use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::io;
use std::os::raw;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[repr(C)]
pub struct Api {
    inner: crate::Api,
    last_error: Option<CString>,
}

#[repr(C)]
pub struct DetectDacsAsync {
    inner: crate::DetectDacsAsync,
    dacs: Arc<Mutex<HashMap<crate::DacId, (Instant, crate::DetectedDac)>>>,
    last_error: Arc<Mutex<Option<CString>>>,
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
    pub source_addr: SocketAddr,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StreamConfig {
    pub detected_dac: *const DetectedDac,
    pub point_hz: raw::c_uint,
    pub latency_points: raw::c_uint,
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
    DetectDacFailed,
    BuildStreamFailed,
    DetectDacsAsyncFailed,
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

/// Given some uninitialised pointer to a `DetectDacsAsync` struct, fill it with a new instance.
///
/// If the given `timeout_secs` is `0`, DAC detection will never timeout and detected DACs that no
/// longer broadcast will remain accessible in the device map.
#[no_mangle]
pub unsafe extern "C" fn detect_dacs_async(
    api: *mut Api,
    timeout_secs: raw::c_float,
    detect_dacs: *mut *mut DetectDacsAsync,
) -> Result {
    let api: &mut Api = &mut *api;
    let duration = if timeout_secs == 0.0 {
        None
    } else {
        let secs = timeout_secs as u64;
        let nanos = ((timeout_secs - secs as raw::c_float) * 1_000_000_000.0) as u32;
        Some(std::time::Duration::new(secs, nanos))
    };
    let res = detect_dacs_async_inner(&api.inner, duration);
    let boxed_detect_dacs = match res {
        Ok(detector) => Box::new(detector),
        Err(err) => {
            api.last_error = Some(err_to_cstring(&err));
            return Result::DetectDacsAsyncFailed;
        }
    };
    *detect_dacs = Box::into_raw(boxed_detect_dacs);
    Result::Success
}

/// Retrieve a list of the currently available DACs.
#[no_mangle]
pub unsafe extern "C" fn available_dacs(
    detect_dacs_async: *mut DetectDacsAsync,
    first_dac: *mut *mut DetectedDac,
    len: *mut raw::c_uint,
) {
    let detect_dacs_async: &mut DetectDacsAsync = &mut *detect_dacs_async;
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
    let api: &mut Api = &mut *api;
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
    api: *mut Api,
    stream: *mut *mut FrameStream,
    config: *const FrameStreamConfig,
    callback_data: *mut raw::c_void,
    frame_render_callback: FrameRenderCallback,
    process_raw_callback: RawRenderCallback,
) -> Result {
    let api: &mut Api = &mut *api;
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
        let detected_dac = detected_dac_from_ffi(ffi_dac);
        builder = builder.detected_dac(detected_dac);
    }

    let frame_stream = match builder.build() {
        Err(err) => {
            api.last_error = Some(err_to_cstring(&err));
            return Result::BuildStreamFailed;
        }
        Ok(stream) => Box::new(FrameStream { stream }),
    };
    *stream = Box::into_raw(frame_stream);
    Result::Success
}

/// Spawn a new frame rendering stream.
#[no_mangle]
pub unsafe extern "C" fn new_raw_stream(
    api: *mut Api,
    stream: *mut *mut RawStream,
    config: *const StreamConfig,
    callback_data: *mut raw::c_void,
    process_raw_callback: RawRenderCallback,
) -> Result {
    let api: &mut Api = &mut *api;
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
        let detected_dac = detected_dac_from_ffi(ffi_dac);
        builder = builder.detected_dac(detected_dac);
    }

    let raw_stream = match builder.build() {
        Err(err) => {
            api.last_error = Some(err_to_cstring(&err));
            return Result::BuildStreamFailed;
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

#[no_mangle]
pub unsafe extern "C" fn frame_hz(frame: *const Frame) -> u32 {
    (*frame).frame.frame_hz()
}

#[no_mangle]
pub unsafe extern "C" fn frame_point_hz(frame: *const Frame) -> u32 {
    (*frame).frame.point_hz()
}

#[no_mangle]
pub unsafe extern "C" fn frame_latency_points(frame: *const Frame) -> u32 {
    (*frame).frame.latency_points()
}

#[no_mangle]
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

/// Must be called in order to correctly clean up the `DetectDacsAsync` resources.
#[no_mangle]
pub unsafe extern "C" fn detect_dacs_async_drop(ptr: *mut DetectDacsAsync) {
    if ptr != std::ptr::null_mut() {
        Box::from_raw(ptr);
    }
}

/// Must be called in order to correctly clean up the API resources.
#[no_mangle]
pub unsafe extern "C" fn api_drop(api_ptr: *mut Api) {
    if api_ptr != std::ptr::null_mut() {
        Box::from_raw(api_ptr);
    }
}

/// Used for retrieving the last error that occurred from the API.
#[no_mangle]
pub unsafe extern "C" fn api_last_error(api: *const Api) -> *const raw::c_char {
    let api: &Api = &*api;
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
    let detect_dacs_async: &DetectDacsAsync = &*detect;
    let mut s = std::ptr::null();
    if let Ok(last_error) = detect_dacs_async.last_error.lock() {
        if let Some(ref cstring) = *last_error {
            s = cstring.as_ptr();
        }
    }
    s
}

/// Begin asynchronous DAC detection.
///
/// The given timeout corresponds to the duration of time since the last DAC broadcast was received
/// before a DAC will be considered to be unavailable again.
fn detect_dacs_async_inner(
    api: &crate::Api,
    dac_timeout: Option<Duration>,
) -> io::Result<DetectDacsAsync> {
    let dacs = Arc::new(Mutex::new(HashMap::new()));
    let last_error = Arc::new(Mutex::new(None));
    let dacs2 = dacs.clone();
    let last_error2 = last_error.clone();
    let inner = api.detect_dacs_async(
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
    Ok(DetectDacsAsync {
        inner,
        dacs,
        last_error,
    })
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
    StreamConfig {
        detected_dac,
        point_hz,
        latency_points,
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
