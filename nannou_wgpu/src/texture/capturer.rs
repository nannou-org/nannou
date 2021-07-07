use crate as wgpu;
use std::fmt;
use std::future::Future;
use std::sync::atomic::{self, AtomicU32};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use image::{GenericImage, GenericImageView};

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
    workers: Option<u32>,
    timeout: Option<Duration>,
}

/// A wrapper around the futures thread pool that counts active futures.
#[derive(Debug)]
struct ThreadPool {
    thread_pool: futures::executor::ThreadPool,
    active_futures: Arc<AtomicU32>,
    workers: u32,
    timeout: Option<Duration>,
}

/// A snapshot captured by a **Capturer**.
///
/// A snapshot is a thin wrapper around a **wgpu::BufferImage** that knows that the image format is
/// specifically non-linear sRGBA8.
pub struct Snapshot {
    buffer: wgpu::RowPaddedBuffer,
    thread_pool: Arc<Mutex<Option<Arc<ThreadPool>>>>,
    workers: Option<u32>,
    timeout: Option<Duration>,
}

/// An error indicating that the threadpool timed out while waiting for a worker to become
/// available.
pub struct AwaitWorkerTimeout<F>(pub F);

#[derive(Debug)]
struct ConverterDataPair {
    src_descriptor: wgpu::TextureDescriptor<'static>,
    reshaper: wgpu::TextureReshaper,
    dst_texture: wgpu::Texture,
}

/// A wrapper around a slice of bytes representing a non-linear sRGBA image.
///
/// Can be read from a captured `Snapshot`.
pub struct Rgba8AsyncMappedImageBuffer<'buffer>(wgpu::ImageReadMapping<'buffer>);

impl ThreadPool {
    /// Spawns the given future if a worker is available. Otherwise, blocks and waits for a worker
    /// to become available before spawning the future.
    fn spawn_when_worker_available<F>(&self, future: F) -> Result<(), AwaitWorkerTimeout<F>>
    where
        F: 'static + Future<Output = ()> + Send,
    {
        // Wait until the number of active futures is less than the number of threads.
        // If we don't wait, the capture futures may quickly fall far behind the main
        // swapchain thread resulting in an out of memory error.
        let mut start = None;
        let mut interval_us = 128;
        while self.active_futures() >= self.workers() {
            if let Some(timeout) = self.timeout {
                let start = start.get_or_insert_with(instant::Instant::now);
                if start.elapsed() > timeout {
                    return Err(AwaitWorkerTimeout(future));
                }
            }
            let duration = Duration::from_micros(interval_us);
            std::thread::sleep(duration);
            interval_us *= 2;
        }

        // Wrap the future with the counter.
        let active_futures = self.active_futures.clone();
        let future = async move {
            active_futures.fetch_add(1, atomic::Ordering::SeqCst);
            future.await;
            active_futures.fetch_sub(1, atomic::Ordering::SeqCst);
        };

        self.thread_pool.spawn_ok(future);
        Ok(())
    }

    fn active_futures(&self) -> u32 {
        self.active_futures.load(atomic::Ordering::SeqCst)
    }

    fn workers(&self) -> u32 {
        self.workers
    }

    /// Await for the completion of all active futures, polling the device as necessary until all
    /// futures have completed.
    fn await_active_futures(&self, device: &wgpu::Device) -> Result<(), AwaitWorkerTimeout<()>> {
        let mut start = None;
        let mut interval_us = 128;
        while self.active_futures() > 0 {
            if let Some(timeout) = self.timeout {
                let start = start.get_or_insert_with(instant::Instant::now);
                if start.elapsed() > timeout {
                    return Err(AwaitWorkerTimeout(()));
                }
            }
            device.poll(wgpu::Maintain::Wait);
            let duration = Duration::from_micros(interval_us);
            std::thread::sleep(duration);
            interval_us *= 2;
        }
        Ok(())
    }
}

