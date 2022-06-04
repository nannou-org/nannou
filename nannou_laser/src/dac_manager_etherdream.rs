//! Items related specifically to the EtherDream DAC.

use std::io;
use std::sync::atomic::{self, AtomicBool};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

use crate::util::{clamp, map_range};
use crate::{DetectedDacError,RawPoint};
use crate::dac_manager::{DetectDacs,DetectedDac};
/// Callback functions that may be passed to the `detect_dacs_async` function.
pub trait DetectedDacCallback: FnMut(io::Result<DetectedDac>) {}
impl<F> DetectedDacCallback for F where F: FnMut(io::Result<DetectedDac>) {}

/// Messages that driver forward the DAC detector thread.
enum DetectorThreadMsg {
    /// A message indicating to stop detection and close the thread immediately.
    Close,
    /// A message emitted from a timer to step forward and check for DACs again.
    Tick,
}

/// A handle to a non-blocking DAC detection thread.
pub struct DetectEtherDreamDacsAsync {
    msg_tx: mpsc::Sender<DetectorThreadMsg>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl DetectEtherDreamDacsAsync {
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

impl Drop for DetectEtherDreamDacsAsync {
    fn drop(&mut self) {
        self.close_inner();
    }
}

/// An iterator yielding DACs available on the system as they are discovered.
pub(crate) fn detect_dacs() -> io::Result<DetectDacs> {
    let dac_broadcasts = ether_dream::recv_dac_broadcasts()?;
    Ok(DetectDacs::EtherDream { dac_broadcasts })
}

/// Spawn a thread for DAC detection.
///
/// Calls the given `callback` with broadcasts as they are received.
pub(crate) fn detect_dacs_async<F>(
    timeout: Option<Duration>,
    callback: F,
) -> io::Result<DetectEtherDreamDacsAsync>
where
    F: 'static + DetectedDacCallback + Send,
{
    detect_dacs_async_inner(timeout, Box::new(callback) as Box<_>)
}

/// Inner implementation of `detect_dacs_async` removing static dispatch indirection.
fn detect_dacs_async_inner(
    timeout: Option<Duration>,
    mut callback: Box<dyn 'static + DetectedDacCallback + Send>,
) -> io::Result<DetectEtherDreamDacsAsync> {
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
                while let Some(res) = detect_dacs.next() {
                    if let Err(ref e) = res {
                        match e {
                            DetectedDacError::IoError(err) => {
                                if let io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock = err.kind(){
                                    continue 'msgs
                                }
                            }, 
                            _ => (),
                        }
                    }
                    let cb = res.map_err(|e|{
                        if let DetectedDacError::IoError(err) = e{
                            io::Error::from(err)
                        }else{
                            unreachable!("The detect_dacs enum variant here should be 'EtherDream'")
                        }
                    });
                    callback(cb);
                }
            }
        })
        .expect("failed to spawn DAC detection thread");

    Ok(DetectEtherDreamDacsAsync {
        msg_tx,
        thread: Some(thread),
    })
}

// The number of remaining points in the DAC.
pub fn dac_remaining_buffer_capacity(dac: &ether_dream::dac::Dac) -> u16 {
    dac.buffer_capacity - 1 - dac.status.buffer_fullness
}

// Determine the number of points needed to fill the DAC.
pub fn points_to_generate(dac: &ether_dream::dac::Dac, latency_points: u16) -> u16 {
    let remaining_capacity = dac_remaining_buffer_capacity(dac);
    let n = if dac.status.buffer_fullness < latency_points {
        latency_points - dac.status.buffer_fullness
    } else {
        0
    };
    std::cmp::min(n, remaining_capacity)
}

// Constructor for a centered, blank ether dream DAC point.
pub fn centered_blank() -> ether_dream::protocol::DacPoint {
    ether_dream::protocol::DacPoint {
        control: 0,
        x: 0,
        y: 0,
        r: 0,
        g: 0,
        b: 0,
        i: 0,
        u1: 0,
        u2: 0,
    }
}

// Convert a `lase::point::Position` type to an `i16` representation compatible with ether dream.
fn position_to_ether_dream_position([px, py]: crate::point::Position) -> [i16; 2] {
    let min = std::i16::MIN;
    let max = std::i16::MAX;
    let x = map_range(clamp(px, -1.0, 1.0), -1.0, 1.0, min as f64, max as f64) as i16;
    let y = map_range(clamp(py, -1.0, 1.0), -1.0, 1.0, min as f64, max as f64) as i16;
    [x, y]
}

// Convert a `lase::point::Rgb` type to an `u16` representation compatible with ether dream.
fn color_to_ether_dream_color([pr, pg, pb]: crate::point::Rgb) -> [u16; 3] {
    let r = (clamp(pr, 0.0, 1.0) * std::u16::MAX as f32) as u16;
    let g = (clamp(pg, 0.0, 1.0) * std::u16::MAX as f32) as u16;
    let b = (clamp(pb, 0.0, 1.0) * std::u16::MAX as f32) as u16;
    [r, g, b]
}

// Convert the laser point to an ether dream DAC point.
pub fn point_to_ether_dream_point(p: RawPoint) -> ether_dream::protocol::DacPoint {
    let [x, y] = position_to_ether_dream_position(p.position);
    let [r, g, b] = color_to_ether_dream_color(p.color);
    let (control, i, u1, u2) = (0, 0, 0, 0);
    ether_dream::protocol::DacPoint {
        control,
        x,
        y,
        r,
        g,
        b,
        i,
        u1,
        u2,
    }
}

/// Errors that may occur while creating a node crate.
#[derive(Debug, Error)]
pub enum EtherDreamStreamError {
    #[error("laser DAC detection failed: {err}")]
    FailedToDetectDacs {
        #[source]
        err: io::Error,
        /// The number of DAC detection attempts so far.
        attempts: u32,
    },
    #[error("failed to connect the DAC stream (attempt {attempts}): {err}")]
    FailedToConnectStream {
        #[source]
        err: ether_dream::dac::stream::CommunicationError,
        /// The number of connection attempts so far.
        attempts: u32,
    },
    #[error("failed to prepare the DAC stream: {err}")]
    FailedToPrepareStream {
        #[source]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[error("failed to begin the DAC stream: {err}")]
    FailedToBeginStream {
        #[source]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[error("failed to submit data over the DAC stream: {err}")]
    FailedToSubmitData {
        #[source]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[error("failed to submit point rate change over the DAC stream: {err}")]
    FailedToSubmitPointRate {
        #[source]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[error("failed to submit stop command to the DAC stream: {err}")]
    FailedToStopStream {
        #[source]
        err: ether_dream::dac::stream::CommunicationError,
    },
}