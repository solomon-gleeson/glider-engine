pub mod assets;
pub mod ecs;
pub mod input;
pub mod physics;
pub mod renderer;
pub mod scripting;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ecs::EcsPlugin,
            assets::AssetsPlugin,
            physics::PhysicsPlugin,
            input::InputPlugin,
            scripting::ScriptingPlugin,
            renderer::RendererPlugin,
        ));
    }
}
