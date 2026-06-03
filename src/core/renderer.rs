use crate::core::physics::Player;
use bevy::prelude::*;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
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
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation;

    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };
    let camera_pos = camera_transform.translation;

    let tracking_speed = 4.5;
    let delta = 1.0 - (-tracking_speed * time.delta_seconds()).exp();

    camera_transform.translation.x = camera_pos.x + (player_pos.x - camera_pos.x) * delta;
    camera_transform.translation.y = camera_pos.y + (player_pos.y - camera_pos.y) * delta;
}
