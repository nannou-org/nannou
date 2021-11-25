use image::{DynamicImage, ImageBuffer, Rgb, Rgba};
#[cfg(feature = "wgpu")]
use nannou_wgpu::{Device, Queue, Texture, TextureUsages, WithDeviceQueuePair};
use nokhwa::Camera;
#[doc(inline)]
pub use nokhwa::{
    nokhwa_check, nokhwa_initialize, query_devices, CameraControl, CameraFormat, CameraInfo,
    CaptureAPIBackend, FrameFormat, KnownCameraControlFlag, KnownCameraControls, NokhwaError,
    Resolution,
};
use parking_lot::Mutex;
use std::{
    any::Any,
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use image::buffer::ConvertBuffer;

/// The image texture struct, which is a wrapper type for `ImageBuffer<Rgb<u8>, Vec<u8>>`.
#[derive(Clone, Debug, Default)]
pub struct ImageTexture {
    buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
}

impl ImageTexture {
    /// Create a new `ImageTexture`
    pub fn new(buffer: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
        ImageTexture { buffer }
    }

    /// Get the internal buffer.
    #[inline]
    pub fn into_buffer(self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        self.buffer
    }

    /// Get the internal buffer as an RGBA
    #[inline]
    pub fn into_rgba_buffer(self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        self.buffer.convert()
    }

    /// Get the internal buffer as a `Texture`. Requires `wgpu` feature.
    #[inline]
    #[cfg(feature = "wgpu")]
    pub fn into_texture(self, dev_queue: impl WithDeviceQueuePair) -> Texture {
        Texture::from_image(dev_queue, &DynamicImage::ImageRgb8(self.buffer))
    }

    /// Get the internal buffer as a `Texture`. Requires `wgpu` feature.
    #[inline]
    #[cfg(feature = "wgpu")]
    pub fn loaded_texture_with_device_and_queue(
        self,
        device: &Device,
        queue: &Queue,
        usage: TextureUsages,
    ) -> Texture {
        Texture::load_from_image_buffer(device, queue, usage, &self.into_rgba_buffer())
    }
}

impl Deref for ImageTexture {
    type Target = ImageBuffer<Rgb<u8>, Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for ImageTexture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl Into<ImageBuffer<Rgb<u8>, Vec<u8>>> for ImageTexture {
    #[inline]
    fn into(self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        self.into_buffer()
    }
}

impl Into<DynamicImage> for ImageTexture {
    fn into(self) -> DynamicImage {
        DynamicImage::ImageRgb8(self.buffer)
    }
}

/// Creates a camera that runs in a different thread that you can use a callback to access the frames of.
/// It uses a `Arc` and a `Mutex` to ensure that this feels like a normal camera, but callback based.
/// See [`Camera`] for more details on the camera itself.
///
/// Your function is called every time there is a new frame. In order to avoid frame loss, it should
/// complete before a new frame is available. If you need to do heavy image processing, it may be
/// beneficial to directly pipe the data to a new thread to process it there.
///
/// Note that this does not have `WGPU` capabilities. However, it should be easy to implement.
/// # SAFETY
/// The `Mutex` guarantees exclusive access to the underlying camera struct. They should be safe to
/// impl `Send` on.
#[derive(Clone)]
pub struct ThreadedCamera {
    camera: Arc<Mutex<Camera>>,
    frame_callback: Arc<Mutex<Option<fn(ImageTexture)>>>,
    last_frame_captured: Arc<Mutex<ImageTexture>>,
    die_bool: Arc<AtomicBool>,
}

impl ThreadedCamera {
    /// Create a new `ThreadedCamera` from an `index` and `format`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new(index: usize, format: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        ThreadedCamera::with_backend(index, format, CaptureAPIBackend::Auto)
    }

    /// Create a new camera from an `index`, `format`, and `backend`. `format` can be `None`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn with_backend(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        Self::customized_all(index, format, backend, None)
    }

    /// Create a new `ThreadedCamera` from raw values.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn new_with(
        index: usize,
        width: u32,
        height: u32,
        fps: u32,
        fourcc: FrameFormat,
        backend: CaptureAPIBackend,
    ) -> Result<Self, NokhwaError> {
        let camera_format = CameraFormat::new_from(width, height, fourcc, fps);
        ThreadedCamera::with_backend(index, Some(camera_format), backend)
    }

    /// Create a new `ThreadedCamera` from raw values, including the raw capture function.
    ///
    /// **This is meant for advanced users only.**
    ///
    /// An example capture function can be found by clicking `[src]` and scrolling down to the bottom to function `camera_frame_thread_loop()`.
    /// # Errors
    /// This will error if you either have a bad platform configuration (e.g. `input-v4l` but not on linux) or the backend cannot create the camera (e.g. permission denied).
    pub fn customized_all(
        index: usize,
        format: Option<CameraFormat>,
        backend: CaptureAPIBackend,
        func: Option<
            fn(
                _: Arc<Mutex<Camera>>,
                _: Arc<Mutex<Option<fn(ImageTexture)>>>,
                _: Arc<Mutex<ImageTexture>>,
                _: Arc<AtomicBool>,
            ),
        >,
    ) -> Result<Self, NokhwaError> {
        let camera = Arc::new(Mutex::new(Camera::with_backend(index, format, backend)?));
        let format = match format {
            Some(fmt) => fmt,
            None => CameraFormat::default(),
        };
        let frame_callback = Arc::new(Mutex::new(None));
        let die_bool = Arc::new(AtomicBool::new(false));
        let holding_cell = Arc::new(Mutex::new(ImageTexture::new(ImageBuffer::new(
            format.width(),
            format.height(),
        ))));

        let die_clone = die_bool.clone();
        let camera_clone = camera.clone();
        let callback_clone = frame_callback.clone();
        let holding_cell_clone = holding_cell.clone();
        let func = match func {
            Some(f) => f,
            None => camera_frame_thread_loop,
        };
        std::thread::spawn(move || {
            func(camera_clone, callback_clone, holding_cell_clone, die_clone)
        });

        Ok(ThreadedCamera {
            camera,
            frame_callback,
            last_frame_captured: holding_cell,
            die_bool,
        })
    }

    /// Gets the current Camera's index.
    #[must_use]
    pub fn index(&self) -> usize {
        self.camera.lock().index()
    }

    /// Sets the current Camera's index. Note that this re-initializes the camera.
    /// # Errors
    /// The Backend may fail to initialize.
    pub fn set_index(&mut self, new_idx: usize) -> Result<(), NokhwaError> {
        self.camera.lock().set_index(new_idx)
    }

    /// Gets the current Camera's backend
    #[must_use]
    pub fn backend(&self) -> CaptureAPIBackend {
        self.camera.lock().backend()
    }

    /// Sets the current Camera's backend. Note that this re-initializes the camera.
    /// # Errors
    /// The new backend may not exist or may fail to initialize the new camera.
    pub fn set_backend(&mut self, new_backend: CaptureAPIBackend) -> Result<(), NokhwaError> {
        self.camera.lock().set_backend(new_backend)
    }

    /// Gets the camera information such as Name and Index as a [`CameraInfo`].
    #[must_use]
    pub fn info(&self) -> CameraInfo {
        self.camera.lock().info().clone()
    }

    /// Gets the current [`CameraFormat`].
    #[must_use]
    pub fn camera_format(&self) -> CameraFormat {
        self.camera.lock().camera_format()
    }

    /// Will set the current [`CameraFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new camera format, this will return an error.
    pub fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        *self.last_frame_captured.lock() =
            ImageTexture::new(ImageBuffer::new(new_fmt.width(), new_fmt.height()));
        self.camera.lock().set_camera_format(new_fmt)
    }

    /// A hashmap of [`Resolution`]s mapped to framerates
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_list_by_resolution(
        &mut self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        self.camera.lock().compatible_list_by_resolution(fourcc)
    }

    /// A Vector of compatible [`FrameFormat`]s.
    /// # Errors
    /// This will error if the camera is not queryable or a query operation has failed. Some backends will error this out as a [`UnsupportedOperationError`](crate::NokhwaError::UnsupportedOperationError).
    pub fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        self.camera.lock().compatible_fourcc()
    }

    /// Gets the current camera resolution (See: [`Resolution`], [`CameraFormat`]).
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.camera.lock().resolution()
    }

    /// Will set the current [`Resolution`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new resolution, this will return an error.
    pub fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        *self.last_frame_captured.lock() =
            ImageTexture::new(ImageBuffer::new(new_res.width(), new_res.height()));
        self.camera.lock().set_resolution(new_res)
    }

    /// Gets the current camera framerate (See: [`CameraFormat`]).
    #[must_use]
    pub fn frame_rate(&self) -> u32 {
        self.camera.lock().frame_rate()
    }

    /// Will set the current framerate
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new framerate, this will return an error.
    pub fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        self.camera.lock().set_frame_rate(new_fps)
    }

    /// Gets the current camera's frame format (See: [`FrameFormat`], [`CameraFormat`]).
    #[must_use]
    pub fn frame_format(&self) -> FrameFormat {
        self.camera.lock().frame_format()
    }

    /// Will set the current [`FrameFormat`]
    /// This will reset the current stream if used while stream is opened.
    /// # Errors
    /// If you started the stream and the camera rejects the new frame format, this will return an error.
    pub fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        self.camera.lock().set_frame_format(fourcc)
    }

    /// Gets the current supported list of [`KnownCameraControls`]
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn supported_camera_controls(&self) -> Result<Vec<KnownCameraControls>, NokhwaError> {
        self.camera.lock().supported_camera_controls()
    }

    /// Gets the current supported list of [`CameraControl`]s keyed by its name as a `String`.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn camera_controls(&self) -> Result<Vec<CameraControl>, NokhwaError> {
        let known_controls = self.supported_camera_controls()?;
        let maybe_camera_controls = known_controls
            .iter()
            .map(|x| self.camera_control(*x))
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .collect::<Vec<CameraControl>>();

        Ok(maybe_camera_controls)
    }

    /// Gets the current supported list of [`CameraControl`]s keyed by its name as a `String`.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn camera_controls_string(&self) -> Result<HashMap<String, CameraControl>, NokhwaError> {
        let known_controls = self.supported_camera_controls()?;
        let maybe_camera_controls = known_controls
            .iter()
            .map(|x| (x.to_string(), self.camera_control(*x)))
            .filter(|(_, x)| x.is_ok())
            .map(|(c, x)| (c, Result::unwrap(x)))
            .collect::<Vec<(String, CameraControl)>>();
        let mut control_map = HashMap::with_capacity(maybe_camera_controls.len());

        for (kc, cc) in maybe_camera_controls.into_iter() {
            control_map.insert(kc, cc);
        }

        Ok(control_map)
    }

    /// Gets the current supported list of [`CameraControl`]s keyed by its name as a `String`.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn camera_controls_known_camera_controls(
        &self,
    ) -> Result<HashMap<KnownCameraControls, CameraControl>, NokhwaError> {
        let known_controls = self.supported_camera_controls()?;
        let maybe_camera_controls = known_controls
            .iter()
            .map(|x| (*x, self.camera_control(*x)))
            .filter(|(_, x)| x.is_ok())
            .map(|(c, x)| (c, Result::unwrap(x)))
            .collect::<Vec<(KnownCameraControls, CameraControl)>>();
        let mut control_map = HashMap::with_capacity(maybe_camera_controls.len());

        for (kc, cc) in maybe_camera_controls.into_iter() {
            control_map.insert(kc, cc);
        }

        Ok(control_map)
    }

    /// Gets the value of [`KnownCameraControls`].
    /// # Errors
    /// If the `control` is not supported or there is an error while getting the camera control values (e.g. unexpected value, too high, etc)
    /// this will error.
    pub fn camera_control(
        &self,
        control: KnownCameraControls,
    ) -> Result<CameraControl, NokhwaError> {
        self.camera.lock().camera_control(control)
    }

    /// Sets the control to `control` in the camera.
    /// Usually, the pipeline is calling [`camera_control()`](crate::CaptureBackendTrait::camera_control()), getting a camera control that way
    /// then calling one of the methods to set the value: [`set_value()`](CameraControl::set_value()) or [`with_value()`](CameraControl::with_value()).
    /// # Errors
    /// If the `control` is not supported, the value is invalid (less than min, greater than max, not in step), or there was an error setting the control,
    /// this will error.
    pub fn set_camera_control(&mut self, control: CameraControl) -> Result<(), NokhwaError> {
        self.camera.lock().set_camera_control(control)
    }

    /// Gets the current supported list of Controls as an `Any` from the backend.
    /// The `Any`'s type is defined by the backend itself, please check each of the backend's documentation.
    /// # Errors
    /// If the list cannot be collected, this will error. This can be treated as a "nothing supported".
    pub fn raw_supported_camera_controls(&self) -> Result<Vec<Box<dyn Any>>, NokhwaError> {
        self.camera.lock().raw_supported_camera_controls()
    }

    /// Sets the control to `control` in the camera.
    /// The control's type is defined the backend itself. It may be a string, or more likely its a integer ID.
    /// The backend itself has documentation of the proper input/return values, please check each of the backend's documentation.
    /// # Errors
    /// If the `control` is not supported or there is an error while getting the camera control values (e.g. unexpected value, too high, wrong Any type)
    /// this will error.
    pub fn raw_camera_control(&self, control: &dyn Any) -> Result<Box<dyn Any>, NokhwaError> {
        self.camera.lock().raw_camera_control(control)
    }

    /// Sets the control to `control` in the camera.
    /// The `control`/`value`'s type is defined the backend itself. It may be a string, or more likely its a integer ID/Value.
    /// Usually, the pipeline is calling [`camera_control()`](crate::CaptureBackendTrait::camera_control()), getting a camera control that way
    /// then calling one of the methods to set the value: [`set_value()`](CameraControl::set_value()) or [`with_value()`](CameraControl::with_value()).
    /// # Errors
    /// If the `control` is not supported, the value is invalid (wrong Any type, backend refusal), or there was an error setting the control,
    /// this will error.
    pub fn set_raw_camera_control(
        &mut self,
        control: &dyn Any,
        value: &dyn Any,
    ) -> Result<(), NokhwaError> {
        self.camera.lock().set_raw_camera_control(control, value)
    }

    /// Will open the camera stream with set parameters. This will be called internally if you try and call [`frame()`](crate::Camera::frame()) before you call [`open_stream()`](crate::Camera::open_stream()).
    /// The callback will be called every frame.
    /// # Errors
    /// If the specific backend fails to open the camera (e.g. already taken, busy, doesn't exist anymore) this will error.
    pub fn open_stream(&mut self, callback: fn(ImageTexture)) -> Result<(), NokhwaError> {
        *self.frame_callback.lock() = Some(callback);
        self.camera.lock().open_stream()
    }

    /// Sets the frame callback to the new specified function. This function will be called instead of the previous one(s).
    pub fn set_callback(&mut self, callback: fn(ImageTexture)) {
        *self.frame_callback.lock() = Some(callback);
    }

    /// Polls the camera for a frame, analogous to [`Camera::frame`](crate::Camera::frame)
    pub fn poll_frame(&mut self) -> Result<ImageTexture, NokhwaError> {
        let frame: ImageTexture = ImageTexture::new(self.camera.lock().frame()?);
        *self.last_frame_captured.lock() = frame.clone();
        Ok(frame)
    }

    /// Polls the camera as a texture. Internally calls `poll_frame()`
    #[cfg(feature = "wgpu")]
    pub fn poll_texture(
        &mut self,
        dev_queue: impl WithDeviceQueuePair,
    ) -> Result<Texture, NokhwaError> {
        Ok(self.poll_frame()?.into_texture(dev_queue))
    }

    /// Polls the camera as a texture. Internally calls `poll_frame()`
    #[cfg(feature = "wgpu")]
    pub fn poll_texture_with_device_queue_usage(
        &mut self,
        device: &Device,
        queue: &Queue,
        usages: TextureUsages,
    ) -> Result<Texture, NokhwaError> {
        Ok(self
            .poll_frame()?
            .loaded_texture_with_device_and_queue(device, queue, usages))
    }

    /// Gets the last frame captured by the camera.
    pub fn last_frame(&self) -> ImageTexture {
        self.last_frame_captured.lock().clone()
    }

    /// The last frame from the camera as a texture. Internally calls `poll_frame()`
    #[cfg(feature = "wgpu")]
    pub fn last_frame_texture(&self, dev_queue: impl WithDeviceQueuePair) -> Texture {
        self.last_frame().into_texture(dev_queue)
    }

    /// The last frame from the camera as a texture. Internally calls `poll_frame()`
    #[cfg(feature = "wgpu")]
    pub fn last_frame_texture_with_device_queue_usage(
        &self,
        device: &Device,
        queue: &Queue,
        usages: TextureUsages,
    ) -> Texture {
        self.last_frame()
            .loaded_texture_with_device_and_queue(device, queue, usages)
    }

    /// Checks if stream if open. If it is, it will return true.
    pub fn is_stream_open(&self) -> bool {
        self.camera.lock().is_stream_open()
    }

    /// Will drop the stream.
    /// # Errors
    /// Please check the `Quirks` section of each backend.
    pub fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        self.camera.lock().stop_stream()
    }
}

impl Drop for ThreadedCamera {
    fn drop(&mut self) {
        let _ = self.stop_stream();
        self.die_bool.store(true, Ordering::SeqCst);
    }
}

fn camera_frame_thread_loop(
    camera: Arc<Mutex<Camera>>,
    callback: Arc<Mutex<Option<fn(ImageTexture)>>>,
    holding_cell: Arc<Mutex<ImageTexture>>,
    die_bool: Arc<AtomicBool>,
) {
    loop {
        if let Ok(img) = camera.lock().frame() {
            let texture = ImageTexture::new(img);
            *holding_cell.lock() = texture.clone();
            if let Some(cb) = callback.lock().deref() {
                cb(texture)
            }
        }
        if die_bool.load(Ordering::SeqCst) {
            break;
        }
    }
}
