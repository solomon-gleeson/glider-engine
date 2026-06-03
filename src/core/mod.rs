use bevy::prelude::*;

pub mod assets;
pub mod ecs;
pub mod renderer;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            assets::AssetsPlugin,
            ecs::EcsPlugin,
            renderer::RendererPlugin,
        ));
    }
}
