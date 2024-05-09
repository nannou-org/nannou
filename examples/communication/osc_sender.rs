use nannou::prelude::*;
use nannou_osc as osc;
use nannou_osc::Type;

fn main() {
    nannou::app(model).run();
}

struct Model {
    sender: osc::Sender<osc::Connected>,
}

// Make sure this matches `PORT` in the `osc_receiver.rs` example.
const TARGET_PORT: u16 = 34254;

fn target_address_string() -> String {
    format!("{}:{}", "127.0.0.1", TARGET_PORT)
}

fn model(app: &App) -> Model {
    let _w_id = app
        .new_window()
        .title("OSC Sender")
        .size(680, 480)
        .event(event)
        .view(view)
        .build();

    // The address to which the `Sender` will send messages.
    let target_addr = target_address_string();

    // Bind an `osc::Sender` and connect it to the target address.
    let sender = osc::sender().unwrap().connect(target_addr).unwrap();

    Model { sender }
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

fn view(app: &App, _model: &Model) {
    let draw = app.draw();
    draw.background().color(DARKRED);

    let text = format!(
        "Move or click the mouse to send\nmessages to the \
         receiver example!\n\nSending OSC packets to {}",
        target_address_string()
    );
    let rect = frame.rect();
    draw.text(&text)
        .font_size(16)
        .line_spacing(10.0)
        .wh(rect.wh());


}
