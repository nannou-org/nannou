use crate::render::NannouMaterialOptions;

pub trait SetMaterial: Sized {
    fn material_mut(&mut self) -> &mut NannouMaterialOptions;
}