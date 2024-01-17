use bevy::prelude::*;

struct NannouPlugin;

impl Plugin for NannouPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_nannou_render::NannouRenderPlugin,
            bevy_nannou_draw::NannouDrawPlugin,
        ));
    }
}


#[cfg(test)]
mod tests {
    use bevy::app::App;

    #[test]
    fn it_works() {
        let mut app = App::new();
        app.add_plugins(super::NannouPlugin);
        app.update();
    }
}