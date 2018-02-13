use audio::cpal;
use std::ops::Deref;

/// A device that can be used to spawn an audio stream.
pub struct Device {
    pub(crate) device: cpal::Device,
}

/// An iterator yielding all available audio devices.
pub struct Devices {
    pub(crate) devices: cpal::Devices,
}

impl Deref for Device {
    type Target = cpal::Device;
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
