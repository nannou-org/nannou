//! The nannou audio API and implementation.
//!
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

pub extern crate cpal;
pub extern crate sample;

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
