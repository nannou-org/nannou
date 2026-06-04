use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct MidiPortAdded {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub struct MidiPortRemoved {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub struct MidiConnected {
    pub entity: Entity,
}

#[derive(EntityEvent)]
pub struct MidiDisconnected {
    pub entity: Entity,
    pub reason: String,
}
