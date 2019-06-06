//! The nannou audio API and implementation.
//!
//! - [**Api**](./Api.html) - top-level access to device enumeration and spawning streams.
//! - [**Stream**](./stream/struct.Stream.html) - for managing an input/output audio stream. This may be
//!   created via the **App**'s **Audio** API.
//! - [**Buffer**](./buffer/struct.Buffer.html) - contains audio data, either for reading or writing.
//!   This is passed to the `capture` or `render` function for each stream.
//! - [**Devices**](./device/struct.Devices.html) - for enumerating all audio devices on the system.
//! - [**Device**](./device/struct.Device.html) - for querying information about supported stream
//!   formats or for creating a stream targeted towards a specific audio device.
//! - [**Receiver**](./receiver/struct.Receiver.html) and
//!   [**Requester**](./requester/struct.Requester.html) for buffering input and output streams that
//!   may deliver buffers of inconsistent sizes into a stream of consistently sized buffers.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::{mpsc, Arc};
use std::thread;

pub use self::buffer::Buffer;
pub use self::device::{Device, Devices};
pub use self::receiver::Receiver;
pub use self::requester::Requester;
pub use self::stream::Stream;

pub mod buffer;
pub mod device;
pub mod receiver;
pub mod requester;
pub mod stream;

/// The top-level audio API, for enumerating devices and spawning input/output streams.
pub struct Api {
    event_loop: Arc<cpal::EventLoop>,
    process_fn_tx: RefCell<Option<mpsc::Sender<stream::ProcessFnMsg>>>,
}

impl Api {
    /// Initialise the API.
    ///
    /// Internally, this creates a new, inactive CPAL event loop ready for stream creation.
    ///
    /// The `Default` implementation for `Api` calls this constructor internally.
    pub fn new() -> Self {
        let event_loop = Arc::new(cpal::EventLoop::new());
        let process_fn_tx = RefCell::new(None);
        Api {
            event_loop,
            process_fn_tx,
        }
    }

    /// Enumerate the available audio devices on the system.
    ///
    /// Produces an iterator yielding `Device`s.
    pub fn devices(&self) -> Devices {
        let devices = cpal::devices();
        Devices { devices }
    }

    /// Enumerate the available audio devices on the system that support input streams.
    ///
    /// Produces an iterator yielding `Device`s.
    pub fn input_devices(&self) -> stream::input::Devices {
        let devices = cpal::input_devices();
        stream::input::Devices { devices }
    }

    /// Enumerate the available audio devices on the system that support output streams.
    ///
    /// Produces an iterator yielding `Device`s.
    pub fn output_devices(&self) -> stream::output::Devices {
        let devices = cpal::output_devices();
        stream::output::Devices { devices }
    }

    /// The current default audio input device.
    pub fn default_input_device(&self) -> Option<Device> {
        cpal::default_input_device().map(|device| Device { device })
    }

    /// The current default audio output device.
    pub fn default_output_device(&self) -> Option<Device> {
        cpal::default_output_device().map(|device| Device { device })
    }

    /// Begin building a new input audio stream.
    ///
    /// If this is the first time a stream has been created, this method will spawn the
    /// `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    pub fn new_input_stream<M, F, S>(
        &self,
        model: M,
        capture: F,
    ) -> stream::input::Builder<M, F, S> {
        stream::input::Builder {
            capture,
            builder: self.new_stream(model),
        }
    }

    /// Begin building a new output audio stream.
    ///
    /// If this is the first time a stream has been created, this method will spawn the
    /// `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    pub fn new_output_stream<M, F, S>(
        &self,
        model: M,
        render: F,
    ) -> stream::output::Builder<M, F, S> {
        stream::output::Builder {
            render,
            builder: self.new_stream(model),
        }
    }

    // Builder initialisation shared between input and output streams.
    //
    // If this is the first time a stream has been created, this method will spawn the
    // `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    fn new_stream<M, S>(&self, model: M) -> stream::Builder<M, S> {
        let process_fn_tx = if self.process_fn_tx.borrow().is_none() {
            let event_loop = self.event_loop.clone();
            let (tx, rx) = mpsc::channel();
            let mut loop_context = stream::LoopContext::new(rx);
            thread::Builder::new()
                .name("cpal::EventLoop::run thread".into())
                .spawn(move || event_loop.run(move |id, data| loop_context.process(id, data)))
                .expect("failed to spawn cpal::EventLoop::run thread");
            *self.process_fn_tx.borrow_mut() = Some(tx.clone());
            tx
        } else {
            self.process_fn_tx.borrow().as_ref().unwrap().clone()
        };

        stream::Builder {
            event_loop: self.event_loop.clone(),
            process_fn_tx: process_fn_tx,
            model,
            sample_rate: None,
            channels: None,
            frames_per_buffer: None,
            device: None,
            sample_format: PhantomData,
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}
