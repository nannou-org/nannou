//! Implementation of the general laser point type used throughout the crate.
use std::io;
use std::io::ErrorKind;
use thiserror::Error;
use helios_dac::{NativeHeliosError, NativeHeliosDac, NativeHeliosDacParams, NativeHeliosDacController};

use crate::dac_manager_helios;
use crate::{dac_base::DacBase, dac};
pub type Result<T> = std::result::Result<T, DetectedDacError>;

/// DAC discovery Iterators for all supported DACs available on the system.
pub(crate) fn detect_all_dacs() -> Vec<Result<DetectDacs>> {
    let dacs = vec![];
    
    let etherdream = dac::detect_dacs().map_err(|e|DetectedDacError::from(e));
    let helios = dac_manager_helios::detect_dacs();
    
    dacs.push(etherdream);
    dacs.push(helios);
    dacs
}
pub trait DacManager {
//     /// Return a detected DAC by DacId
//     fn detect_dac(&self, id: Id) -> Result<Box<dyn DetectedDac>>;
//     /// Function to detect DACs per DAC type
//     fn detect_dacs() -> Result<Vec<DacManagerData>>;
//     /// get map of DACs indexed by DacId
//     fn get_dac_map(&self) -> HashMap<&str, Box<dyn DacBase>>;  
}

/// Iterators yielding laser DACs available on the system as they are discovered.
pub enum DetectDacs {
    EtherDream{
        dac_broadcasts: ether_dream::RecvDacBroadcasts,
    },
    Helios{    
        previous_dac:NativeHeliosDacParams
    }
}

impl DetectDacs {
    /// Specify a duration for the detection to wait before timing out.
    pub fn set_timeout(&self, duration: Option<std::time::Duration>) -> io::Result<()> {
        if let DetectDacs::EtherDream { dac_broadcasts } = self {
            dac_broadcasts.set_timeout(duration)
        }else{
            // Helios DAC does not require this function
            Err(io::Error::new(ErrorKind::Other, "The Helios DAC does not implement the set_timeout function"))
        }
    }

    /// Specify whether or not retrieving the next DAC should block.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        if let DetectDacs::EtherDream { dac_broadcasts } = self {
            dac_broadcasts.set_nonblocking(nonblocking)
        }else{
            // Helios DAC does not require this function
            Err(io::Error::new(ErrorKind::Other, "The Helios DAC does not implement the set_nonblocking function"))
        }
    }
}

impl Iterator for DetectDacs {
    type Item = Result<DetectedDac>;
    fn next(&mut self) -> Option<Self::Item> {
        match self{
            DetectDacs::EtherDream { dac_broadcasts } => {
                let res = dac_broadcasts.next()?;
                match res {
                    Err(err) => Some(Err(err.into())),
                    Ok((broadcast, source_addr)) => Some(Ok(DetectedDac::EtherDream{
                        broadcast,
                        source_addr,
                    })),
                }
            },
            DetectDacs::Helios { previous_dac } => {
                match NativeHeliosDacController::new(){
                    Ok(res)=>{
                        match res.list_devices(){
                            Ok(dacs) =>{
                                match dacs.iter().position(|d|d.get_id() == previous_dac.id){
                                    Some(i) => Some(Ok(DetectedDac::Helios{dac: dacs.get(i+1)?.get_helios_constants()})),
                                    None => Some(Err(NativeHeliosError::InvalidDeviceResult.into()))
                                }
                            },
                            Err(e) => Some(Err(e.into()))
                        }
                    },
                    Err(e)=>Some(Err(e.into()))
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
    Helios{
        dac: NativeHeliosDacParams
    }
}

impl DetectedDac {
    /// The maximum point rate allowed by the DAC.
    pub fn max_point_hz(&self) -> u32 {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => broadcast.max_point_rate as _,
            DetectedDac::Helios {ref dac} => dac.max_point_rate as _ // NativeHeliosDac::get_helios_constants().max_point_rate as _
        }
    }

    /// The number of points that can be stored within the buffer.
    pub fn buffer_capacity(&self) -> u32 {
        match self {
            DetectedDac::EtherDream { ref broadcast, .. } => broadcast.buffer_capacity as _,
            DetectedDac::Helios {ref dac} => dac.buffer_capacity as _ //NativeHeliosDac::get_helios_constants().buffer_capacity as _
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
            DetectedDac::Helios {ref dac} => Id::Helios {
                id: dac.id
            }
        }
    }
}
/// A persistent, unique identifier associated with a DAC (like a MAC address for Etherdream).
///
/// It should be possible to use this to uniquely identify the same DAC on different occasions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Id {
    EtherDream { mac_address: [u8; 6] },
    Helios {id: u32}
}


// #[derive(Clone,Debug)]
// pub struct DacManagerData{
//     /// The unique hardware identifier for the DAC.
//     pub id: Id,
//     /// The DAC's maximum point rate.
//     ///
//     /// As of writing this, this is hardcoded to `0xFFFF` in the original DAC source code.
//     pub max_point_rate:u32,
//     /// The DAC's maximum buffer capacity for storing points that are yet to be converted to
//     /// output.
//     ///
//     /// As of writing this, this is hardcoded to `HELIOS_MAX_POINTS (0x1000) * 7 + 5` in the original DAC source code.
//     pub buffer_capacity: u32,
// }

#[derive(Error, Debug)]
pub enum DetectedDacError {
    #[error("Helios_dac error: {0}")]
    HeliosDacError(#[from] NativeHeliosError),
    #[error("EtherDream DAC IO detection error: {0}")]
    IoError(#[from] io::Error),
}