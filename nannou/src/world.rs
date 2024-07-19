use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use bevy_nannou::prelude::{Asset, Assets, AssetServer, Deref, DerefMut, Mut};
use crate::prelude::bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell;

#[derive(Deref, DerefMut)]
pub struct WorldAcess<'w>(pub UnsafeWorldCell<'w>);

impl <'w> WorldAcess<'w> {
    pub fn asset_server(&self) -> Mut<AssetServer> {
        unsafe {
            let world = self.world_mut();
            world.resource_mut::<AssetServer>()
        }
    }

    pub fn assets_mut<T: Asset>(&self) -> Mut<'_, Assets<T>> {
        unsafe {
            let world = self.world_mut();
            world.resource_mut::<Assets<T>>()
        }
    }

    pub fn assets<T: Asset>(&self) -> &Assets<T> {
        unsafe {
            let world = self.world_mut();
            world.resource::<Assets<T>>()
        }
    }
}

pub struct AssetAccessor<'w, T: Asset> {
    assets: Rc<RefCell<WorldAcess<'w>>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'w, T: Asset> AssetAccessor<'w, T> {
    pub(crate) fn new(assets: Rc<RefCell<WorldAcess<'w>>>) -> Self {
        Self {
            assets,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'w, T: Asset> Deref for AssetAccessor<'w, T> {
    type Target = Assets<T>;

    fn deref(&self) -> &Self::Target {
        &self.assets.borrow().assets::<T>()
    }
}

impl<'w, T: Asset> DerefMut for AssetAccessor<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.assets.borrow_mut().assets_mut::<T>().deref_mut()
    }
}

pub struct AssetServerAccessor<'w> {
    asset_server: Rc<RefCell<WorldAcess<'w>>>,
}

impl AssetServerAccessor<'_> {
    pub(crate) fn new(asset_server: Rc<RefCell<WorldAcess>>) -> Self {
        Self {
            asset_server,
        }
    }
}

impl Deref for AssetServerAccessor<'_> {
    type Target = AssetServer;

    fn deref(&self) -> &Self::Target {
        &self.asset_server.borrow().asset_server()
    }
}

impl DerefMut for AssetServerAccessor<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.asset_server.borrow_mut().asset_server().deref_mut()
    }
}

