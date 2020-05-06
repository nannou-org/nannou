use crate::wgpu;
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use futures::future::FutureExt;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// A type dedicated to capturing a texture as a non-linear sRGBA image that can be read on the
/// CPU.
///
/// Calling **capture** will return a **Snapshot** that may be read after the given command encoder
/// has been submitted. **Snapshot**s can be read on the current thread via **read** or on a thread
/// pool via **read_threaded**.
///
/// If the **Capturer** is dropped while threaded callbacks are still being processed, the drop
/// implementation will block the current thread.
#[derive(Debug, Default)]
pub struct Capturer {
    converter_data_pair: Mutex<Option<ConverterDataPair>>,
    thread_pool: Arc<Mutex<Option<Arc<ThreadPool>>>>,
    num_threads: Option<usize>,
}

/// A snapshot captured by a **Capturer**.
///
/// A snapshot is a thin wrapper around a **wgpu::BufferImage** that knows that the image format is
/// specifically non-linear sRGBA8.
pub struct Snapshot {
    buffer: wgpu::BufferImage,
    thread_pool: Arc<Mutex<Option<Arc<ThreadPool>>>>,
    num_threads: Option<usize>,
}

/// A wrapper around a slice of bytes representing a non-linear sRGBA image.
///
/// An **ImageReadMapping** may only be created by reading from a **Snapshot** returned by a
/// `Texture::to_image` call.
pub struct Rgba8ReadMapping {
    mapping: wgpu::ImageReadMapping,
}

#[derive(Debug)]
struct ConverterDataPair {
    src_descriptor: wgpu::TextureDescriptor<'static>,
    reshaper: wgpu::TextureReshaper,
    dst_texture: wgpu::Texture,
}

/// An alias for the image buffer that can be read from a captured **Snapshot**.
pub struct Rgba8AsyncMappedImageBuffer(
    image::ImageBuffer<image::Rgba<u8>, Rgba8ReadMapping>,
);

impl Capturer {
    /// The format to which textures will be converted before being mapped back to the CPU.
    pub const DST_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    /// Create a new **TextureCapturer**.
    ///
    /// Note that a **TextureCapturer** must only be used with a single texture. If you require
    /// capturing multiple textures, you may create multiple **TextureCapturers**.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same as **new** but allows for specifying the number of threads to use when processing
    /// callbacks submitted to `read_threaded` on produced snapshots.
    ///
    /// By default, **Capturer** uses a single dedicated thread. This reduces the chance that the
    /// thread will interfere with the core running the main event loop, but also reduces the
    /// amount of processing power applied to processing callbacks in turn increasing the chance
    /// that the thread may fall behind under heavy load. This constructor is provided to allow for
    /// users to choose how to handle this trade-off.
    pub fn with_num_threads(num_threads: usize) -> Self {
        Self {
            converter_data_pair: Default::default(),
            thread_pool: Default::default(),
            num_threads: Some(num_threads),
        }
    }

    /// Capture the given texture at the state of the given command encoder.
    pub fn capture(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        src_texture: &wgpu::Texture,
    ) -> Snapshot {
        let buffer_image = if src_texture.format() != Self::DST_FORMAT {
            let mut converter_data_pair = self
                .converter_data_pair
                .lock()
                .expect("failed to lock converter");

            // Create converter and target texture if they don't exist.
            let converter_data_pair = converter_data_pair
                .get_or_insert_with(|| create_converter_data_pair(device, src_texture));

            // If the texture has changed in some way, recreate the converter.
            if !wgpu::texture_descriptor_eq(
                src_texture.descriptor(),
                &converter_data_pair.src_descriptor,
            ) {
                *converter_data_pair = create_converter_data_pair(device, src_texture);
            }

            // Encode the texture format conversion.
            let dst_view = converter_data_pair.dst_texture.create_default_view();
            converter_data_pair
                .reshaper
                .encode_render_pass(&dst_view, encoder);

            converter_data_pair
                .dst_texture
                .to_image(device, encoder)
                .expect("texture has unsupported format")
        } else {
            src_texture
                .to_image(device, encoder)
                .expect("texture has unsupported format")
        };

        Snapshot {
            buffer: buffer_image,
            thread_pool: self.thread_pool.clone(),
            num_threads: self.num_threads,
        }
    }

    /// Finish capturing and wait for any threaded callbacks to complete if there are any.
    pub fn finish(self) {
        self.finish_inner()
    }

    fn finish_inner(&self) {
        unimplemented!("wait for all active snapshots to complete");
    }
}

