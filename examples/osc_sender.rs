use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou_osc as osc;
use nannou_osc::Type;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    sender: osc::Sender<osc::Connected>,
    ui: Ui,
    text: widget::Id,
}

// Make sure this matches `PORT` in the `osc_receiver.rs` example.
const TARGET_PORT: u16 = 34254;

fn target_address_string() -> String {
    format!("{}:{}", "127.0.0.1", TARGET_PORT)
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("OSC Sender")
        .dimensions(680, 480)
        .event(event)
        .view(view)
        .build()
        .unwrap();

    // The address to which the `Sender` will send messages.
    let target_addr = target_address_string();

    // Bind an `osc::Sender` and connect it to the target address.
    let sender = osc::sender().unwrap().connect(target_addr).unwrap();

    // Create a simple UI to tell the user what to do.
    let mut ui = app.new_ui().build().unwrap();
    let text = ui.generate_widget_id();

    Model { sender, ui, text }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        MouseMoved(pos) => {
            let addr = "/example/mouse_moved/";
            let args = vec![Type::Float(pos.x), Type::Float(pos.y)];
            model.sender.send((addr, args)).ok();
        }

        MousePressed(button) => {
            let addr = "/example/mouse_pressed/";
            let button = format!("{:?}", button);
            let args = vec![Type::String(button)];
            model.sender.send((addr, args)).ok();
        }

        MouseReleased(button) => {
            let addr = "/example/mouse_released/";
            let button = format!("{:?}", button);
            let args = vec![Type::String(button)];
            model.sender.send((addr, args)).ok();
        }

        _other => (),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Use the UI to show the user where packets are being sent.
    model.ui.clear_with(color::DARK_RED);
    let mut ui = model.ui.set_widgets();
    let text = format!(
        "Move or click the mouse to send\nmessages to the \
         receiver example!\n\nSending OSC packets to {}",
        target_address_string()
    );
    widget::Text::new(&text)
        .middle_of(ui.window)
        .center_justify()
        .color(color::WHITE)
        .line_spacing(10.0)
        .set(model.text, &mut ui);
}

fn view(app: &App, model: &Model, frame: &Frame) {
    model.ui.draw_to_frame(app, &frame).unwrap();
}
