use crate::wgpu;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

/// A type dedicated to capturing a texture as a non-linear sRGBA image that can be read on the
/// CPU.
///
/// Calling **capture** will return a **Snapshot** that may be read after the given command encoder
/// has been submitted. **Snapshot**s can be read on the current thread via **read** or on a thread
/// pool via **read_threaded**.
///
/// If the **Capturer** is dropped while threaded callbacks are still being processed, the drop
/// implementation will block the current thread.
#[derive(Debug)]
pub struct Capturer {
    converter_data_pair: Mutex<Option<ConverterDataPair>>,
    thread_pool: Arc<Mutex<Option<Arc<ThreadPool>>>>,
    num_threads: usize,
}

/// A snapshot captured by a **Capturer**.
///
/// A snapshot is a thin wrapper around a **wgpu::BufferImage** that knows that the image format is
/// specifically non-linear sRGBA8.
pub struct Snapshot {
    buffer: wgpu::BufferImage,
    thread_pool: Arc<Mutex<Option<Arc<ThreadPool>>>>,
    num_threads: usize,
}

/// A wrapper around a slice of bytes representing a non-linear sRGBA image.
///
/// An **ImageAsyncMapping** may only be created by reading from a **Snapshot** returned by a
/// `Texture::to_image` call.
pub struct Rgba8AsyncMapping<'a> {
    mapping: wgpu::ImageAsyncMapping<'a>,
}

#[derive(Debug)]
struct ConverterDataPair {
    src_descriptor: wgpu::TextureDescriptor,
    reshaper: wgpu::TextureReshaper,
    resolved_src_texture: Option<wgpu::Texture>,
    dst_texture: wgpu::Texture,
}

/// An alias for the image buffer that can be read from a captured **Snapshot**.
pub struct Rgba8AsyncMappedImageBuffer<'a>(
    image::ImageBuffer<image::Rgba<u8>, Rgba8AsyncMapping<'a>>,
);

impl Capturer {
    /// The format to which textures will be converted before being mapped back to the CPU.
    pub const DST_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    /// Create a new **TextureCapturer**.
    ///
    /// Note that a **TextureCapturer** must only be used with a single texture. If you require
    /// capturing multiple textures, you may create multiple **TextureCapturers**.
    pub fn new() -> Self {
        Self::with_num_threads(1)
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
            num_threads,
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

            // If the src is multisampled, add the resolve command.
            if let Some(ref resolved_src_texture) = converter_data_pair.resolved_src_texture {
                let src_view = src_texture.create_default_view();
                let resolved_view = resolved_src_texture.create_default_view();
                wgpu::resolve_texture(&src_view, &resolved_view, encoder);
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
        let mut guard = self
            .thread_pool
            .lock()
            .expect("failed to acquire thread handle");
        if let Some(thread_pool) = guard.take() {
            thread_pool.join();
        }
    }
}

impl Snapshot {
    /// Reads the non-linear sRGBA image from mapped memory.
    ///
    /// Specifically, this asynchronously maps the buffer of bytes from GPU to host memory and,
    /// once mapped, calls the given user callback with the data represented as an
    /// `Rgba8AsyncMapping`.
    ///
    /// Note: The given callback will not be called until the memory is mapped and the device is
    /// polled. You should not rely on the callback being called immediately.
    ///
    /// The given callback will be called on the current thread. If you would like the callback to
    /// be processed on a thread pool, see the `read_threaded` method.
    pub fn read<F>(&self, callback: F)
    where
        F: 'static + FnOnce(Result<Rgba8AsyncMappedImageBuffer, ()>),
    {
        let [width, height] = self.buffer.size();
        self.buffer.read(move |result| {
            let result = result.map(move |mapping| {
                let mapping = Rgba8AsyncMapping { mapping };
                Rgba8AsyncMappedImageBuffer(
                    image::ImageBuffer::from_raw(width, height, mapping)
                        .expect("image buffer dimensions did not match mapping"),
                )
            });
            callback(result);
        })
    }

    /// Similar to `read`, but rather than delivering the mapped memory directly to the callback,
    /// this method will first clone the mapped data, send it to another thread and then call the
    /// callback from the other thread.
    ///
    /// This is useful when the callback performs an operation that could take a long or unknown
    /// amount of time (e.g. writing the image to disk).
    ///
    /// Note however that if this method is called repeatedly (e.g. every frame) and the given
    /// callback takes longer than the interval between calls, then the underlying thread will fall
    /// behind and may take a while to complete by the time the application has exited.
    pub fn read_threaded<F>(&self, callback: F)
    where
        F: 'static + Send + FnOnce(Result<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, ()>),
    {
        let mut guard = self
            .thread_pool
            .lock()
            .expect("failed to acquire thread handle");
        let thread_pool = guard.get_or_insert_with(|| Arc::new(ThreadPool::new(self.num_threads)));
        let thread_pool = thread_pool.clone();
        self.read(move |result| {
            let result = result.map(|img| img.to_owned());
            thread_pool.execute(|| callback(result));
        });
    }
}

impl<'a> Rgba8AsyncMappedImageBuffer<'a> {
    /// Convert the mapped image buffer to an owned buffer.
    pub fn to_owned(&self) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let vec = self.as_flat_samples().as_slice().to_vec();
        let (width, height) = self.dimensions();
        image::ImageBuffer::from_raw(width, height, vec)
            .expect("image buffer dimensions do not match vec len")
    }
}

impl Default for Capturer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        self.finish_inner()
    }
}

impl<'a> Deref for Rgba8AsyncMapping<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> Deref for Rgba8AsyncMappedImageBuffer<'a> {
    type Target = image::ImageBuffer<image::Rgba<u8>, Rgba8AsyncMapping<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> AsRef<[u8]> for Rgba8AsyncMapping<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.mapping.mapping().data
    }
}

// Create the format converter and the target texture.
fn create_converter_data_pair(
    device: &wgpu::Device,
    src_texture: &wgpu::Texture,
) -> ConverterDataPair {
    // If the src is multisampled, it must be resolved first.
    let resolved_src_texture = if src_texture.sample_count() > 1 {
        let texture = wgpu::TextureBuilder::from(src_texture.descriptor_cloned())
            .sample_count(1)
            .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
            .build(device);
        Some(texture)
    } else {
        None
    };

    // Create the destination format texture.
    let dst_texture = wgpu::TextureBuilder::from(src_texture.descriptor_cloned())
        .sample_count(1)
        .format(Capturer::DST_FORMAT)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_SRC)
        .build(device);

    // If we have a resolved texture, use it as the conversion src. Otherwise use `src_texture`.
    let src_view = resolved_src_texture
        .as_ref()
        .map(|tex| tex.create_default_view())
        .unwrap_or_else(|| src_texture.create_default_view());

    // Create the converter.
    let dst_format = dst_texture.format();
    let src_multisampled = src_texture.sample_count() > 1;
    let dst_sample_count = 1;
    let reshaper = wgpu::TextureReshaper::new(
        device,
        &src_view,
        src_multisampled,
        dst_sample_count,
        dst_format,
    );

    // Keep track of the `src_descriptor` to check if we need to recreate the converter.
    let src_descriptor = src_texture.descriptor_cloned();

    ConverterDataPair {
        src_descriptor,
        reshaper,
        resolved_src_texture,
        dst_texture,
    }
}
