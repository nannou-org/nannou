use nannou::prelude::*;
use nannou_osc as osc;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    receiver: osc::Receiver,
    received_packets: Vec<(std::net::SocketAddr, osc::Packet)>,
}

// Make sure this matches the `TARGET_PORT` in the `osc_sender.rs` example.
const PORT: u16 = 34254;

fn model(app: &App) -> Model {
    let _w_id = app
        .new_window()
        .title("OSC Receiver")
        .size(1400, 480)
        .view(view)
        .build()
        .unwrap();

    // Bind an `osc::Receiver` to a port.
    let receiver = osc::receiver(PORT).unwrap();

    // A vec for collecting packets and their source address.
    let received_packets = vec![];

    Model {
        receiver,
        received_packets,
    }
}

fn update(_app: &App, model: &mut Model) {
    // Receive any pending osc packets.
    for (packet, addr) in model.receiver.try_iter() {
        model.received_packets.push((addr, packet));
    }

    // We'll display 10 packets at a time, so remove any excess.
    let max_packets = 10;
    while model.received_packets.len() > max_packets {
        model.received_packets.remove(0);
    }
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(DARKBLUE);

    // Create a string showing all the packets.
    let mut packets_text = format!("Listening on port {}\nReceived packets:\n", PORT);
    for &(addr, ref packet) in model.received_packets.iter().rev() {
        packets_text.push_str(&format!("{}: {:?}\n", addr, packet));
    }
    let rect = frame.rect().pad(10.0);
    draw.text(&packets_text)
        .font_size(16)
        .align_text_top()
        .line_spacing(10.0)
        .left_justify()
        .wh(rect.wh());


}
