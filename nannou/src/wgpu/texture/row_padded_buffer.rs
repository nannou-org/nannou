use super::image::Pixel;
use crate::wgpu;
use std::ops::Deref;

/// A wrapper around a wgpu buffer suitable for copying to and from Textures. Automatically handles
/// the padding necessary for buffer-to-texture and texture-to-buffer copies.
///
/// Note: as of `wgpu` 0.6, texture-to-buffer and buffer-to-texture copies require that image rows
/// are padded to a multiple `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT` bytes. Note that this is a
/// requirement on the *buffers*, not on the textures! You can have textures of whatever size you
/// like, but when you copy them to/from a buffer, the *buffer rows* need padding. This is referred
/// to as "pitch alignment".
///
/// In a `RowPaddedBuffer`, the image is stored in row-major order, with rows padded at the end with
/// uninitialized bytes to reach the necessary size.
#[derive(Debug)]
pub struct RowPaddedBuffer {
    /// The width of the buffer in bytes, *without padding*.
    width: u32,
    /// The padding on each row of the buffer in bytes.
    row_padding: u32,
    /// The height of the buffer.
    height: u32,
    /// The wrapped buffer handle.
    buffer: wgpu::Buffer,
    /// The descriptor used to create the wrapped buffer.
    buffer_descriptor: wgpu::BufferDescriptor<'static>,
}

impl RowPaddedBuffer {
    /// Create a row-padded buffer on the device.
    ///
    /// Width should be given in bytes.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        usage: wgpu::BufferUsage,
    ) -> RowPaddedBuffer {
        let row_padding = RowPaddedBuffer::compute_row_padding(width);

        // only create mapped for buffers that we're going to write to.
        let mapped_at_creation = usage.contains(wgpu::BufferUsage::MAP_WRITE);

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("nannou::RowPaddedBuffer"),
            size: ((width + row_padding) * height) as u64,
            usage,
            mapped_at_creation,
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        RowPaddedBuffer {
            width,
            row_padding,
            height,
            buffer,
            buffer_descriptor,
        }
    }

    /// Initialize from an image buffer (i.e. an image on CPU).
    pub fn from_image_buffer<P, Container>(
        device: &wgpu::Device,
        image_buffer: &image::ImageBuffer<P, Container>,
    ) -> RowPaddedBuffer
    where
        P: 'static + Pixel,
        Container: std::ops::Deref<Target = [P::Subpixel]>,
    {
        let result = RowPaddedBuffer::new(
            device,
            image_buffer.width() * P::COLOR_TYPE.bytes_per_pixel() as u32,
            image_buffer.height(),
            wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
        );
        // TODO:
        // This can theoretically be exploited by implementing `image::Primitive` for some type
        // that has padding. Instead, should make some `Subpixel` trait that we can control and is
        // only guaranteed to be implemented for safe types.
        result.write(unsafe { wgpu::bytes::from_slice(&*image_buffer) });
        result
    }

    /// Creates a buffer compatible with a 2d slice of the given texture.
    pub fn for_texture(
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        usage: wgpu::BufferUsage,
    ) -> RowPaddedBuffer {
        RowPaddedBuffer::new(
            device,
            texture.extent().width * wgpu::texture_format_size_bytes(texture.format()),
            texture.extent().height,
            usage,
        )
    }

    /// Compute the necessary padding for each row.
    fn compute_row_padding(width: u32) -> u32 {
        wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - (width % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
    }

    /// The width of the buffer, in bytes, NOT including padding bytes.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The padding of each row of this buffer.
    pub fn row_padding(&self) -> u32 {
        self.row_padding
    }

    /// The width of the buffer, in bytes, INCLUDING padding bytes.
    pub fn padded_width(&self) -> u32 {
        self.width + self.row_padding
    }

    /// The height of the buffer.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Copy data into the padded buffer.
    ///
    /// Will copy `data_width` bytes of data into each row of the buffer, leaving the remainder
    /// of the buffer unmodified.
    ///
    /// The buffer usage must include `BufferUsage::map_read()`.
    pub fn write(&self, buf: &[u8]) {
        assert_eq!(
            (self.width * self.height) as usize,
            buf.len(),
            "Incorrect input slice size"
        );
        assert!(
            self.buffer_descriptor
                .usage
                .contains(wgpu::BufferUsage::MAP_WRITE),
            "Wrapped buffer cannot be mapped for writing"
        );

        let mut mapped = self.buffer.slice(..).get_mapped_range_mut();
        let mapped = &mut mapped[..];

        let width = self.width as usize;
        let padded_width = width + self.row_padding as usize;
        let height = self.height as usize;
        for row in 0..height {
            let in_start = row * width;
            let out_start = row * padded_width;

            // note: leaves mapped[out_start + width..out_start + padded_width] uninitialized!
            mapped[out_start..out_start + width].copy_from_slice(&buf[in_start..in_start + width]);
        }
    }

    /// Asynchronously maps the buffer of bytes from GPU to host memory and, once mapped, calls the
    /// given user callback with the data represented as an `ImageReadMapping`.
    ///
    /// Note: The given callback will not be called until the memory is mapped and the device is
    /// polled. You should not rely on the callback being called immediately.
    pub async fn read<'b>(&'b self) -> Result<ImageReadMapping<'b>, wgpu::BufferAsyncError> {
        let slice = self.buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read).await?;
        Ok(wgpu::ImageReadMapping {
            buffer: self,
            // fun exercise:
            // read the signature of wgpu::BufferSlice::get_mapped_range()
            // and try to figure out why we don't need another lifetime in ImageReadMapping :)
            view: slice.get_mapped_range(),
        })
    }

    /// Encode a copy into a texture. Assumes the texture is 2d.
    ///
    /// The copy will not be performed until the encoded command buffer is submitted.
    pub fn encode_copy_into(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::Texture,
    ) {
        assert_eq!(
            destination.extent().depth,
            1,
            "use encode_copy_into_at for 3d textures"
        );
        self.encode_copy_into_at(encoder, destination, 0);
    }

    /// Encode a copy into a 3d texture at a given depth. Will copy this buffer (modulo padding)
    /// to a slice of the texture at the given depth.
    ///
    /// The copy will not be performed until the encoded command buffer is submitted.
    pub fn encode_copy_into_at(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::Texture,
        depth: u32,
    ) {
        let (source, destination, copy_size) = self.copy_views(destination, depth);
        encoder.copy_buffer_to_texture(source, destination, copy_size);
    }

    /// Encode a copy from a texture.
    ///
    /// The copy will not be performed until the encoded command buffer is submitted.
    pub fn encode_copy_from(&self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Texture) {
        assert_eq!(
            source.extent().depth,
            1,
            "use encode_copy_from_at for 3d textures"
        );
        self.encode_copy_from_at(encoder, source, 0);
    }

    /// Encode a copy from a 3d texture at a given depth. Will copy a slice of the texture to fill
    /// this whole buffer (modulo padding).
    ///
    /// The copy will not be performed until the encoded command buffer is submitted.
    pub fn encode_copy_from_at(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        depth: u32,
    ) {
        let (destination, source, copy_size) = self.copy_views(source, depth);
        encoder.copy_texture_to_buffer(source, destination, copy_size);
    }

    /// Copy view logic.
    /// This is precisely the same for texture-to-buffer and buffer-to-texture copies.
    fn copy_views<'s, 't>(
        &'s self,
        texture: &'t wgpu::Texture,
        depth: u32,
    ) -> (
        wgpu::BufferCopyView<'s>,
        wgpu::TextureCopyView<'t>,
        wgpu::Extent3d,
    ) {
        let format_size_bytes = wgpu::texture_format_size_bytes(texture.format());

        assert_eq!(
            self.width % format_size_bytes,
            0,
            "buffer rows do not map evenly onto texture rows"
        );
        assert_eq!(
            texture.extent().width,
            self.width / format_size_bytes,
            "buffer rows are the wrong width"
        );
        assert_eq!(
            texture.extent().height,
            self.height,
            "buffer is the wrong height"
        );
        assert!(depth <= texture.extent().depth, "texture not deep enough");

        let mut copy_size = texture.extent();
        copy_size.depth = 1;

        let buffer_view = wgpu::BufferCopyView {
            buffer: &self.buffer,
            // note: this is the layout of *this buffer*.
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: self.padded_width(),
                rows_per_image: self.height,
            },
        };
        let texture_view = wgpu::TextureCopyView {
            texture,
            mip_level: 0, // TODO(jhg): should we handle this?
            origin: wgpu::Origin3d {
                x: 0,
                y: 0,
                z: depth,
            },
        };
        (buffer_view, texture_view, copy_size)
    }
}

