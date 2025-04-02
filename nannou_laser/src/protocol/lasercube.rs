// Represent lasercube device IDs by their serial number.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id {
    pub serial_number: [u8; 6],
}

/// A lasercube discovered via a `GET_FULL_INFO` response.
#[derive(Clone, Debug)]
pub struct DetectedDac {
    info: lasercube::core::LaserInfo,
    source_addr: std::net::SocketAddr,
}

impl DetectedDac {
    pub fn max_point_hz(&self) -> u32 {
        self.info.header.max_dac_rate
    }

    pub fn buffer_capacity(&self) -> u32 {
        self.info.header.rx_buffer_size as u32
    }

    pub fn id(&self) -> Id {
        Id {
            serial_number: self.info.header.serial_number,
        }
    }
}
