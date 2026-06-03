use bevy::prelude::*;

mod core;

fn main() {
    App::new()
        .add_plugins((
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
            core::CorePlugin,
        ))
        .run();
}
