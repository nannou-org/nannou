#[macro_use] extern crate glium;
extern crate nannou;

use nannou::prelude::*;

fn main() {
    nannou::run(model, event, view);
}

struct Model {
    window: WindowId,
    time: f32,
}

fn model(app: &App) -> Model {
    let window = app.new_window().with_vsync(true).with_dimensions(640,480).build().unwrap();
    let time = 0.0;

    Model { window, time }
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    match event {
        Event::WindowEvent { simple: Some(event), .. } => match event {

            Moved(_pos) => {
            },

            KeyPressed(_key) => {
            },

            KeyReleased(_key) => {
            },

            MouseMoved(_pos) => {
            },

            MouseDragged(_pos, _button) => {
            },

            MousePressed(_button) => {
            },

            MouseReleased(_button) => {
            },

            MouseEntered => {
            },

            MouseExited => {
            },

            Resized(_size) => {
            },

            _other => (),
        },

        Event::Update(_dt) => {
            model.time += 1.0;
        },

        _ => (),
    }
    model
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Our app only has one window, so retrieve this part of the `Frame`. Color it gray.
    frame.window(model.window).unwrap().clear_color(0.1, 0.11, 0.12, 1.0);

    let rect = nannou::geom::Rect::from_xy_wh(Point2 { x: 0.0, y: 0.0 }, Vector2 {x: 2.0, y: 2.0 });
    // Get the 2 triangles to form a rectangle 
    let (tri_a, tri_b) = rect.triangles();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    implement_vertex!(Vertex, position);

    let shape: Vec<Vertex> = tri_a.iter()
        .chain(tri_b.iter())
        .map(|p| Vertex { position: [p.x as f32, p.y as f32] })
        .collect();

    let win = app.window(model.window).unwrap();
    let display = win.inner_glium_display();
    let vertex_buffer = nannou::glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = nannou::glium::index::NoIndices(nannou::glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 330

        in vec2 position;
        out vec2 uv;

        void main() {
            uv = position;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 330

        out vec4 color;
        in vec2 uv;

        uniform float iTime;

        void main() {
            color = vec4(abs(sin(abs(uv.x)+iTime*0.3)), cos(abs(uv.y)+iTime*0.05), sin(iTime*0.1), 1.0);
        }
    "#;


    let program = nannou::glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

    frame.window(model.window).unwrap().draw(&vertex_buffer, &indices, &program, &uniform! { iTime: model.time },
            &Default::default()).unwrap();
    // Return the drawn frame.
    frame
}
