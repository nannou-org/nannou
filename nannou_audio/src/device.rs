use std::ops::Deref;

use cpal::traits::DeviceTrait;

use crate::{
    DefaultStreamConfigError, DeviceNameError, SupportedStreamConfig, SupportedStreamConfigsError,
};

/// A device that can be used to spawn an audio stream.
pub struct Device {
    pub(crate) device: cpal::Device,
}

/// An iterator yielding all available audio devices.
pub struct Devices {
    pub(crate) devices: cpal::Devices,
}

/// An iterator yielding configs that are supported by the backend.
pub type SupportedInputConfigs = cpal::SupportedInputConfigs;

/// An iterator yielding configs that are supported by the backend.
pub type SupportedOutputConfigs = cpal::SupportedOutputConfigs;

impl Device {
    /// The unique name associated with this device.
    pub fn name(&self) -> Result<String, DeviceNameError> {
        self.device.name()
    }

    /// An iterator yielding formats that are supported by the backend.
    ///
    /// Can return an error if the device is no longer valid (e.g. it has been disconnected).
    pub fn supported_input_configs(
        &self,
    ) -> Result<SupportedInputConfigs, SupportedStreamConfigsError> {
        self.device.supported_input_configs()
    }

    /// An iterator yielding configs that are supported by the backend.
    ///
    /// Can return an error if the device is no longer valid (e.g. it has been disconnected).
    pub fn supported_output_configs(
        &self,
    ) -> Result<SupportedOutputConfigs, SupportedStreamConfigsError> {
        self.device.supported_output_configs()
    }

    /// The default config used for input streams.
    pub fn default_input_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        self.device.default_input_config()
    }

    /// The default config used for output streams.
    pub fn default_output_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        self.device.default_output_config()
    }

    /// The maximum number of output channels of any format supported by this device.
    pub fn max_supported_output_channels(&self) -> usize {
        self.supported_output_configs()
            .expect("failed to get supported output audio stream formats")
            .map(|fmt| fmt.channels() as usize)
            .max()
            .unwrap_or(0)
    }

    /// The maximum number of input channels of any format supported by this device.
    pub fn max_supported_input_channels(&self) -> usize {
        self.supported_input_configs()
            .expect("failed to get supported input audio stream formats")
            .map(|fmt| fmt.channels() as usize)
            .max()
            .unwrap_or(0)
    }
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
