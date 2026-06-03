#![forbid(unsafe_code)]

use bevy::prelude::*;

mod core;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Glider Engine"),
                resolution: (1280.0_f32, 720.0_f32).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(core::CorePlugin)
        .run();
}
