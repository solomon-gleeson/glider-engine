use bevy::prelude::*;
use crate::core::physics::Player;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        // Keep the dark grey background from our previous step
        app.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                camera_follow.after(bevy::transform::TransformSystem::TransformPropagate),
            );
    }
}

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    // 1. Get the player's position
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    // 2. Get the camera's position
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };
    let camera_pos = camera_transform.translation;

    // 3. Define the tracking speed coefficient (higher = snappier)
    let tracking_speed = 4.5;
    let delta = 1.0 - (-tracking_speed * time.delta_seconds()).exp();

    // 4. Interpolate X and Y, keeping Z fixed (so 2D layering remains intact)
    camera_transform.translation.x = camera_pos.x + (player_pos.x - camera_pos.x) * delta;
    camera_transform.translation.y = camera_pos.y + (player_pos.y - camera_pos.y) * delta;
}
