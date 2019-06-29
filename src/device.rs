use cpal::Device as DeviceTrait;
use cpal::platform::{Device as CpalDevice, Devices as CpalDevices};
use std::ops::Deref;

/// A device that can be used to spawn an audio stream.
pub struct Device {
    pub(crate) device: CpalDevice,
}

/// An iterator yielding all available audio devices.
pub struct Devices {
    pub(crate) devices: CpalDevices,
}

impl Device {
    /// The maximum number of output channels of any format supported by this device.
    pub fn max_supported_output_channels(&self) -> usize {
        self.supported_output_formats()
            .expect("failed to get supported output audio stream formats")
            .map(|fmt| fmt.channels as usize)
            .max()
            .unwrap_or(0)
    }

    /// The maximum number of input channels of any format supported by this device.
    pub fn max_supported_input_channels(&self) -> usize {
        self.supported_input_formats()
            .expect("failed to get supported input audio stream formats")
            .map(|fmt| fmt.channels as usize)
            .max()
            .unwrap_or(0)
    }
}

impl Deref for Device {
    type Target = CpalDevice;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl Iterator for Devices {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        self.devices.next().map(|device| Device { device })
    }
}
