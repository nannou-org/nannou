//! Items related specifically to the Helios DAC.
use helios_dac::{NativeHeliosDacController, NativeHeliosError};
use thiserror::Error;

use crate::{dac_manager::{DetectDacs, DetectedDacError, Result}, RawPoint, util::{clamp, map_range}};

/// An iterator yielding Helios DACs available on the system as they are discovered.
pub(crate) fn detect_dacs() -> Result<DetectDacs> {
    let helios_controller = NativeHeliosDacController::new().map_err(DetectedDacError::from)?;
    let devices = helios_controller.list_devices().map_err(DetectedDacError::from)?;
    if !devices.is_empty() {
        Ok(DetectDacs::Helios { previous_dac: devices.get(0).unwrap().0.into()})
    }else{
        println!("No Helios DACs found");
        Err(NativeHeliosError::InvalidDeviceResult.into())
    }
}

pub fn position_to_helios_coordinate([px, py]: crate::point::Position) -> helios_dac::Coordinate {
    let min = 0;
    let max = 0xFFF;
    helios_dac::Coordinate {
        x:map_range(clamp(px, -1.0, 1.0), -1.0, 1.0, min as f64, max as f64) as u16,
        y:map_range(clamp(py, -1.0, 1.0), -1.0, 1.0, min as f64, max as f64) as u16
    }
}

pub fn color_to_helios_color([pr, pg, pb]: crate::point::Rgb) -> helios_dac::Color{
    helios_dac::Color { 
        r: (clamp(pr, 0.0, 1.0) * u8::MAX as f32) as u8,
        g: (clamp(pg, 0.0, 1.0) * u8::MAX as f32) as u8,
        b: (clamp(pb, 0.0, 1.0) * u8::MAX as f32) as u8
    }
}

pub fn point_to_helios_point(p: RawPoint) -> helios_dac::Point {
    helios_dac::Point{
       coordinate:position_to_helios_coordinate(p.position),
       color:color_to_helios_color(p.color),
       intensity: 0xFF // TODO: enable user to change intensity of point
    } 
}

/// Errors that may occur while creating a node crate.
#[derive(Debug, Error)]
pub enum HeliosStreamError {
    #[error("Failed to create USB context before detecting DACs: {err}")]
    FailedToCreateUSBContext{
        #[source]
        err: NativeHeliosError,
        attempts:u32
    },
    #[error("Laser DAC detection failed (attempt {attempts}): {err}")]
    FailedToDetectDacs {
        #[source]
        err: NativeHeliosError,
        /// The number of DAC detection attempts so far.
        attempts: u32,
    },
    #[error("{err}")]
    InvalidDeviceResult {
        #[source]
        err: NativeHeliosError,
    },
    #[error("Error when writing frame: {err}")]
    FailedToWriteFrame {
        #[source]
        err: NativeHeliosError,
    },
    #[error("Failed to submit stop command to the DAC: {err}")]
    FailedToStopStream {
        #[source]
        err: NativeHeliosError
    },
}