impl Capturer {
    /// The format to which textures will be converted before being mapped back to the CPU.
    pub const DST_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    /// Create a new **TextureCapturer**.
    ///
    /// Note that a **TextureCapturer** must only be used with a single texture. If you require
    /// capturing multiple textures, you may create multiple **TextureCapturers**.
    ///
    /// `workers` refers to the number of worker threads used to await GPU buffers to be mapped for
    /// reading and for running user callbacks. If `None` is specified, a threadpool will be
    /// spawned with a number of threads equal to the number of CPUs available on the system.
    ///
    /// `timeout` specifies how long to block and wait for an available worker in the case that all
    /// workers are busy at the time a `Snapshot::read` occurs. If `None` is specified, calls to
    /// `Snapshot::read` will never time out (the default behaviour).
    ///
    /// Note that the specified parameters are only relevant to calls to `Snapshot::read`. In the
    /// case that the user uses `Snapshot::read_async`, it is the responsibility of the user to
    /// execute the future.
    pub fn new(workers: Option<u32>, timeout: Option<Duration>) -> Self {
        Capturer {
            converter_data_pair: Default::default(),
            thread_pool: Default::default(),
            workers,
            timeout,
        }
    }

    /// The number of futures currently running on the inner `ThreadPool`.
    ///
    /// Note that futures are only run on the threadpool when the `Snapshot::read` method is used.
    /// In the case that `Snapshot::read_async` is used it is up to the user to track their
    /// futures.
    ///
    /// If the inner thread pool mutex has been poisoned, or if the thread pool has not been
    /// created due to no calls to `read`, this will return `0`.
    pub fn active_snapshots(&self) -> u32 {
        if let Ok(guard) = self.thread_pool.lock() {
            if let Some(tp) = guard.as_ref() {
                return tp.active_futures.load(atomic::Ordering::SeqCst);
            }
        }
        0
    }

    /// The number of worker threads used to await GPU buffers to be mapped for reading and for
    /// running user callbacks.
    pub fn workers(&self) -> u32 {
        if let Ok(guard) = self.thread_pool.lock() {
            if let Some(tp) = guard.as_ref() {
                return tp.workers();
            }
        }
        self.workers.unwrap_or(num_cpus::get() as u32)
    }

    /// Capture the given texture at the state of the given command encoder.
    pub fn capture(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        src_texture: &wgpu::Texture,
    ) -> Snapshot {
        let buffer = if src_texture.format() != Self::DST_FORMAT {
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
            let dst_view = converter_data_pair.dst_texture.view();
            converter_data_pair
                .reshaper
                .encode_render_pass(&dst_view.build(), encoder);

            converter_data_pair.dst_texture.to_buffer(device, encoder)
        } else {
            src_texture.to_buffer(device, encoder)
        };

        Snapshot {
            buffer,
            thread_pool: self.thread_pool.clone(),
            workers: self.workers,
            timeout: self.timeout,
        }
    }

    /// Await for the completion of all `Snapshot::read` active futures, polling the device as
    /// necessary until all futures have reached completion or until a timeout is reached.
    pub fn await_active_snapshots(
        &self,
        device: &wgpu::Device,
    ) -> Result<(), AwaitWorkerTimeout<()>> {
        if let Ok(guard) = self.thread_pool.lock() {
            if let Some(tp) = guard.as_ref() {
                return tp.await_active_futures(device);
            }
        }
        Ok(())
    }
}

