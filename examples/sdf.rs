#[macro_use]
extern crate glium;
extern crate nannou;

use nannou::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    window: WindowId,
    time: f32,
}

fn model(app: &App) -> Model {
    let window = app.new_window()
        .with_vsync(true)
        .with_dimensions(640, 480)
        .with_gl_debug_flag(true)
        .build()
        .unwrap();
    println!(
        "gl version = {:?}",
        app.window(window).unwrap().opengl_version()
    );
    let time = 0.0;

    Model { window, time }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        } => match event {
            Moved(_pos) => {}

            KeyPressed(_key) => {}

            KeyReleased(_key) => {}

            MouseMoved(_pos) => {}

            MouseDragged(_pos, _button) => {}

            MousePressed(_button) => {}

            MouseReleased(_button) => {}

            MouseEntered => {}

            MouseExited => {}

            Resized(_size) => {}

            _other => (),
        },

        Event::Update(_dt) => {
            model.time += 1.0;
        }

        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it gray.
    frame
        .window(model.window)
        .unwrap()
        .clear_color(0.1, 0.11, 0.12, 1.0);

    let rect =
        nannou::geom::Rect::from_xy_wh(Point2 { x: 0.0, y: 0.0 }, Vector2 { x: 2.0, y: 2.0 });
    // Get the 2 triangles to form a rectangle
    let (tri_a, tri_b) = rect.triangles();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 4],
        texcoord: [f32; 2],
    }
    implement_vertex!(Vertex, position, texcoord);

    let shape: Vec<Vertex> = tri_a
        .iter()
        .chain(tri_b.iter())
        .map(|p| Vertex {
            position: [p.x as f32, p.y as f32, 0.0, 1.0],
            texcoord: [0.0; 2],
        })
        .collect();

    let win = app.window(model.window).unwrap();
    let display = win.inner_glium_display();
    let vertex_buffer = nannou::glium::VertexBuffer::new(display, &shape).unwrap();
    let indices =
        nannou::glium::index::NoIndices(nannou::glium::index::PrimitiveType::TrianglesList);

    let path = app.assets_path().unwrap();
    let sdf_path = path.join("glsl/sdf_playground");
    let frag_path = sdf_path.join("main.frag");
    let vert_path = sdf_path.join("passthrough.vert");

    let frag_file = File::open(frag_path).unwrap();
    let vert_file = File::open(vert_path).unwrap();

    let mut frag_buf_reader = BufReader::new(frag_file);
    let mut vert_buf_reader = BufReader::new(vert_file);

    let mut frag_string = String::new();
    let mut vert_string = String::new();

    frag_buf_reader.read_to_string(&mut frag_string).unwrap();
    vert_buf_reader.read_to_string(&mut vert_string).unwrap();

    let program =
        nannou::glium::Program::from_source(display, &vert_string, &frag_string, None).unwrap();

    let uniforms = uniform! {
        iTime: model.time,
        iResolution: [app.window.width as f32, app.window.height as f32, 0.0]
    };

    frame
        .window(model.window)
        .unwrap()
        .draw(
            &vertex_buffer,
            &indices,
            &program,
            &uniforms,
            &Default::default(),
        )
        .unwrap();
    // Return the drawn frame.
    frame
}
