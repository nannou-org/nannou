# Receiving OSC

**Tutorial Info**

- Author: mitchmindtree
- Required Knowledge:
    - [Anatomy of a nannou App](/tutorials/basics/anatomy-of-a-nannou-app.md)
    - [OSC introduction](/tutorials/osc/osc-introduction.md)
    - [Sending OSC](/tutorials/osc/osc-sender.md)
- Reading Time: 15 minutes

---

In the [previous tutorial](/tutorials/osc/osc-sender.md) we sent the position of
a circle out over OSC. In this tutorial we will do the opposite: we will *receive*
OSC packets from another application and display them in a window.

We will again use the `nannou_osc` crate. If you have not already added it to your
project, see the [OSC introduction](/tutorials/osc/osc-introduction.md).

```rust,no_run
# #![allow(unused_imports)]
use nannou_osc as osc;
# fn main() {}
```

## Setting up an OSC receiver

Where the sender stored an `osc::Sender` in its `Model`, the receiver stores an
[`osc::Receiver`](https://docs.rs/nannou_osc/latest/nannou_osc/recv/struct.Receiver.html).
We will also keep a list of the packets we have received so that we can draw them.
Each received packet comes paired with the network address it was sent from, so we
store both.

```rust,no_run
# #![allow(dead_code, unused_imports)]
# use nannou_osc as osc;
struct Model {
    receiver: osc::Receiver,
    received_packets: Vec<(std::net::SocketAddr, osc::Packet)>,
}
# fn main() {}
```

Next, we set up our `Model` in the `model` function. An `osc::Receiver` is bound
to a network *port* - the same port that the sending application is targeting.
Make sure this matches the port used by your sender (in the
[sending tutorial](/tutorials/osc/osc-sender.md) we used `1234`, here we use
`34254` to match nannou's `osc_sender.rs` example - pick whichever you like, as
long as the sender and receiver agree).

```rust,no_run
# #![allow(dead_code, unused_imports)]
# use nannou_osc as osc;
# use nannou::prelude::*;
# struct Model {
#     receiver: osc::Receiver,
#     received_packets: Vec<(std::net::SocketAddr, osc::Packet)>,
# }
// The port on which we will listen for OSC packets.
const PORT: u16 = 34254;

fn model(_app: &App) -> Model {
    // Bind an `osc::Receiver` to the port.
    let receiver = osc::receiver(PORT).unwrap();

    // A vec for collecting packets and their source address.
    let received_packets = vec![];

    Model {
        receiver,
        received_packets,
    }
}
# fn main() {}
```

Binding can fail (for example if another program is already using the port), so
`osc::receiver` returns a `Result`. Here we simply `unwrap()` it, which will exit
the program with an error message if binding was unsuccessful.

## Receiving OSC messages

The receiver collects incoming packets in the background. To retrieve them, we
poll the receiver each frame from an `update` function. The
[`try_iter`](https://docs.rs/nannou_osc/latest/nannou_osc/recv/struct.Receiver.html#method.try_iter)
method returns an iterator over all packets that have arrived since we last
checked, *without* blocking if there are none. Each item is a tuple of the
`osc::Packet` and the `SocketAddr` it came from.

```rust,no_run
# #![allow(dead_code, unused_imports)]
# use nannou_osc as osc;
# use nannou::prelude::*;
# struct Model {
#     receiver: osc::Receiver,
#     received_packets: Vec<(std::net::SocketAddr, osc::Packet)>,
# }
fn update(_app: &App, model: &mut Model) {
    // Receive any pending OSC packets.
    for (packet, addr) in model.receiver.try_iter() {
        model.received_packets.push((addr, packet));
    }

    // We'll display 10 packets at a time, so remove any excess.
    while model.received_packets.len() > 10 {
        model.received_packets.remove(0);
    }
}
# fn main() {}
```

> **Note:** If you would rather *wait* for a packet to arrive than continue
> without one, you can use the blocking
> [`recv`](https://docs.rs/nannou_osc/latest/nannou_osc/recv/struct.Receiver.html#method.recv)
> method instead. In a nannou app we usually prefer the non-blocking `try_iter`
> so that drawing and other events are not held up.

## The finished app

Finally, we draw the packets we have received as text in the window. Putting it
all together:

```rust,no_run
use nannou::prelude::*;
use nannou_osc as osc;

// Match this to the port used by the sender.
const PORT: u16 = 34254;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    receiver: osc::Receiver,
    received_packets: Vec<(std::net::SocketAddr, osc::Packet)>,
}

fn model(_app: &App) -> Model {
    let receiver = osc::receiver(PORT).unwrap();
    let received_packets = vec![];
    Model {
        receiver,
        received_packets,
    }
}

fn update(_app: &App, model: &mut Model) {
    // Receive any pending OSC packets.
    for (packet, addr) in model.receiver.try_iter() {
        model.received_packets.push((addr, packet));
    }

    // We'll display 10 packets at a time, so remove any excess.
    while model.received_packets.len() > 10 {
        model.received_packets.remove(0);
    }
}

fn view(app: &App, model: &Model, _window: Entity) {
    let draw = app.draw();
    draw.background().color(DARK_BLUE);

    // Create a string showing all the packets received so far.
    let mut packets_text = format!("Listening on port {}\nReceived packets:\n", PORT);
    for &(addr, ref packet) in model.received_packets.iter().rev() {
        packets_text.push_str(&format!("{}: {:?}\n", addr, packet));
    }

    let rect = app.window_rect().pad(10.0);

    draw.text(&packets_text)
        .font_size(16)
        .align_text_top()
        .left_justify()
        .wh(rect.wh());
}
```

Run this app alongside the sender from the previous tutorial (or nannou's
`osc_sender.rs` example) and you should see the OSC messages appear in the window
as they arrive.

You can find a complete, runnable version of this example as
[`osc_receiver.rs`](https://github.com/nannou-org/nannou/blob/master/examples/communication/osc_receiver.rs)
in the nannou repository.
