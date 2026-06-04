use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct WebcamDeviceAdded {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub struct WebcamDeviceRemoved {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub struct WebcamConnected {
    pub entity: Entity,
    pub resolution: UVec2,
    pub framerate: u32,
}

#[derive(EntityEvent)]
pub struct WebcamDisconnected {
    pub entity: Entity,
    pub reason: String,
}