impl Snapshot {
    /// Reads the non-linear sRGBA image from mapped memory.
    ///
    /// Specifically, this asynchronously maps the buffer of bytes from GPU to host memory and,
    /// once mapped, calls the given user callback with the data represented as an
    /// `Rgba8ReadMapping`.
    ///
    /// Note: The given callback will not be called until the memory is mapped and the device is
    /// polled. You should not rely on the callback being called immediately.
    ///
    /// The given callback will be called on the current thread. If you would like the callback to
    /// be processed on a thread pool, see the `read_threaded` method.
    pub async fn read_async(self) -> Result<Rgba8AsyncMappedImageBuffer, wgpu::BufferAsyncErr> {
        let [width, height] = self.buffer.size();
        let mapping = self.buffer.read().await?;
        let mapping = Rgba8ReadMapping { mapping };
        Ok(Rgba8AsyncMappedImageBuffer(
            image::ImageBuffer::from_raw(width, height, mapping)
                .expect("image buffer dimensions did not match mapping"),
        ))
    }

    /// TODO:
    /// - Remove `read_threaded` in favour of specifying num threads.
    /// - Count the number of active snapshots.
    /// - Block after `view` when `num_threads` number of snapshots are active.
    pub fn read<F>(self, callback: F)
    where
        F: 'static + Send + FnOnce(Result<Rgba8AsyncMappedImageBuffer, wgpu::BufferAsyncErr>),
    {
        let thread_pool = self.thread_pool();
        let read_future = self.read_async().map(|res| {
            // TODO:
            unimplemented!();
            callback(res);
        });
        thread_pool.spawn_ok(read_future);
    }

    // /// Similar to `read`, but rather than delivering the mapped memory directly to the callback,
    // /// this method will first clone the mapped data, send it to another thread and then call the
    // /// callback from the other thread.
    // ///
    // /// This is useful when the callback performs an operation that could take a long or unknown
    // /// amount of time (e.g. writing the image to disk).
    // ///
    // /// Note however that if this method is called repeatedly (e.g. every frame) and the given
    // /// callback takes longer than the interval between calls, then the underlying thread will fall
    // /// behind and may take a while to complete by the time the application has exited.
    // pub async fn read_threaded(&self) -> Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, ()> {

    //     let thread_pool = thread_pool.clone();
    //     thread_pool.spawn_ok(
    //     let
    //     self.read()
    //         .map(|result| result.map(|img| img.to_owned()))
    //         .map(|result|

    //     self.read(move |result| {
    //         let result = result.map(|img| img.to_owned());
    //         thread_pool.execute(|| callback(result));
    //     });
    // }

    fn thread_pool(&self) -> Arc<ThreadPool> {
        let mut guard = self
            .thread_pool
            .lock()
            .expect("failed to acquire thread handle");
        let thread_pool = guard.get_or_insert_with(|| {
            let thread_pool = self
                .num_threads
                .map(|n| {
                    ThreadPoolBuilder::new()
                        .pool_size(n)
                        .create()
                })
                .unwrap_or_else(ThreadPool::new)
                .expect("failed to create thread pool");
            Arc::new(thread_pool)
        });
        thread_pool.clone()
    }
}

impl Rgba8AsyncMappedImageBuffer {
    /// Convert the mapped image buffer to an owned buffer.
    pub fn to_owned(&self) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let vec = self.as_flat_samples().as_slice().to_vec();
        let (width, height) = self.dimensions();
        image::ImageBuffer::from_raw(width, height, vec)
            .expect("image buffer dimensions do not match vec len")
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        self.finish_inner()
    }
}

impl Deref for Rgba8ReadMapping {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Deref for Rgba8AsyncMappedImageBuffer {
    type Target = image::ImageBuffer<image::Rgba<u8>, Rgba8ReadMapping>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Rgba8ReadMapping {
    fn as_ref(&self) -> &[u8] {
        self.mapping.mapping().as_slice()
    }
}

// Create the format converter and the target texture.
fn create_converter_data_pair(
    device: &wgpu::Device,
    src_texture: &wgpu::Texture,
) -> ConverterDataPair {
    // Create the destination format texture.
    let dst_texture = wgpu::TextureBuilder::from(src_texture.descriptor_cloned())
        .sample_count(1)
        .format(Capturer::DST_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC)
        .build(device);

    // Create the converter.
    let src_sample_count = src_texture.sample_count();
    let src_component_type = src_texture.component_type();
    let src_view = src_texture.create_default_view();
    let dst_sample_count = 1;
    let dst_format = dst_texture.format();
    let reshaper = wgpu::TextureReshaper::new(
        device,
        &src_view,
        src_sample_count,
        src_component_type,
        dst_sample_count,
        dst_format,
    );

    // Keep track of the `src_descriptor` to check if we need to recreate the converter.
    let src_descriptor = src_texture.descriptor_cloned();

    ConverterDataPair {
        src_descriptor,
        reshaper,
        dst_texture,
    }
}
