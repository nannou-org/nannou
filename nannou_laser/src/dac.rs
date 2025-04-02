//! Items related to DACs and DAC detection.

use crate::protocol;
use std::io;
use std::sync::atomic::{self, AtomicBool};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

/// Callback functions that may be passed to the `detect_dacs_async` function.
pub trait DetectedDacCallback: FnMut(io::Result<DetectedDac>) {}
impl<F> DetectedDacCallback for F where F: FnMut(io::Result<DetectedDac>) {}

/// A persistent, unique identifier associated with a DAC (like a MAC address).
///
/// It should be possible to use this to uniquely identify the same DAC on different occasions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    EtherDream(protocol::etherdream::Id),
    LaserCube(protocol::lasercube::Id),
}

/// An available DAC detected on the system.
#[derive(Clone, Debug)]
pub enum DetectedDac {
    /// An ether dream laser DAC discovered via the ether dream protocol broadcast message.
    EtherDream(protocol::etherdream::DetectedDac),
    LaserCube(protocol::lasercube::DetectedDac),
}

/// An iterator yielding laser DACs available on the system as they are discovered.
pub struct DetectDacs {
    pub(crate) dac_broadcasts: ether_dream::RecvDacBroadcasts,
}

/// Messages that driver forward the DAC detector thread.
enum DetectorThreadMsg {
    /// A message indicating to stop detection and close the thread immediately.
    Close,
    /// A message emitted from a timer to step forward and check for DACs again.
    Tick,
}

/// A handle to a non-blocking DAC detection thread.
pub struct DetectDacsAsync {
    msg_tx: mpsc::Sender<DetectorThreadMsg>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl DetectedDac {
    /// The maximum point rate allowed by the DAC.
    pub fn max_point_hz(&self) -> u32 {
        match self {
            DetectedDac::EtherDream(dac) => dac.max_point_hz(),
            DetectedDac::LaserCube(dac) => dac.max_point_hz(),
        }
    }

    /// The number of points that can be stored within the buffer.
    pub fn buffer_capacity(&self) -> u32 {
        match self {
            DetectedDac::EtherDream(dac) => dac.buffer_capacity(),
            DetectedDac::LaserCube(dac) => dac.buffer_capacity(),
        }
    }

    /// A persistent, unique identifier associated with the DAC (like a MAC
    /// address).
    ///
    /// It should be possible to use this to uniquely identify the same DAC on
    /// different occasions.
    pub fn id(&self) -> Id {
        match self {
            DetectedDac::EtherDream(dac) => Id::EtherDream(dac.id()),
            DetectedDac::LaserCube(dac) => Id::LaserCube(dac.id()),
        }
    }
}

impl DetectDacs {
    /// Specify a duration for the detection to wait before timing out.
    pub fn set_timeout(&self, duration: Option<std::time::Duration>) -> io::Result<()> {
        self.dac_broadcasts.set_timeout(duration)
    }

    /// Specify whether or not retrieving the next DAC should block.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.dac_broadcasts.set_nonblocking(nonblocking)
    }
}

impl DetectDacsAsync {
    /// Close the DAC detection thread.
    pub fn close(mut self) {
        self.close_inner()
    }

    /// Private close implementation shared between `Drop` and `close`.
    fn close_inner(&mut self) {
        if let Some(thread) = self.thread.take() {
            if self.msg_tx.send(DetectorThreadMsg::Close).is_ok() {
                thread.join().ok();
            }
        }
    }
}

impl Iterator for DetectDacs {
    type Item = io::Result<DetectedDac>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.dac_broadcasts.next()?;
        match res {
            Err(err) => Some(Err(err)),
            Ok((broadcast, source_addr)) => Some(Ok(DetectedDac::EtherDream(
                protocol::etherdream::DetectedDac {
                    broadcast,
                    source_addr,
                },
            ))),
        }
    }
}

impl Drop for DetectDacsAsync {
    fn drop(&mut self) {
        self.close_inner();
    }
}

/// An iterator yielding DACs available on the system as they are discovered.
pub(crate) fn detect_dacs() -> io::Result<DetectDacs> {
    let dac_broadcasts = ether_dream::recv_dac_broadcasts()?;
    Ok(DetectDacs { dac_broadcasts })
}

/// Spawn a thread for DAC detection.
///
/// Calls the given `callback` with broadcasts as they are received.
pub(crate) fn detect_dacs_async<F>(
    timeout: Option<Duration>,
    callback: F,
) -> io::Result<DetectDacsAsync>
where
    F: 'static + DetectedDacCallback + Send,
{
    detect_dacs_async_inner(timeout, Box::new(callback) as Box<_>)
}

/// Inner implementation of `detect_dacs_async` removing static dispatch indirection.
fn detect_dacs_async_inner(
    timeout: Option<Duration>,
    mut callback: Box<dyn 'static + DetectedDacCallback + Send>,
) -> io::Result<DetectDacsAsync> {
    let mut detect_dacs = detect_dacs()?;
    detect_dacs.set_nonblocking(true)?;
    let (msg_tx, msg_rx) = mpsc::channel();
    let msg_tx2 = msg_tx.clone();
    let thread = std::thread::Builder::new()
        .name("nannou_laser-dac-detection".to_string())
        .spawn(move || {
            // For closing the timer thread.
            let is_closed = Arc::new(AtomicBool::new(false));

            // Start the timer.
            let is_closed2 = is_closed.clone();
            std::thread::spawn(move || {
                let tick_interval = timeout.unwrap_or(std::time::Duration::from_secs(1));
                while !is_closed2.load(atomic::Ordering::Relaxed) {
                    std::thread::sleep(tick_interval);
                    if msg_tx2.send(DetectorThreadMsg::Tick).is_err() {
                        break;
                    }
                }
            });

            // Loop until we receive a close.
            'msgs: for msg in msg_rx {
                if let DetectorThreadMsg::Close = msg {
                    is_closed.store(true, atomic::Ordering::Relaxed);
                    break;
                }
                for res in detect_dacs.by_ref() {
                    if let Err(ref e) = res {
                        match e.kind() {
                            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => continue 'msgs,
                            _ => (),
                        }
                    }
                    callback(res);
                }
            }
        })
        .expect("failed to spawn DAC detection thread");

    Ok(DetectDacsAsync {
        msg_tx,
        thread: Some(thread),
    })
}
