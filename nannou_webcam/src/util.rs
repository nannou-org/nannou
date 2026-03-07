use bevy::image::Image;
use bevy::render::render_resource::{Extent3d, TextureFormat};

pub fn frame_texture_format(is_srgb: bool) -> TextureFormat {
    if is_srgb {
        TextureFormat::Rgba8UnormSrgb
    } else {
        TextureFormat::Rgba8Unorm
    }
}

pub fn write_frame_to_image(
    image: &mut Image,
    extent: Extent3d,
    pixels: Vec<u8>,
    format: TextureFormat,
) {
    if image.texture_descriptor.size != extent {
        image.resize(extent);
    }
    if image.texture_descriptor.format != format {
        image.texture_descriptor.format = format;
    }
    image.data = Some(pixels);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn rgb_to_rgba(rgb: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(rgb.len() / 3 * 4);
    for chunk in rgb.chunks_exact(3) {
        rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 0xff]);
    }
    rgba
}
