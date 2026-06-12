#[cfg(feature = "render")]
pub mod animation;
#[cfg(feature = "render")]
pub mod assets;
pub mod ecs;

#[cfg(feature = "render")]
pub mod input;

#[cfg(feature = "render")]
pub mod physics;

#[cfg(feature = "render")]
pub mod renderer;

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
            bevy_luau::ScriptingPlugin,
            #[cfg(feature = "render")]
            renderer::RendererPlugin,
            #[cfg(feature = "render")]
            animation::AnimationPlugin,
        ));
    }
}

pub fn run() {
    App::new()
        .add_plugins((
            crate::instance::InstancePlugin,
            #[cfg(feature = "render")]
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Glider Engine"),
                    resolution: (1280, 720).into(),
                    present_mode: bevy::window::PresentMode::Fifo,
                    ..default()
                }),
                ..default()
            }),
            CorePlugin,
            #[cfg(feature = "editor")]
            crate::editor::EditorPlugin,
        ))
        .run();
}
