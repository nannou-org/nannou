//! Items related to the `osc::Sender` implementation.

use super::{encode, CommunicationError, Connected, Packet, Unconnected};
use std;
use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};

/// The default port bound to by the `Sender`.
///
/// In most cases, mono-directional networking tools will bind to port `0` for outgoing
/// connections, which means that the OS assigns a freely available random outgoing port.
pub const DEFAULT_PORT: u16 = 0;

/// A type used for sending OSC packets.
pub struct Sender<M = Unconnected> {
    socket: UdpSocket,
    mode: M,
}

/// The default socket address bound to by the `Sender`.
///
/// This address is the `default_ipv4_addr` with the `DEFAULT_PORT`.
pub fn default_sender_socket_addr_v4() -> SocketAddrV4 {
    SocketAddrV4::new(super::default_ipv4_addr(), DEFAULT_PORT)
}

impl<M> Sender<M> {
    /// The socket address that this `Sender`'s socket was created from.
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.socket.local_addr()
    }
}

impl Sender<Unconnected> {
    /// Creates a new `Sender` with the UDP socket bound to the given address. **Note: this is
    /// not the target address**, rather it is the address of the socket used by the `Sender` to
    /// send packets. Use the `bind` constructor instead if you would like the `Sender`'s socket
    /// bound to the default address.
    ///
    /// ```
    /// extern crate nannou;
    ///
    /// use nannou::osc::Sender;
    ///
    /// fn main() {
    ///     let tx = Sender::bind_to("0.0.0.0:0").expect("Couldn't bind socket to the address");
    /// }
    /// ```
    ///
    /// The returned `Sender` is `Unconnected`, meaning that it is not currently connected to a
    /// specific target address. This means that the target address will have to be specified when
    /// calling `send`. To connect the `Sender` to a specific target address, use the `connect`
    /// builder method which will return a `Sender` in the `Connected` mode.
    pub fn bind_to<A>(addr: A) -> Result<Self, std::io::Error>
    where
        A: ToSocketAddrs,
    {
        let socket = UdpSocket::bind(addr)?;
        let mode = Unconnected;
        let sender = Sender { socket, mode };
        Ok(sender)
    }

    /// The same as `bind_to` but assumes the `default_sender_socket_addr_v4` socket address.
    pub fn bind() -> Result<Self, std::io::Error> {
        Self::bind_to(default_sender_socket_addr_v4())
    }

    /// Connects the `Sender`'s UDP socket to the given target, remote address.
    ///
    /// The returned `Sender` will only send packets to the specified address.
    ///
    /// Returns an error if some IO error occurred.
    ///
    /// **Panic!**s if the given `addr` cannot resolve to a valid `SocketAddr`.
    ///
    /// ```
    /// extern crate nannou;
    ///
    /// use nannou::osc::Sender;
    ///
    /// fn main() {
    ///     let tx = Sender::bind()
    ///         .expect("Couldn't bind to default socket")
    ///         .connect("127.0.0.1:34254")
    ///         .expect("Couldn't connect to socket at address");
    /// }
    /// ```
    pub fn connect<A>(self, addr: A) -> Result<Sender<Connected>, std::io::Error>
    where
        A: ToSocketAddrs,
    {
        let Sender { socket, .. } = self;
        let mut addrs = addr.to_socket_addrs()?;
        let addr = addrs.next().expect("could not resolve any `SocketAddr`s");
        socket.connect(addr)?;
        let mode = Connected { addr };
        Ok(Sender { socket, mode })
    }

    /// Sends the given packet on the `Sender`s socket to the given address.
    ///
    /// The given `packet` can be of any type that can be converted directly into a `Packet`. This
    /// includes `Message`, `Bundle` and `(String, Vec<Type>)` (which will be interpreted as a
    /// `Message`).
    ///
    /// On success, returns the number of bytes written.
    ///
    /// This will return a `CommunicationError` if:
    ///
    /// - The given packet fails to be encoded to bytes
    /// - The IP version of the local socket does not match that returned from `addr` or
    /// - The inner `UdpSocket::send_to` call fails.
    pub fn send<P, A>(&self, packet: P, addr: A) -> Result<usize, CommunicationError>
    where
        P: Into<Packet>,
        A: ToSocketAddrs,
    {
        let bytes = encode(packet.into())?;
        let bytes_written = self.socket.send_to(&bytes, addr)?;
        Ok(bytes_written)
    }
}

impl Sender<Connected> {
    /// Returns the address of the socket to which the `Sender` is `Connected`.
    pub fn remote_addr(&self) -> SocketAddr {
        self.mode.addr
    }

    /// Sends the given packet on the `Sender`s socket to the connected address.
    ///
    /// On success, returns the number of bytes written.
    ///
    /// This will return a `CommunicationError` if:
    ///
    /// - The given packet fails to be encoded to bytes
    /// - The IP version of the local socket does not match the connected socket or
    /// - The inner `UdpSocket::send` call fails.
    pub fn send<P>(&self, packet: P) -> Result<usize, CommunicationError>
    where
        P: Into<Packet>,
    {
        let bytes = encode(packet.into())?;
        let bytes_written = self.socket.send(&bytes)?;
        Ok(bytes_written)
    }
}
