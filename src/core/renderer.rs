use crate::core::physics::Player;
use bevy::prelude::*;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
            .add_systems(Startup, spawn_camera)
            .add_systems(FixedUpdate, camera_follow);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera, IsDefaultUiCamera));
}

#[allow(clippy::needless_pass_by_value)]
fn camera_follow(
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera: Single<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let player_pos = player.translation;
    let camera_pos = camera.translation;

    let tracking_speed = 4.5;
    let delta = 1.0 - (-tracking_speed * time.delta_secs()).exp();

    camera.translation.x = (player_pos.x - camera_pos.x).mul_add(delta, camera_pos.x);
    camera.translation.y = (player_pos.y - camera_pos.y).mul_add(delta, camera_pos.y);
}