/// A wrapper around a slice of bytes representing an image.
///
/// An `ImageReadMapping` may only be created via `RowPaddedBuffer::read()`.
pub struct ImageReadMapping<'buffer> {
    buffer: &'buffer wgpu::RowPaddedBuffer,
    view: wgpu::BufferView<'buffer>,
}

impl<'buffer> ImageReadMapping<'buffer> {
    /// View as an image::SubImage.
    ///
    /// Unsafe: `P::TEXTURE_FORMAT` MUST match the texture format / image type used to create the
    /// wrapped RowPaddedBuffer! If this is not the case, may result in undefined behavior!
    pub unsafe fn as_image<P>(&self) -> image::SubImage<ImageHolder<P>>
    where
        P: Pixel + 'static,
    {
        let subpixel_size = std::mem::size_of::<P::Subpixel>() as u32;
        let pixel_size = subpixel_size * P::CHANNEL_COUNT as u32;
        assert_eq!(pixel_size, P::COLOR_TYPE.bytes_per_pixel() as u32);

        assert_eq!(
            self.buffer.padded_width() % pixel_size,
            0,
            "buffer padded width not an even multiple of primitive size"
        );
        assert_eq!(
            self.buffer.width() % pixel_size,
            0,
            "buffer row width not an even multiple of primitive size"
        );

        let width_pixels = self.buffer.width() / pixel_size;
        let padded_width_pixels = self.buffer.padded_width() / pixel_size;

        // ways this cast could go wrong:
        // - buffer is the wrong size: checked in to_slice, panics
        // - buffer is the wrong alignment: checked in to_slice, panics
        // - buffer rows are the wrong size: checked above, panics
        // - buffer has not been initialized / has invalid data for primitive type:
        //   very possible. That's why this function is `unsafe`.
        let container = wgpu::bytes::to_slice::<P::Subpixel>(&self.view[..]);

        let full_image =
            image::ImageBuffer::from_raw(padded_width_pixels, self.buffer.height(), container)
                .expect("nannou internal error: incorrect buffer size");
        image::SubImage::new(
            ImageHolder(full_image),
            0,
            0,
            width_pixels,
            self.buffer.height(),
        )
    }
}

/// Workaround for the fact that `image::SubImage` requires a `Deref` impl on the wrapped image.
pub struct ImageHolder<'b, P: Pixel>(image::ImageBuffer<P, &'b [P::Subpixel]>);
impl<'b, P: Pixel> Deref for ImageHolder<'b, P> {
    type Target = image::ImageBuffer<P, &'b [P::Subpixel]>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn row_alignment_subsumes_buffer_alignment() {
        assert_eq!(
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64 % wgpu::COPY_BUFFER_ALIGNMENT,
            0,
            "BufferImage allocation logic is broken!"
        );
    }
}
