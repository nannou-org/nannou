use bevy::prelude::Material;

pub trait SetMaterial<M>: Sized
    where M: Material + Default
{
    fn material_mut(&mut self) -> &mut M;
}