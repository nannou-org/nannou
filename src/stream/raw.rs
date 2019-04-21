use crate::{DetectedDac, RawPoint};
use crate::util::{clamp, map_range};
use derive_more::From;
use failure::Fail;
use std::io;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{self, AtomicBool};
use std::sync::{mpsc, Arc, Mutex};

/// The function that will be called when a `Buffer` of points is requested.
pub trait RenderFn<M>: Fn(&mut M, &mut Buffer) {}
impl<M, F> RenderFn<M> for F where F: Fn(&mut M, &mut Buffer) {}

/// A clone-able handle around a raw laser stream.
#[derive(Clone)]
pub struct Stream<M> {
    // A channel of updating the raw laser stream thread state.
    state_update_tx: mpsc::Sender<StateUpdate>,
    // A channel for sending model updates to the laser stream thread.
    model_update_tx: mpsc::Sender<ModelUpdate<M>>,
    // Data shared between each `Stream` handle to a single stream.
    shared: Arc<Shared<M>>,
}

// State managed on the laser thread.
#[derive(Clone)]
struct State {
    point_hz: u32,
    latency_points: u32,
}

// Data shared between each `Stream` handle to a single stream.
struct Shared<M> {
    // The user's laser model
    model: Arc<Mutex<Option<M>>>,
    // Whether or not the stream is currently paused.
    is_paused: AtomicBool,
    // Whether or not the stream has been closed.
    is_closed: Arc<AtomicBool>,
}

/// A buffer of laser points yielded by either a raw or frame stream source function.
#[derive(Debug)]
pub struct Buffer {
    pub(crate) point_hz: u32,
    pub(crate) latency_points: u32,
    pub(crate) points: Box<[RawPoint]>,
}

/// A type allowing to build a raw laser stream.
pub struct Builder<M, F> {
    /// The laser API inner state, used to find a DAC during `build` if one isn't specified.
    pub(crate) api_inner: Arc<super::super::Inner>,
    pub builder: super::Builder,
    pub model: M,
    pub render: F,
}

// The type used for sending state updates from the stream handle thread to the laser thread.
type StateUpdate = Box<FnMut(&mut State) + 'static + Send>;

/// The type used for sending model updates from the stream handle thread to the laser thread.
pub type ModelUpdate<M> = Box<FnMut(&mut M) + 'static + Send>;

/// Errors that may occur while running a laser stream.
#[derive(Debug, Fail, From)]
pub enum RawStreamError {
    #[fail(display = "an Ether Dream DAC stream error occurred: {}", err)]
    EtherDreamStream {
        #[fail(cause)]
        err: EtherDreamStreamError,
    }
}