impl Snapshot {
    /// Reads the non-linear sRGBA image from mapped memory and convert it to an owned buffer.
    pub async fn read_async<'buffer>(
        &'buffer self,
    ) -> Result<Rgba8AsyncMappedImageBuffer<'buffer>, wgpu::BufferAsyncError> {
        let mapping = self.buffer.read().await?;
        Ok(Rgba8AsyncMappedImageBuffer(mapping))
    }

    /// The same as `read_async`, but runs the resulting future on an inner threadpool and calls
    /// the given callback with the mapped image buffer once complete.
    ///
    /// Note: The given callback will not be called until the memory is mapped and the device is
    /// polled. You should not rely on the callback being called immediately.
    ///
    /// Note: The given callback will be called on the inner thread pool and will not be called on
    /// the current thread.
    ///
    /// Note: **This method may block** if the associated `wgpu::TextureCapturer` has an
    /// `active_futures` count that is greater than the number of worker threads with which it was
    /// created. This is necessary in order to avoid "out of memory" errors resulting from an
    /// accumulating queue of pending texture buffers waiting to be mapped. To avoid blocking, you
    /// can try using a higher thread count, capturing a smaller texture, or using `read_async`
    /// instead and running the resulting future on a custom runtime or threadpool.
    pub fn read<F>(self, callback: F) -> Result<(), AwaitWorkerTimeout<impl Future<Output = ()>>>
    where
        F: 'static + Send + FnOnce(Result<Rgba8AsyncMappedImageBuffer, wgpu::BufferAsyncError>),
    {
        let thread_pool = self.thread_pool();
        let read_future = async move {
            let res = self.read_async().await;
            callback(res);
        };
        thread_pool.spawn_when_worker_available(read_future)
    }

    fn thread_pool(&self) -> Arc<ThreadPool> {
        let mut guard = self
            .thread_pool
            .lock()
            .expect("failed to acquire thread handle");
        let thread_pool = guard.get_or_insert_with(|| {
            let workers = self.workers.unwrap_or(num_cpus::get() as u32);
            let thread_pool = futures::executor::ThreadPoolBuilder::new()
                .pool_size(workers as usize)
                .create()
                .expect("failed to create thread pool");
            let thread_pool = ThreadPool {
                thread_pool,
                active_futures: Arc::new(AtomicU32::new(0)),
                workers,
                timeout: self.timeout,
            };
            Arc::new(thread_pool)
        });
        thread_pool.clone()
    }
}

impl<'b> Rgba8AsyncMappedImageBuffer<'b> {
    pub fn as_image(&self) -> image::SubImage<wgpu::ImageHolder<image::Rgba<u8>>> {
        // safe: we know it's Rgba<u8>
        unsafe { self.0.as_image::<image::Rgba<u8>>() }
    }
    /// Convert the mapped image buffer to an owned buffer.
    pub fn to_owned(&self) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let view = self.as_image();
        let mut result = image::ImageBuffer::new(view.width(), view.height());
        result
            .copy_from(&view, 0, 0)
            .expect("nannou internal error: image copy failed");
        result
    }
}

impl<T> std::error::Error for AwaitWorkerTimeout<T> {}

impl<T> fmt::Debug for AwaitWorkerTimeout<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AwaitWorkerTimeout").finish()
    }
}

impl<T> fmt::Display for AwaitWorkerTimeout<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AwaitWorkerTimeout").finish()
    }
}

// Create the format converter and the target texture.
fn create_converter_data_pair(
    device: &wgpu::Device,
    src_texture: &wgpu::Texture,
) -> ConverterDataPair {
    // Create the destination format texture.
    let dst_texture = wgpu::TextureBuilder::from(src_texture.descriptor.clone())
        .sample_count(1)
        .format(Capturer::DST_FORMAT)
        .usage(wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_SRC)
        .build(device);

    // Create the converter.
    let src_sample_count = src_texture.sample_count();
    let src_sample_type = src_texture.sample_type();
    let src_view = src_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let dst_sample_count = 1;
    let dst_format = dst_texture.format();
    let reshaper = wgpu::TextureReshaper::new(
        device,
        &src_view,
        src_sample_count,
        src_sample_type,
        dst_sample_count,
        dst_format,
    );

    // Keep track of the `src_descriptor` to check if we need to recreate the converter.
    let src_descriptor = src_texture.descriptor.clone();

    ConverterDataPair {
        src_descriptor,
        reshaper,
        dst_texture,
    }
}
