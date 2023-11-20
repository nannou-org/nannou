//! DAC detection items shared among all supported DACs

use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DetectedDacError>;

/// Iterators yielding laser DACs available on the system as they are discovered.
pub enum DetectDacs {
    EtherDream {
        dac_broadcasts: ether_dream::RecvDacBroadcasts,
    },
    Helios {
        previous_dac: helios_dac::NativeHeliosDacParams,
    },
}

impl DetectDacs {
    /// Specify a duration for the detection to wait before timing out.
    pub fn set_timeout(&self, duration: Option<std::time::Duration>) -> io::Result<()> {
        match self {
            DetectDacs::EtherDream { dac_broadcasts } => dac_broadcasts.set_timeout(duration),
            // Helios DAC does not require this function
            DetectDacs::Helios { .. } => Ok(()),
        }
    }

    /// Specify whether or not retrieving the next DAC should block.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        match self {
            DetectDacs::EtherDream { dac_broadcasts } => {
                dac_broadcasts.set_nonblocking(nonblocking)
            }
            // Helios DAC does not require this function
            DetectDacs::Helios { .. } => Ok(()),
        }
    }
}

impl Iterator for DetectDacs {
    type Item = Result<DetectedDac>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DetectDacs::EtherDream { dac_broadcasts } => {
                let res = dac_broadcasts.next()?;
                match res {
                    Err(err) => Some(Err(err.into())),
                    Ok((broadcast, source_addr)) => Some(Ok(DetectedDac::EtherDream {
                        broadcast,
                        source_addr,
                    })),
                }
            }
            DetectDacs::Helios { previous_dac } => {
                match helios_dac::NativeHeliosDacController::new() {
                    Ok(res) => {
                        match res.list_devices() {
                            Ok(dacs) => {
                                let current_position =
                                    dacs.iter().position(|(id, ..)| *id == previous_dac.id);
                                if let Some(pos) = current_position {
                                    if let Some((new_id, ..)) = dacs.get(pos + 1) {
                                        Some(Ok(DetectedDac::Helios {
                                            dac: (*new_id).into(),
                                        }))
                                    } else {
                                        // Reached end of list of detected Helios DACs
                                        // Return the first DAC in list:
                                        Some(Ok(DetectedDac::Helios {
                                            dac: dacs.get(0)?.0.into(),
                                        }))
                                    }
                                } else {
                                    Some(Err(
                                        helios_dac::NativeHeliosError::InvalidDeviceResult.into()
                                    ))
                                }
                            }
                            Err(e) => Some(Err(e.into())),
                        }
                    }
                    Err(e) => Some(Err(e.into())),
                }
            }
        }
    }
}
/// An available DAC detected on the system.
#[derive(Clone, Debug)]
pub enum DetectedDac {
    /// An ether dream laser DAC discovered via the ether dream protocol broadcast message.
    EtherDream {
        broadcast: ether_dream::protocol::DacBroadcast,
        source_addr: std::net::SocketAddr,
    },
    Helios {
        dac: helios_dac::NativeHeliosDacParams,
    },
}

impl From<helios_dac::NativeHeliosDacParams> for DetectedDac {
    fn from(dac: helios_dac::NativeHeliosDacParams) -> Self {
        DetectedDac::Helios { dac }
    }
}

impl DetectedDac {
    /// The maximum point rate allowed by the DAC.
    pub fn max_point_hz(&self) -> u32 {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => broadcast.max_point_rate as _,
            DetectedDac::Helios { ref dac } => dac.max_point_rate as _,
        }
    }

    /// The number of points that can be stored within the buffer.
    pub fn buffer_capacity(&self) -> u32 {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => broadcast.buffer_capacity as _,
            DetectedDac::Helios { ref dac } => dac.buffer_capacity as _,
        }
    }

    /// A persistent, unique identifier associated with the DAC (like a MAC address).
    ///
    /// It should be possible to use this to uniquely identify the same DAC on different occasions.
    pub fn id(&self) -> Id {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => Id::EtherDream {
                mac_address: broadcast.mac_address,
            },
            DetectedDac::Helios { ref dac } => Id::Helios { id: dac.id },
        }
    }
}
/// A persistent, unique identifier associated with a DAC (like a MAC address for Etherdream).
///
/// It should be possible to use this to uniquely identify the same DAC on different occasions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    EtherDream { mac_address: [u8; 6] },
    Helios { id: u32 },
}

#[derive(Clone, Debug, Copy)]
pub enum DacVariant {
    DacVariantEtherdream,
    DacVariantHelios,
}

impl Default for DacVariant {
    fn default() -> Self {
        DacVariant::DacVariantEtherdream
    }
}

#[derive(Error, Debug)]
pub enum DetectedDacError {
    #[error("Helios_dac error: {0}")]
    HeliosDacError(#[from] helios_dac::NativeHeliosError),
    #[error("EtherDream DAC IO detection error: {0}")]
    IoError(#[from] io::Error),
}
