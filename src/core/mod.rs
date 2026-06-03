#[cfg(feature = "render")]
pub mod assets;
pub mod ecs;

#[cfg(feature = "render")]
pub mod input;

#[cfg(feature = "render")]
pub mod physics;

#[cfg(feature = "render")]
pub mod renderer;
pub mod scripting;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ecs::EcsPlugin,
            #[cfg(feature = "render")]
            assets::AssetsPlugin,
            #[cfg(feature = "render")]
            physics::PhysicsPlugin,
            #[cfg(feature = "render")]
            input::InputPlugin,
            scripting::ScriptingPlugin,
            #[cfg(feature = "render")]
            renderer::RendererPlugin,
        ));
    }
}