/// Errors that may occur while creating a node crate.
#[derive(Debug, Fail, From)]
pub enum EtherDreamStreamError {
    #[fail(display = "laser DAC detection failed: {}", err)]
    FailedToDetectDacs {
        #[fail(cause)]
        err: io::Error,
    },
    #[fail(display = "failed to connect the DAC stream: {}", err)]
    FailedToConnectStream {
        #[fail(cause)]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[fail(display = "failed to prepare the DAC stream: {}", err)]
    FailedToPrepareStream {
        #[fail(cause)]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[fail(display = "failed to begin the DAC stream: {}", err)]
    FailedToBeginStream {
        #[fail(cause)]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[fail(display = "failed to submit data over the DAC stream: {}", err)]
    FailedToSubmitData {
        #[fail(cause)]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[fail(display = "failed to submit point rate change over the DAC stream: {}", err)]
    FailedToSubmitPointRate {
        #[fail(cause)]
        err: ether_dream::dac::stream::CommunicationError,
    },
    #[fail(display = "failed to submit stop command to the DAC stream: {}", err)]
    FailedToStopStream {
        #[fail(cause)]
        err: ether_dream::dac::stream::CommunicationError,
    },
}

impl<M> Stream<M> {
    /// Update the rate at which the DAC should process points per second.
    ///
    /// This value should be no greater than the detected DAC's `max_point_hz`.
    ///
    /// By default this value is `stream::DEFAULT_POINT_HZ`.
    pub fn set_point_hz(&self, point_hz: u32) -> Result<(), mpsc::SendError<()>> {
        self.send_raw_state_update(move |state| state.point_hz = point_hz)
            .map_err(|_| mpsc::SendError(()))
    }

    /// The maximum latency specified as a number of points.
    ///
    /// Each time the laser indicates its "fullness", the raw stream will request enough points
    /// from the render function to fill the DAC buffer up to `latency_points`.
    ///
    /// This value should be no greaterthan the DAC's `buffer_capacity`.
    pub fn set_latency_points(&self, points: u32) -> Result<(), mpsc::SendError<()>> {
        self.send_raw_state_update(move |state| state.latency_points = points)
            .map_err(|_| mpsc::SendError(()))
    }

    /// Send the given model update to the laser thread to be applied ASAP.
    ///
    /// If the laser is currently rendering, the update will be applied immediately after the
    /// function call completes.
    ///
    /// If the stream is currently paused, the update will be applied immediately.
    ///
    /// **Note:** This function will be applied on the real-time laser thread so users should avoid
    /// performing any kind of I/O, locking, blocking, (de)allocations or anything that may run for
    /// an indeterminate amount of time.
    pub fn send<F>(&self, update: F) -> Result<(), mpsc::SendError<ModelUpdate<M>>>
    where
        F: FnOnce(&mut M) + Send + 'static,
    {
        // NOTE: The following code may mean that on extremely rare occasions an update does
        // not get applied for an indeterminate amount of time. This might be the case if a
        // stream is unpaused but becomes paused *immediately* after the `is_paused` atomic
        // condition is read as `false` - the update would be sent but the stream would be
        // paused and in turn the update will not get processed until the stream is unpaused
        // again. It would be nice to work out a solution to this that does not require
        // spawning another thread for each stream.

        // If the thread is currently paused, take the lock and immediately apply it as we know
        // there will be no contention with the laser thread.
        if self.shared.is_paused.load(atomic::Ordering::Relaxed) {
            if let Ok(mut guard) = self.shared.model.lock() {
                let mut model = guard.take().unwrap();
                update(&mut model);
                *guard = Some(model);
            }
        // Otherwise send the update to the laser thread.
        } else {
            // Move the `FnOnce` into a `FnMut` closure so that it can be called when it gets to
            // the laser thread. We do this as it's currently not possible to call a `Box<FnOnce>`,
            // as `FnOnce`'s `call` method takes `self` by value and thus is technically not object
            // safe.
            let mut update_opt = Some(update);
            let update_fn = move |model: &mut M| {
                if let Some(update) = update_opt.take() {
                    update(model);
                }
            };
            self.model_update_tx.send(Box::new(update_fn))?;
        }

        Ok(())
    }

    // Simplify sending a `StateUpdate` to the laser thread.
    fn send_raw_state_update<F>(&self, update: F) -> Result<(), mpsc::SendError<StateUpdate>>
    where
        F: FnOnce(&mut State) + Send + 'static,
    {
        let mut update_opt = Some(update);
        let update_fn = move |state: &mut State| {
            if let Some(update) = update_opt.take() {
                update(state);
            }
        };
        self.state_update_tx.send(Box::new(update_fn))
    }
}

impl Buffer {
    /// The rate at which these points will be emitted by the DAC.
    pub fn point_hz(&self) -> u32 {
        self.point_hz
    }

    /// The maximum number of points with which to fill the DAC's buffer.
    pub fn latency_points(&self) -> u32 {
        self.latency_points
    }
}

impl<M, F> Builder<M, F> {
    /// The DAC with which the stream should be established.
    ///
    /// If none is specified, the stream will associate itself with the first DAC detecged on the
    /// system.
    ///
    /// ## DAC Dropouts
    ///
    /// If communication is lost with the DAC that was specified by this method, the stream will
    /// attempt to re-establish connection with this DAC as quickly as possible. If no DAC was
    /// specified, the stream will attempt to establish a new connection with the next DAC that is
    /// detected on the system.
    pub fn detected_dac(mut self, dac: DetectedDac) -> Self {
        self.builder.dac = Some(dac);
        self
    }

    /// The initial rate at which the DAC should process points per second.
    ///
    /// This value should be no greater than the detected DAC's `max_point_hz`.
    ///
    /// By default this value is `stream::DEFAULT_POINT_HZ`.
    pub fn point_hz(mut self, point_hz: u32) -> Self {
        self.builder.point_hz = Some(point_hz);
        self
    }

    /// The maximum latency specified as a number of points.
    ///
    /// Each time the laser indicates its "fullness", the raw stream will request enough points
    /// from the render function to fill the DAC buffer up to `latency_points`.
    pub fn latency_points(mut self, points: u32) -> Self {
        self.builder.latency_points = Some(points);
        self
    }

    /// Build the stream with the specified parameters.
    ///
    /// **Note:** If no `dac` was specified, this will method will block until a DAC is detected.
    /// The first detected DAC is the DAC with which a stream will be established.
    pub fn build(self) -> io::Result<Stream<M>>
    where
        M: 'static + Send,
        F: 'static + RenderFn<M> + Send,
    {
        let Builder { api_inner, builder, model, render } = self;

        // Prepare the model for sharing between the laser thread and stream handle.
        let model = Arc::new(Mutex::new(Some(model)));
        let model_2 = model.clone();

        // The channels used for sending updates to the model via the stream handle.
        let (model_update_tx, m_rx) = mpsc::channel();
        let (state_update_tx, s_rx) = mpsc::channel();

        // Retrieve the specified point rate or use a default.
        let point_hz = builder.point_hz.unwrap_or(super::DEFAULT_POINT_HZ);
        // Retrieve the latency as a number of points.
        let latency_points = builder.latency_points
            .unwrap_or_else(|| default_latency_points(point_hz));

        // The raw laser stream state to live on the laser thread.
        let state = Arc::new(Mutex::new(State { point_hz, latency_points }));

        // Retrieve whether or not the user specified a detected DAC.
        let mut maybe_dac = builder.dac;

        // A flag for tracking whether or not the stream has been closed.
        let is_closed = Arc::new(AtomicBool::new(false));
        let is_closed2 = is_closed.clone();

        // Spawn the thread for communicating with the DAC.
        std::thread::Builder::new()
            .name("raw_laser_stream_thread".into())
            .spawn(move || {
                let mut connect_attempts = 3;
                while !is_closed2.load(atomic::Ordering::Relaxed) {
                    // If there are no more remaining connection attempts, try to redetect the DAC
                    // if a specific DAC was specified by the user.
                    if connect_attempts == 0 {
                        connect_attempts = 3;
                        if let Some(ref mut dac) = maybe_dac {
                            let dac_id = dac.id();
                            eprintln!("re-attempting to detect DAC with id: {:?}", dac_id);
                            *dac = match api_inner.detect_dac(dac_id) {
                                Ok(dac) => dac,
                                Err(err) => {
                                    let err = EtherDreamStreamError::FailedToDetectDacs { err };
                                    return Err(RawStreamError::EtherDreamStream { err });
                                }
                            };
                        }
                    }

                    // Retrieve the DAC or find one.
                    let dac = match maybe_dac {
                        Some(ref dac) => dac.clone(),
                        None => api_inner.detect_dacs()
                            .map_err(|err| EtherDreamStreamError::FailedToDetectDacs { err })?
                            .next()
                            .expect("ether dream DAC detection iterator should never return `None`")
                            .map_err(|err| EtherDreamStreamError::FailedToDetectDacs { err })?,
                    };

                    // Connect and run the laser stream.
                    match run_laser_stream(&dac, &state, &model_2, &render, &s_rx, &m_rx, &is_closed2) {
                        Ok(()) => return Ok(()),
                        Err(RawStreamError::EtherDreamStream { err }) => match err {
                            // If we failed to connect to the DAC, keep track of attempts.
                            EtherDreamStreamError::FailedToConnectStream { err } => {
                                eprintln!("failed to connect to stream: {}", err);
                                connect_attempts -= 1;
                                eprintln!(
                                    "connection attempts remaining before re-detecting DAC: {}",
                                    connect_attempts,
                                );
                                // Sleep for a moment to avoid spamming the socket.
                                std::thread::sleep(std::time::Duration::from_millis(16));
                            }

                            // If we failed to prepare the stream or submit data, retry.
                            EtherDreamStreamError::FailedToPrepareStream { .. }
                            | EtherDreamStreamError::FailedToBeginStream { .. }
                            | EtherDreamStreamError::FailedToSubmitData { .. }
                            | EtherDreamStreamError::FailedToSubmitPointRate { .. } => {
                                eprintln!("{} - will now attempt to reconnect", err);
                            }

                            // Return all other errors.
                            err => return Err(RawStreamError::EtherDreamStream { err }),
                        }
                    }
                }

                Ok(())
            })?;

        let is_paused = AtomicBool::new(false);
        let shared = Arc::new(Shared { model, is_paused, is_closed });
        let stream = Stream { shared, state_update_tx, model_update_tx };
        Ok(stream)
    }
}

impl Deref for Buffer {
    type Target = [RawPoint];
    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.points
    }
}

impl<M> Drop for Stream<M> {
    fn drop(&mut self) {
        self.shared.is_closed.store(true, atomic::Ordering::Relaxed)
    }
}

/// Given the point rate, determine a default latency at ~16ms.
pub fn default_latency_points(point_hz: u32) -> u32 {
    super::points_per_frame(point_hz, 60)
}

// The function to run on the laser stream thread.
fn run_laser_stream<M, F>(
    dac: &DetectedDac,
    state: &Arc<Mutex<State>>,
    model: &Arc<Mutex<Option<M>>>,
    render: F,
    state_update_rx: &mpsc::Receiver<StateUpdate>,
    model_update_rx: &mpsc::Receiver<ModelUpdate<M>>,
    is_closed: &AtomicBool,
) -> Result<(), RawStreamError>
where
    F: RenderFn<M>,
{
    // Currently only ether dream is supported, so retrieve the broadcast and addr.
    let (broadcast, src_addr) = match dac {
        DetectedDac::EtherDream { broadcast, source_addr } => {
            (broadcast, source_addr)
        }
    };

    // A buffer for collecting model updates.
    let mut pending_model_updates: Vec<ModelUpdate<M>> = Vec::new();

    // Establish the TCP connection.
    let ip = src_addr.ip().clone();

    let mut stream = ether_dream::dac::stream::connect(&broadcast, ip)
        .map_err(|err| EtherDreamStreamError::FailedToConnectStream { err })?;

    // Prepare the DAC's playback engine and await the repsonse.
    stream
        .queue_commands()
        .prepare_stream()
        .submit()
        .map_err(|err| EtherDreamStreamError::FailedToPrepareStream { err })?;

    let dac_max_point_hz = dac.max_point_hz();

    // Get the initial point hz by clamping via the DAC's maximum point rate.
    let init_point_hz = {
        let hz = state.lock().expect("failed to acquire raw state lock").point_hz;
        std::cmp::min(hz, dac_max_point_hz)
    };

    // Queue the initial frame and tell the DAC to begin producing output.
    let low_water_mark = 0;
    let n_points = dac_remaining_buffer_capacity(stream.dac());
    stream
        .queue_commands()
        .data((0..n_points).map(|_| centered_blank()))
        .begin(low_water_mark, init_point_hz)
        .submit()
        .map_err(|err| EtherDreamStreamError::FailedToBeginStream { err })?;

    // For collecting the ether-dream points.
    let mut ether_dream_points = vec![];

    while !is_closed.load(atomic::Ordering::Relaxed) {
        // Collect any pending updates.
        pending_model_updates.extend(model_update_rx.try_iter());
        // If there are some updates available, take the lock and apply them.
        if !pending_model_updates.is_empty() {
            if let Ok(mut guard) = model.lock() {
                let mut model = guard.take().unwrap();
                for mut update in pending_model_updates.drain(..) {
                    update(&mut model);
                }
                *guard = Some(model);
            }
        }

        // Check for updates and retrieve a copy of the state.
        let (state, prev_point_hz) = {
            let mut state = state.lock().expect("failed to acquare raw state lock");

            // Keep track of whether or not the `point_hz` as changed.
            let prev_point_hz = std::cmp::min(state.point_hz, dac.max_point_hz());

            // Apply updates.
            for mut state_update in state_update_rx.try_iter() {
                (*state_update)(&mut state);
            }

            (state.clone(), prev_point_hz)
        };

        // Clamp the point hz by the DAC's maximum point rate.
        let point_hz = std::cmp::min(state.point_hz, dac.max_point_hz());

        // If the point rate changed, we need to tell the DAC and set the control value on point 0.
        // TODO
        let point_rate_changed = point_hz != prev_point_hz;
        if point_rate_changed {
            stream
                .queue_commands()
                .point_rate(point_hz)
                .submit()
                .map_err(|err| EtherDreamStreamError::FailedToSubmitPointRate { err })?;
        }

        // Clamp the latency by the DAC's buffer capacity.
        let latency_points = std::cmp::min(state.latency_points, dac.buffer_capacity());
        // Determine how many points the DAC can currently receive.
        let n_points = points_to_generate(stream.dac(), latency_points as u16) as usize;

        // The buffer that the user will write to. TODO: Re-use this.
        let mut buffer = Buffer {
            point_hz,
            latency_points: latency_points as _,
            points: vec![RawPoint::centered_blank(); n_points].into_boxed_slice(),
        };

        // Request the points from the user.
        if let Ok(mut guard) = model.lock() {
            let mut m = guard.take().unwrap();
            render(&mut m, &mut buffer);
            *guard = Some(m);
        }

        // Retrieve the points.
        ether_dream_points.extend(buffer.iter().cloned().map(point_to_ether_dream_point));

        // If the point rate changed, set the control value on the first point to trigger it.
        if point_rate_changed {
            ether_dream_points[0].control = ether_dream::dac::PointControl::CHANGE_RATE.bits();
        }

        // Submit the points.
        stream
            .queue_commands()
            .data(ether_dream_points.drain(..))
            .submit()
            .map_err(|err| EtherDreamStreamError::FailedToSubmitData { err })?;
    }

    stream
        .queue_commands()
        .stop()
        .submit()
        .map_err(|err| EtherDreamStreamError::FailedToStopStream { err })?;
    Ok(())
}

// The number of remaining points in the DAC.
fn dac_remaining_buffer_capacity(dac: &ether_dream::dac::Dac) -> u16 {
    dac.buffer_capacity - 1 - dac.status.buffer_fullness
}

// Determine the number of points needed to fill the DAC.
fn points_to_generate(dac: &ether_dream::dac::Dac, latency_points: u16) -> u16 {
    let remaining_capacity = dac_remaining_buffer_capacity(dac);
    let n = if dac.status.buffer_fullness < latency_points {
        latency_points - dac.status.buffer_fullness
    } else {
        0
    };
    std::cmp::min(n, remaining_capacity)
}

// Constructor for a centered, blank ether dream DAC point.
fn centered_blank() -> ether_dream::protocol::DacPoint {
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
fn point_to_ether_dream_point(p: RawPoint) -> ether_dream::protocol::DacPoint {
    let [x, y] = position_to_ether_dream_position(p.position);
    let [r, g, b] = color_to_ether_dream_color(p.color);
    let (control, i, u1, u2) = (0, 0, 0, 0);
    ether_dream::protocol::DacPoint { control, x, y, r, g, b, i, u1, u2 }
}
