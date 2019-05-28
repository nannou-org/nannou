use nannou::osc;
use nannou::prelude::*;
use nannou::ui::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    receiver: osc::Receiver,
    received_packets: Vec<(std::net::SocketAddr, osc::Packet)>,
    ui: Ui,
    text: widget::Id,
}

// Make sure this matches the `TARGET_PORT` in the `osc_sender.rs` example.
const PORT: u16 = 34254;

fn model(app: &App) -> Model {
    app.new_window()
        .with_title("OSC Receiver")
        .with_dimensions(1400, 480)
        .view(view)
        .build()
        .unwrap();

    // Bind an `osc::Receiver` to a port.
    let receiver = osc::receiver(PORT).unwrap();

    // A vec for collecting packets and their source address.
    let received_packets = vec![];

    // Create a simple UI to display received messages.
    let mut ui = app.new_ui().build().unwrap();
    let text = ui.generate_widget_id();

    Model {
        receiver,
        received_packets,
        ui,
        text,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Receive any pending osc packets.
    for (packet, addr) in model.receiver.try_iter() {
        model.received_packets.push((addr, packet));
    }

    // We'll display 10 packets at a time, so remove any excess.
    let max_packets = 10;
    while model.received_packets.len() > max_packets {
        model.received_packets.remove(0);
    }

    // Create a string showing all the packets.
    let mut packets_text = format!("Listening on port {}\nReceived packets:\n", PORT);
    for &(addr, ref packet) in model.received_packets.iter().rev() {
        packets_text.push_str(&format!("{}: {:?}\n", addr, packet));
    }

    // Use the UI to display the packet string.
    model.ui.clear_with(color::DARK_BLUE);
    let mut ui = model.ui.set_widgets();
    widget::Text::new(&packets_text)
        .top_left_with_margin_on(ui.window, 20.0)
        .color(color::WHITE)
        .line_spacing(10.0)
        .set(model.text, &mut ui);
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    model.ui.draw_to_frame(app, &frame).unwrap();
    frame
}
