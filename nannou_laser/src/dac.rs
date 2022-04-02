//! Items related to EtherDream DAC detection.

use std::io;
use std::sync::atomic::{self, AtomicBool};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use crate::DetectedDacError;
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
