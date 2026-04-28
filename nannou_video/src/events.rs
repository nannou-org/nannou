use bevy::prelude::*;

#[derive(EntityEvent, Debug, Clone)]
pub struct VideoLoaded {
    pub entity: Entity,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct VideoEnded {
    pub entity: Entity,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct VideoLooped {
    pub entity: Entity,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct VideoFailed {
    pub entity: Entity,
    pub reason: String,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct VideoSeeked {
    pub entity: Entity,
    pub to_seconds: f64,
}
