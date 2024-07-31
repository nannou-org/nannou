//! Tools for working with OSC. [**sender()**](./fn.sender.html) creates an OSC sender,
//! [**receiver(port)**](./fn.receiver.html) creates an OSC receiver.

use std::net::{Ipv4Addr, SocketAddr};

pub use rosc;

// Re-export rosc items.
//
// Remove `Osc` prefix as items are already namespaced via a module, e.g. `OscMessage` becomes
// `nannou_osc::Message`.
pub use self::recv::Receiver;
#[doc(inline)]
pub use self::rosc::{
    address, decoder, encoder, OscArray as Array, OscBundle as Bundle, OscColor as Color,
    OscError as Error, OscMessage as Message, OscMidiMessage as MidiMessage, OscTime as Time,
    OscTimeError as TimeError, OscType as Type,
};
pub use self::send::Sender;

pub mod recv;
pub mod send;

/// Indicates that a `Sender` is not currently connected to a target address, and that the target
/// address will have to be supplied manually when sending packets.
pub struct Unconnected;

/// Indicates that a `Sender` is currently connected to a specific target address.
pub struct Connected {
    addr: SocketAddr,
}

/// An *OSC packet* can contain an OSC message or a bundle of nested packets which is called an
/// *OSC bundle*.
#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    Message(Message),
    Bundle(Bundle),
}

impl From<Message> for Packet {
    fn from(msg: Message) -> Self {
        Packet::Message(msg)
    }
}

impl From<Bundle> for Packet {
    fn from(bundle: Bundle) -> Self {
        Packet::Bundle(bundle)
    }
}

impl From<rosc::OscPacket> for Packet {
    fn from(packet: rosc::OscPacket) -> Self {
        match packet {
            rosc::OscPacket::Message(msg) => msg.into(),
            rosc::OscPacket::Bundle(bundle) => bundle.into(),
        }
    }
}

impl<A> From<(A, Vec<Type>)> for Packet
where
    A: Into<String>,
{
    fn from((addr, args): (A, Vec<Type>)) -> Self {
        msg(addr, args).into()
    }
}

impl From<Packet> for rosc::OscPacket {
    fn from(val: Packet) -> Self {
        match val {
            Packet::Message(msg) => rosc::OscPacket::Message(msg),
            Packet::Bundle(bundle) => rosc::OscPacket::Bundle(bundle),
        }
    }
}

impl Packet {
    /// A recursive function that unfolds the packet into the end of the given buffer of messages.
    pub fn unfold(self, msgs: &mut Vec<Message>) {
        match self {
            Packet::Message(msg) => msgs.push(msg),
            Packet::Bundle(bundle) => {
                for packet in bundle.content {
                    Packet::unfold(packet.into(), msgs);
                }
            }
        }
    }

    /// Convert the `Packet` into a Vec containing all `Message`s contained within.
    ///
    /// This uses the `unfold` method internally.
    pub fn into_msgs(self) -> Vec<Message> {
        let mut msgs = vec![];
        self.unfold(&mut msgs);
        msgs
    }
}

/// The default local IP address.
pub fn default_ipv4_addr() -> Ipv4Addr {
    Ipv4Addr::new(0, 0, 0, 0)
}

/// A simple wrapper around the most commonly used `Receiver` constructor.
pub fn receiver(port: u16) -> Result<Receiver, std::io::Error> {
    Receiver::bind(port)
}

/// A simple wrapper around the most commonly used `Sender` constructor.
pub fn sender() -> Result<Sender, std::io::Error> {
    Sender::bind()
}

/// A simplified constructor for an OSC `Message`.
pub fn msg<A>(addr: A, args: Vec<Type>) -> Message
where
    A: Into<String>,
{
    let addr = addr.into();
    Message { addr, args }
}

/// Decodes the given slice of `bytes` into a `Packet`.
///
/// Returns an `Error` if the slice does not contain a valid OSC packet.
pub fn decode(bytes: &[u8]) -> Result<Packet, Error> {
    rosc::decoder::decode_udp(bytes).map(|(_bytes, p)| p.into())
}

/// Encodes the given `Packet` into a `Vec` of bytes.
///
/// Returns an `Error` if the packet is invalid.
pub fn encode(packet: Packet) -> Result<Vec<u8>, Error> {
    let rosc_packet = packet.into();
    rosc::encoder::encode(&rosc_packet)
}

/// Errors that might occur whilst attempting to send or receive an OSC packet.
#[derive(Debug)]
pub enum CommunicationError {
    Io(std::io::Error),
    Osc(Error),
    Poisoned,
}

impl From<std::io::Error> for CommunicationError {
    fn from(err: std::io::Error) -> Self {
        CommunicationError::Io(err)
    }
}

impl From<Error> for CommunicationError {
    fn from(err: Error) -> Self {
        CommunicationError::Osc(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for CommunicationError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        CommunicationError::Poisoned
    }
}

impl std::error::Error for CommunicationError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            CommunicationError::Io(ref err) => Some(err),
            // TODO: Error isn't implemented for OscError - should fix this upstream.
            CommunicationError::Osc(ref _err) => None,
            // CommunicationError::Osc(ref err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CommunicationError::Io(ref err) => std::fmt::Display::fmt(err, f),
            // TODO: Error isn't implemented for OscError - should fix this upstream.
            CommunicationError::Osc(ref _err) => write!(f, "Failed to decode the OSC packet"),
            CommunicationError::Poisoned => write!(f, "The inner buffer's mutex was poisoned"),
        }
    }
}
