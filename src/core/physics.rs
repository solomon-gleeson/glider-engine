use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::animation::PlayerAnimation;
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

const PLAYER_SIZE: f32 = 50.0;
const PLAYER_SPRITE_SCALE: f32 = 192.0;

#[allow(clippy::needless_pass_by_value)]
fn setup_physics_test(mut commands: Commands, assets: Res<EngineAssets>) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.5, 0.3),
            custom_size: Some(Vec2::new(800.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -200.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(800.0, 50.0),
    ));

    commands.spawn((
        Sprite {
            image: assets.idle.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.idle.layout.clone(),
                index: 0,
            }),
            custom_size: Some(Vec2::splat(PLAYER_SPRITE_SCALE)),
            ..default()
        },
        Anchor(Vec2::new(0.0, (PLAYER_SIZE * 0.5) / PLAYER_SPRITE_SCALE - 0.5)),
        Transform::from_xyz(0.0, 200.0, 0.0),
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_SIZE, PLAYER_SIZE),
        LockedAxes::ROTATION_LOCKED,
        Player,
        PlayerAnimation::default(),
    ));
}
