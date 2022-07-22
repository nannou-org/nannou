use nannou::image;
use nannou::math;
use nannou::prelude::*;
use resvg;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::f32::consts;
use std::fs;
use std::path::PathBuf;
use tiny_skia;
use usvg;
fn main() {
    nannou::app(model).run();
}

#[derive(Copy, Clone)]
enum Resolution {
    LOW,
    MED,
    HIG,
}
type Size = Resolution;

impl TryFrom<u8> for Resolution {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::LOW),
            1 => Ok(Self::MED),
            2 => Ok(Self::HIG),
            _ => Err("Out of Bounds"),
        }
    }
}
struct Model {
    svg_texture: wgpu::Texture,
    resolution: Resolution,
    size: Size,
    scale: f32,
    rotation_offset: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1920, 1080)
        .view(view)
        .key_pressed(keypressed)
        .build()
        .unwrap();

    Model {
        svg_texture: generate_texture(app, "module_1.svg").unwrap(),
        resolution: Resolution::MED,
        size: Size::MED,
        scale: 0.0,
        rotation_offset: 0.0,
    }
}

fn dist_points(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    return ((p2.0 - p1.0).pow(2u8) + (p2.1 - p1.0).pow(2u8)).sqrt();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let w_rect = app.main_window().rect();
    let right = w_rect.right();
    let left = w_rect.left();
    let top = w_rect.top();
    let down = w_rect.left();
    draw.background().color(WHITE);
    let n_tiles: (u8, u8);
    match model.resolution {
        Resolution::MED => n_tiles = (20, 20),
        Resolution::HIG => n_tiles = (50, 50),
        Resolution::LOW => n_tiles = (10, 10),
    }

    let _ = (0..n_tiles.0)
        .into_iter()
        .map(|x| {
            _ = (0..n_tiles.1)
                .into_iter()
                .map(|y| {
                    let loc_scale: f32;
                    let x_pos = math::map_range(x, 0, n_tiles.0, left, right);
                    let y_pos = math::map_range(y, 0, n_tiles.1, top, down);
                    match model.size {
                        Size::MED => loc_scale = 1.0,
                        Size::LOW => {
                            loc_scale = math::map_range(
                                dist_points((app.mouse.x, app.mouse.y), (x_pos, y_pos)),
                                0.0,
                                1024.0,
                                0.2,
                                1.0,
                            )
                        }
                        Size::HIG => {
                            loc_scale = math::map_range(
                                dist_points((app.mouse.x, app.mouse.y), (x_pos, y_pos)),
                                0.0,
                                1024.0,
                                1.0,
                                0.2,
                            )
                        }
                    }
                    let angle = (app.mouse.y - y_pos).atan2(app.mouse.x - x_pos);
                    let size = model.svg_texture.extent();
                    let size_scale = 1.0 + model.scale;
                    let rotation_offset = 2.0 * consts::PI * (model.rotation_offset);
                    draw.texture(&model.svg_texture)
                        .x_y(x_pos, y_pos)
                        .w_h(
                            loc_scale * size_scale * size.width as f32,
                            loc_scale * size_scale * size.height as f32,
                        )
                        .rotate(angle + rotation_offset);
                })
                .collect::<Vec<_>>();
        })
        .collect::<Vec<_>>();
    draw.to_frame(app, &frame).unwrap();
}

fn generate_texture(app: &App, svg_name: &str) -> Result<wgpu::Texture, Box<dyn Error>> {
    let assets_path = app.assets_path()?.join("svg/generative_examples/");
    let svg = fs::read_to_string(&assets_path.join(svg_name))?;

    let svg_tree = usvg::Tree::from_str(&svg, &usvg::Options::default().to_ref())?;

    let mut p_m = tiny_skia::Pixmap::new(
        svg_tree.svg_node().size.width() as u32,
        svg_tree.svg_node().size.height() as u32,
    )
    .unwrap();

    resvg::render(
        &svg_tree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        p_m.as_mut(),
    );

    let svg_img =
        image::load_from_memory_with_format(&p_m.encode_png().unwrap(), image::ImageFormat::Png)?;
    Ok(wgpu::Texture::from_image(app, &svg_img))
}

fn keypressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => model.svg_texture = generate_texture(app, "module_1.svg").unwrap(),
        Key::Key2 => model.svg_texture = generate_texture(app, "module_2.svg").unwrap(),
        Key::Key3 => model.svg_texture = generate_texture(app, "module_3.svg").unwrap(),
        Key::Key4 => model.svg_texture = generate_texture(app, "module_4.svg").unwrap(),
        Key::Key5 => model.svg_texture = generate_texture(app, "module_5.svg").unwrap(),
        Key::Key6 => model.svg_texture = generate_texture(app, "module_6.svg").unwrap(),
        Key::Key7 => model.svg_texture = generate_texture(app, "module_7.svg").unwrap(),
        Key::Up => model.scale += 0.1,
        Key::Down => model.scale -= 0.1,
        Key::Left => model.rotation_offset += 0.01,
        Key::Right => model.rotation_offset -= 0.01,
        Key::D => model.size = ((model.size as u8 + 1) % 3).try_into().unwrap(),
        Key::G => model.resolution = ((model.resolution as u8 + 1) % 3).try_into().unwrap(),
        _ => (),
    }
}
