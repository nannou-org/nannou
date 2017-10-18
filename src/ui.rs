pub extern crate conrod;

pub use conrod::widget;

use glium;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use window;

/// Owned by the `App`, the `Arrangement` handles the mapping between `Ui`s and their associated
/// windows.
pub struct Arrangement {
    windows: HashMap<window::Id, conrod::Ui>,
}

/// A handle to the `Ui` for a specific window.
pub struct Ui {
    /// The `Id` of the window upon which this `Ui` is instantiated.
    window: window::Id,
    ui: conrod::Ui,
}

impl Deref for Ui {
    type Output = conrod::Ui;
    fn deref(&self) -> &Self::Output {
        &self.ui
    }
}

impl Ui {
    pub fn generate_next_id(&mut self) -> widget::Id {
    }
}

/// A map from `image::Id`s to their associatd `Texture2d`.
pub type Texture2dMap = conrod::image::Map<glium::texture::Texture2d>;
