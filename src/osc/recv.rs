//! Items related to the `osc::Receiver` implementation.

use super::{decode, rosc, CommunicationError, Connected, Packet, Unconnected};
use std;
use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{self, AtomicBool};
use std::sync::Mutex;

/// The default "maximum transmission unit" size as a number of bytes.
///
/// This is a common MTU size for ethernet.
pub const DEFAULT_MTU: usize = rosc::decoder::MTU;
/// By default UDP sockets are blocking so this is the mode in which the receiver is
/// initialised.
pub const DEFAULT_NON_BLOCKING: bool = false;

/// A type used for receiving OSC packets.
pub struct Receiver<M = Unconnected> {
    buffer: Mutex<Vec<u8>>,
    socket: UdpSocket,
    non_blocking: AtomicBool,
    mode: M,
}

/// An iterator that calls `recv` on the inner `Receiver` and yields the results.
///
/// If the `Receiver` is `Connected`, this will yield `Packet`s.
///
/// If the `Receiver` is `Unconnected`, this will yield `Packet`s alongside the source address.
///
/// Each call to `next` will block until the next packet is received or until some error
/// occurs.
pub struct Iter<'a, M = Unconnected>
where
    M: 'a,
{
    receiver: &'a Receiver<M>,
}

/// An iterator that calls `try_recv` on the inner `Receiver` and yields the results.
///
/// If the `Receiver` is `Connected`, this will yield `Packet`s.
///
/// If the `Receiver` is `Unconnected`, this will yield `Packet`s alongside the source address.
///
/// Each call to `next` will only return `Some` while there are pending messages and will
/// return `None` otherwise.
pub struct TryIter<'a, M = Unconnected>
where
    M: 'a,
{
    receiver: &'a Receiver<M>,
}

