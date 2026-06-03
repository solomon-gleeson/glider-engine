use avian2d::prelude::*;
use bevy::prelude::*;

use super::assets::EngineAssets;
use super::ecs::EngineState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::new(0.0, -980.0)))
            .add_systems(OnEnter(EngineState::Running), setup_physics_test);
    }
}

#[derive(Component)]
pub struct Player;

fn setup_physics_test(mut commands: Commands, assets: Res<EngineAssets>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.3, 0.5, 0.3),
                custom_size: Some(Vec2::new(800.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(800.0, 50.0),
    ));

    commands.spawn((
        SpriteBundle {
            texture: assets.placeholder_texture.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 200.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::rectangle(50.0, 50.0),
        LockedAxes::ROTATION_LOCKED,
        Player,
    ));
}
