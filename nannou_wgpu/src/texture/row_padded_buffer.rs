use crate as wgpu;

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
    pub(crate) buffer: wgpu::Buffer,
    /// The descriptor used to create the wrapped buffer.
    buffer_descriptor: wgpu::BufferDescriptor<'static>,
}

impl RowPaddedBuffer {
    /// Create a row-padded buffer on the device.
    ///
    /// Width should be given in bytes.
    pub fn new(device: &wgpu::Device, width: u32, height: u32, usage: wgpu::BufferUsages) -> Self {
        let row_padding = wgpu::compute_row_padding(width);

        // only create mapped for buffers that we're going to write to.
        let mapped_at_creation = usage.contains(wgpu::BufferUsages::MAP_WRITE);

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("nannou::RowPaddedBuffer"),
            size: ((width + row_padding) * height) as u64,
            usage,
            mapped_at_creation,
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        Self {
            width,
            row_padding,
            height,
            buffer,
            buffer_descriptor,
        }
    }

    /// Creates a buffer compatible with a 2d slice of the given texture.
    pub fn for_texture(
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        usage: wgpu::BufferUsages,
    ) -> RowPaddedBuffer {
        Self::new(
            device,
            texture.extent().width * wgpu::texture_format_size_bytes(texture.format()),
            texture.extent().height,
            usage,
        )
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
    /// The buffer usage must include `BufferUsages::map_read()`.
    pub fn write(&self, buf: &[u8]) {
        assert_eq!(
            (self.width * self.height) as usize,
            buf.len(),
            "Incorrect input slice size"
        );
        assert!(
            self.buffer_descriptor
                .usage
                .contains(wgpu::BufferUsages::MAP_WRITE),
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

    /// Encode a copy into a texture. Assumes the texture is 2d.
    ///
    /// The copy will not be performed until the encoded command buffer is submitted.
    pub fn encode_copy_into(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::Texture,
    ) {
        assert_eq!(
            destination.extent().depth_or_array_layers,
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
            source.extent().depth_or_array_layers,
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
        wgpu::ImageCopyBuffer<'s>,
        wgpu::ImageCopyTexture<'t>,
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
        assert!(
            depth <= texture.extent().depth_or_array_layers,
            "texture not deep enough"
        );

        let mut copy_size = texture.extent();
        copy_size.depth_or_array_layers = 1;

        let buffer_view = wgpu::ImageCopyBuffer {
            buffer: &self.buffer,
            // note: this is the layout of *this buffer*.
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.padded_width()),
                rows_per_image: Some(self.height),
            },
        };
        let texture_view = texture.as_image_copy();
        (buffer_view, texture_view, copy_size)
    }
}
