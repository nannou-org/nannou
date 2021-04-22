use nannou::prelude::*;
use nannou_ui as ui;
use nannou_ui::{widget, Ui};

fn main() {
    nannou::app(model).event(event).update(update).run();
}

struct Model {
    ui: Ui,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();

    let window = app.main_window();

    let ui = create_ui(&window);

    Model { ui }
}

fn create_ui(window: &Window) -> Ui {
    // Initialise the UI.
    let mut ui = Ui::default();

    // The container for this window.
    let size = window.rect().wh();
    let container = widget::Container::from(size);
    let win = ui.add_child(ui.root(), container);

    // Add a mesh for this window. In this case, a coloured square.
    let mut mesh = widget::Mesh::default();
    let to_vertex = |p: Point2, c: Rgb8| {
        let point = p.into();
        let color = rgb8_to_lin_srgba(c);
        widget::mesh::vertex::colored(point, color)
    };
    let vertices = [
        (pt2(-1.0, -1.0), RED),
        (pt2(1.0, -1.0), GREEN),
        (pt2(1.0, 1.0), BLUE),
        (pt2(-1.0, 1.0), YELLOW),
    ];
    let indices = [0, 1, 2, 0, 2, 3];
    mesh.extend_vertices(vertices.iter().map(|&(p, c)| to_vertex(p, c)));
    mesh.extend_indices(indices.iter().cloned());
    ui.add_child(win, mesh);

    // Create some widgets.

    ui
}

fn event(_app: &App, model: &mut Model, event: Event) {
    model.ui.process_event(event);
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    model.ui.draw_to_frame(&frame).unwrap();
}

fn rgb8_to_lin_srgba(c: Rgb8) -> LinSrgba {
    let conv = |u: u8| u as f32 / std::u8::MAX as f32;
    lin_srgba(conv(c.red), conv(c.green), conv(c.blue), 1.0)
}
