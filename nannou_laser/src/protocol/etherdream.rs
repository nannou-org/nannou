// Represent etherdream device IDs by their MAC address.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id {
    pub mac_address: [u8; 6],
}

/// An ether dream laser DAC discovered via the ether dream protocol broadcast
/// message.
#[derive(Clone, Debug)]
pub struct DetectedDac {
    pub broadcast: ether_dream::protocol::DacBroadcast,
    pub source_addr: std::net::SocketAddr,
}

impl DetectedDac {
    pub fn max_point_hz(&self) -> u32 {
        self.broadcast.max_point_rate as u32
    }

    pub fn buffer_capacity(&self) -> u32 {
        self.broadcast.buffer_capacity as u32
    }

    pub fn id(&self) -> Id {
        Id {
            mac_address: self.broadcast.mac_address,
        }
    }
}
