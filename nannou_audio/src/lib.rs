//! The nannou audio API and implementation.
//!
//! - [**Host**](./Host.html) - top-level access to device enumeration and spawning streams.
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

use std::marker::PhantomData;
use std::sync::Arc;

pub use cpal;
use cpal::traits::HostTrait;
#[doc(inline)]
pub use cpal::{
    BackendSpecificError, BufferSize, BuildStreamError, DefaultStreamConfigError, DeviceNameError,
    DevicesError, HostId, HostUnavailable, InputCallbackInfo, InputStreamTimestamp,
    OutputCallbackInfo, OutputStreamTimestamp, PauseStreamError, PlayStreamError, StreamError,
    SupportedBufferSize, SupportedInputConfigs, SupportedOutputConfigs, SupportedStreamConfig,
    SupportedStreamConfigsError,
};
pub use dasp_sample;

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
pub struct Host {
    host: Arc<cpal::Host>,
}

impl Host {
    /// Instantiate the current host for the platform.
    pub fn from_id(id: HostId) -> Result<Self, HostUnavailable> {
        let host = cpal::host_from_id(id)?;
        Ok(Self::from_cpal_host(host))
    }

    /// Initialise the API.
    ///
    /// The `Default` implementation for `Host` calls this constructor internally.
    pub fn new() -> Self {
        let host = cpal::default_host();
        Self::from_cpal_host(host)
    }

    /// Initialise the `Host` from an existing CPAL host.
    fn from_cpal_host(host: cpal::Host) -> Self {
        let host = Arc::new(host);
        Host { host }
    }

    /// Enumerate the available audio devices on the system.
    ///
    /// Produces an iterator yielding `Device`s.
    pub fn devices(&self) -> Result<Devices, DevicesError> {
        let devices = self.host.devices()?;
        Ok(Devices { devices })
    }

    /// Enumerate the available audio devices on the system that support input streams.
    ///
    /// Produces an iterator yielding `Device`s.
    pub fn input_devices(&self) -> Result<stream::input::Devices, DevicesError> {
        let devices = self.host.input_devices()?;
        Ok(stream::input::Devices { devices })
    }

    /// Enumerate the available audio devices on the system that support output streams.
    ///
    /// Produces an iterator yielding `Device`s.
    pub fn output_devices(&self) -> Result<stream::output::Devices, DevicesError> {
        let devices = self.host.output_devices()?;
        Ok(stream::output::Devices { devices })
    }

    /// The current default audio input device.
    pub fn default_input_device(&self) -> Option<Device> {
        self.host
            .default_input_device()
            .map(|device| Device { device })
    }

    /// The current default audio output device.
    pub fn default_output_device(&self) -> Option<Device> {
        self.host
            .default_output_device()
            .map(|device| Device { device })
    }

    /// Begin building a new input audio stream.
    ///
    /// If this is the first time a stream has been created, this method will spawn the
    /// `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    pub fn new_input_stream<M, S>(&self, model: M) -> stream::input::BuilderInit<M, S> {
        stream::input::Builder {
            capture: stream::input::default_capture_fn,
            error: stream::default_error_fn,
            builder: self.new_stream(model),
        }
    }

    /// Begin building a new output audio stream.
    ///
    /// If this is the first time a stream has been created, this method will spawn the
    /// `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    pub fn new_output_stream<M, S>(&self, model: M) -> stream::output::BuilderInit<M, S> {
        stream::output::Builder {
            render: stream::output::default_render_fn,
            error: stream::default_error_fn,
            builder: self.new_stream(model),
        }
    }

    // Builder initialisation shared between input and output streams.
    //
    // If this is the first time a stream has been created, this method will spawn the
    // `cpal::EventLoop::run` method on its own thread, ready to run built streams.
    fn new_stream<M, S>(&self, model: M) -> stream::Builder<M, S> {
        stream::Builder {
            host: self.host.clone(),
            model,
            sample_rate: None,
            channels: None,
            frames_per_buffer: None,
            device_buffer_size: None,
            device: None,
            sample_format: PhantomData,
        }
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}
