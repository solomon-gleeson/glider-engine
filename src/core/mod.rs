// src/core/mod.rs

pub mod ecs;
pub mod assets;
pub mod physics;
pub mod input;
pub mod scripting; // Expose module

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ecs::EcsPlugin,
            assets::AssetsPlugin,
            physics::PhysicsPlugin,
            input::InputPlugin,
            scripting::ScriptingPlugin, // Attach the high-speed engine VM
        ));
    }
}
