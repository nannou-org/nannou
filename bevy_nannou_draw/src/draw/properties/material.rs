use bevy::prelude::Material;
use crate::render::NannouMaterialOptions;

pub trait SetMaterial<M: Material>: Sized {
    fn material_mut(&mut self) -> &mut M;
}