impl<M> Receiver<M> {
    /// The socket address that this `Receiver`'s socket was created from.
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.socket.local_addr()
    }

    // Switch the `Receiver`'s inner socket to blocking mode.
    // This is for internal use only - the `recv` methods will call this automatically.
    fn switch_to_blocking(&self) -> Result<(), std::io::Error> {
        if self.non_blocking.load(atomic::Ordering::Relaxed) {
            self.socket.set_nonblocking(false)?;
            self.non_blocking.store(false, atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    // Switch the `Receiver`'s inner socket to non-blocking mode.
    // This is for internal use only - the `try_recv` methods will call this automatically.
    fn switch_to_non_blocking(&self) -> Result<(), std::io::Error> {
        if !self.non_blocking.load(atomic::Ordering::Relaxed) {
            self.socket.set_nonblocking(true)?;
            self.non_blocking.store(true, atomic::Ordering::Relaxed);
        }
        Ok(())
    }
}

impl Receiver<Unconnected> {
    /// Create a `Receiver` that listen for OSC packets on the given address.
    ///
    /// ```no_run
    /// extern crate nannou;
    ///
    /// use nannou::osc::Receiver;
    ///
    /// fn main() {
    ///     let rx = Receiver::bind_to("127.0.0.1:34254").expect("Couldn't bind socket to address");
    /// }
    /// ```
    pub fn bind_to<A>(addr: A) -> Result<Self, std::io::Error>
    where
        A: ToSocketAddrs,
    {
        Self::bind_to_with_mtu(addr, DEFAULT_MTU)
    }

    /// The same as `bind_to`, but allows for manually specifying the MTU (aka "maximum transition
    /// unit").
    ///
    /// The MTU is the maximum UDP packet size in bytes that the `Receiver`'s UDP socket will
    /// receive before returning an error.
    ///
    /// By default this is `DEFAULT_MTU`.
    ///
    /// ```no_run
    /// extern crate nannou;
    ///
    /// use nannou::osc::Receiver;
    ///
    /// fn main() {
    ///     let rx = Receiver::bind_to_with_mtu("127.0.0.1:34254", 60_000)
    ///         .expect("Couldn't bind socket to address");
    /// }
    /// ```
    pub fn bind_to_with_mtu<A>(addr: A, mtu: usize) -> Result<Self, std::io::Error>
    where
        A: ToSocketAddrs,
    {
        let buffer = Mutex::new(vec![0; mtu]);
        let socket = UdpSocket::bind(addr)?;
        let non_blocking = AtomicBool::new(DEFAULT_NON_BLOCKING);
        let mode = Unconnected;
        let receiver = Receiver {
            buffer,
            socket,
            non_blocking,
            mode,
        };
        Ok(receiver)
    }

    /// The same as `bind_to`, but assumes that the IP address is `0.0.0.0`.
    ///
    /// The resulting socket address will be `0.0.0.0:<port>`.
    ///
    /// ```no_run
    /// extern crate nannou;
    ///
    /// use nannou::osc::Receiver;
    ///
    /// fn main() {
    ///     let rx = Receiver::bind(34254).expect("Couldn't bind socket to default address");
    /// }
    /// ```
    pub fn bind(port: u16) -> Result<Self, std::io::Error> {
        Self::bind_to(SocketAddrV4::new(super::default_ipv4_addr(), port))
    }

    /// The same as `bind_to_with_mtu`, but assumes that the IP address is `0.0.0.0`.
    ///
    /// The resulting socket address will be `0.0.0.0:<port>`.
    ///
    /// ```no_run
    /// extern crate nannou;
    ///
    /// use nannou::osc::Receiver;
    ///
    /// fn main() {
    ///     let port = 34254;
    ///     let mtu = 60_000;
    ///     let rx = Receiver::bind_with_mtu(port, mtu).expect("Couldn't bind to default address");
    /// }
    /// ```
    pub fn bind_with_mtu(port: u16, mtu: usize) -> Result<Self, std::io::Error> {
        Self::bind_to_with_mtu(SocketAddrV4::new(super::default_ipv4_addr(), port), mtu)
    }

    /// Connects the `Receiver`'s UDP socket to the given remote address.
    ///
    /// This applies filters so that only data from the given address is received.
    ///
    /// **Panic!**s if the given `addr` cannot resolve to a valid `SocketAddr`.
    ///
    /// ```no_run
    /// extern crate nannou;
    ///
    /// use nannou::osc::Receiver;
    ///
    /// fn main() {
    ///     let tx = Receiver::bind(34254)
    ///         .expect("Couldn't bind to default socket")
    ///         .connect("127.0.0.1:34255")
    ///         .expect("Couldn't connect to socket at address");
    /// }
    /// ```
    pub fn connect<A>(self, addr: A) -> Result<Receiver<Connected>, std::io::Error>
    where
        A: ToSocketAddrs,
    {
        let Receiver {
            buffer,
            socket,
            non_blocking,
            ..
        } = self;
        let mut addrs = addr.to_socket_addrs()?;
        let addr = addrs.next().expect("could not resolve any `SocketAddr`s");
        socket.connect(addr)?;
        let mode = Connected { addr };
        Ok(Receiver {
            buffer,
            socket,
            non_blocking,
            mode,
        })
    }

    /// Waits for the next OSC packet to be received and returns it along with the source address.
    ///
    /// If the socket is currently in non-blocking mode, this method will first switch the socket
    /// to blocking.
    ///
    /// This will return a `CommunicationError` if:
    ///
    /// - Switching the socket from "non_blocking" to "blocking" fails,
    /// - The Mutex around the inner buffer (used to collect bytes) was poisoned,
    /// - The MTU was not large enough to receive a UDP packet,
    /// - The inner `UdpSocket::recv` call fails or
    /// - The socket received some bytes that could not be decoded into an OSC `Packet`.
    pub fn recv(&self) -> Result<(Packet, SocketAddr), CommunicationError> {
        self.switch_to_blocking()?;
        let mut buffer = self.buffer.lock()?;
        let (len, addr) = self.socket.recv_from(&mut buffer)?;
        let packet = decode(&buffer[..len])?;
        Ok((packet, addr))
    }

    /// Checks for a pending OSC packet and returns `Ok(Some)` if there is one waiting along with
    /// the source address.
    ///
    /// If there are no packets waiting (or if the inner UDP socket's `recv` method returns an
    /// error) this will immediately return with `Ok(None)`.
    ///
    /// If the socket is currently in blocking mode, this method will first switch the socket to
    /// non-blocking.
    ///
    /// This will return a `CommunicationError` if:
    ///
    /// - Switching the socket from "blocking" to "non_blocking" fails,
    /// - The Mutex around the inner buffer (used to collect bytes) was poisoned,
    /// - The socket received some bytes that could not be decoded into an OSC `Packet`.
    pub fn try_recv(&self) -> Result<Option<(Packet, SocketAddr)>, CommunicationError> {
        self.switch_to_non_blocking()?;
        let mut buffer = self.buffer.lock()?;
        let (len, addr) = match self.socket.recv_from(&mut buffer) {
            Ok(tuple) => tuple,
            // TODO: Don't know how to check for the specific error that is returned when the
            // non_blocking socket has no bytes waiting, so we just always assume that's what the
            // error was. This should probably be fixed somehow to distinguish between errors.
            Err(_) => return Ok(None),
        };
        let packet = decode(&buffer[..len])?;
        Ok(Some((packet, addr)))
    }

    /// An iterator yielding OSC `Packet`s along with their source address.
    ///
    /// Each call to `next` will block until the next packet is received or until some error
    /// occurs.
    pub fn iter(&self) -> Iter<Unconnected> {
        Iter { receiver: self }
    }

    /// An iterator yielding OSC `Packet`s along with their source address.
    ///
    /// Each call to `next` will only return `Some` while there are pending packets and will return
    /// `None` otherwise.
    pub fn try_iter(&self) -> TryIter<Unconnected> {
        TryIter { receiver: self }
    }
}

impl Receiver<Connected> {
    /// Returns the address of the socket to which the `Receiver` is `Connected`.
    pub fn remote_addr(&self) -> SocketAddr {
        self.mode.addr
    }

    /// Waits for the next OSC packet to be received and returns it.
    ///
    /// If the socket is currently in non-blocking mode, this method will first switch the socket
    /// to blocking.
    ///
    /// This will return a `CommunicationError` if:
    ///
    /// - Switching the socket from "non_blocking" to "blocking" fails,
    /// - The Mutex around the inner buffer (used to collect bytes) was poisoned,
    /// - The MTU was not large enough to receive a UDP packet,
    /// - The inner `UdpSocket::recv` call fails or
    /// - The socket received some bytes that could not be decoded into an OSC `Packet`.
    pub fn recv(&self) -> Result<Packet, CommunicationError> {
        self.switch_to_blocking()?;
        let mut buffer = self.buffer.lock()?;
        let len = self.socket.recv(&mut buffer)?;
        let packet = decode(&buffer[..len])?;
        Ok(packet)
    }

    /// Checks for a pending OSC packet and returns `Ok(Some)` if there is one waiting.
    ///
    /// If there are no packets waiting (or if the inner UDP socket's `recv` method returns an
    /// error) this will immediately return with `Ok(None)`.
    ///
    /// If the socket is currently in blocking mode, this method will first switch the socket to
    /// non-blocking.
    ///
    /// This will return a `CommunicationError` if:
    ///
    /// - Switching the socket from "blocking" to "non_blocking" fails,
    /// - The Mutex around the inner buffer (used to collect bytes) was poisoned,
    /// - The socket received some bytes that could not be decoded into an OSC `Packet`.
    pub fn try_recv(&self) -> Result<Option<Packet>, CommunicationError> {
        self.switch_to_non_blocking()?;
        let mut buffer = self.buffer.lock()?;
        let len = match self.socket.recv(&mut buffer) {
            Ok(len) => len,
            // TODO: Don't know how to check for the specific error that is returned when the
            // non_blocking socket has no bytes waiting, so we just always assume that's what the
            // error was. This should probably be fixed somehow to distinguish between errors.
            Err(_) => return Ok(None),
        };
        let packet = decode(&buffer[..len])?;
        Ok(Some(packet))
    }

    /// An iterator yielding OSC `Packet`s.
    ///
    /// Each call to `next` will block until the next packet is received or until some error
    /// occurs.
    pub fn iter(&self) -> Iter<Connected> {
        Iter { receiver: self }
    }

    /// An iterator yielding OSC `Packet`s.
    ///
    /// Each call to `next` will only return `Some` while there are pending packets and will return
    /// `None` otherwise.
    pub fn try_iter(&self) -> TryIter<Connected> {
        TryIter { receiver: self }
    }
}

impl<'a> Iterator for Iter<'a, Connected> {
    type Item = Packet;
    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok()
    }
}

impl<'a> Iterator for Iter<'a, Unconnected> {
    type Item = (Packet, SocketAddr);
    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok()
    }
}

impl<'a> Iterator for TryIter<'a, Connected> {
    type Item = Packet;
    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.try_recv().ok().and_then(|p| p)
    }
}

impl<'a> Iterator for TryIter<'a, Unconnected> {
    type Item = (Packet, SocketAddr);
    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.try_recv().ok().and_then(|p| p)
    }
}
