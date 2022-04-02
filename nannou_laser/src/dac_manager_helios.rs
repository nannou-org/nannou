use helios_dac::{NativeHeliosDacController};

use crate::dac_manager::{DetectDacs, Result, DetectedDacError};

/// An iterator yielding Helios DACs available on the system as they are discovered.
pub(crate) fn detect_dacs() -> Result<DetectDacs> {
    match NativeHeliosDacController::new(){
        Ok(helios_controller) => {
            let devices = helios_controller.list_devices()?;
            Ok(DetectDacs::Helios { previous_dac: devices[0].get_helios_constants()})
        },
        Err(e)=>Err(DetectedDacError::HeliosDacError(e))
    }
}