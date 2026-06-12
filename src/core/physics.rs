use std::collections::HashMap;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::animation::PlayerAnimation;
use super::assets::EngineAssets;
use super::ecs::EngineState;
use crate::instance::{Instance, NextInstanceId, WorkspaceRoot};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::new(0.0, -980.0)))
            .init_resource::<EditTransforms>()
            
            
            .add_systems(OnEnter(EngineState::Editing), spawn_world_parts)
            
            
            .add_systems(OnEnter(EngineState::Running), enter_play_physics)
            .add_systems(OnExit(EngineState::Running), exit_play_physics);
    }
}


#[derive(Component)]
pub struct Player;


#[derive(Component)]
struct GroundPart;

#[derive(Component)]
struct PlayerPart;


#[derive(Resource, Default)]
struct EditTransforms(HashMap<Entity, Transform>);

const PLAYER_SIZE: f32 = 50.0;
const PLAYER_SPRITE_SCALE: f32 = 192.0;
const GROUND_W: f32 = 800.0;
const GROUND_H: f32 = 50.0;

fn spawn_world_parts(world: &mut World) {
    
    if world
        .query_filtered::<(), With<PlayerPart>>()
        .iter(world)
        .next()
        .is_some()
    {
        return;
    }
    let Some(workspace) = world.resource::<WorkspaceRoot>().0 else {
        return;
    };
    let idle = world.resource::<EngineAssets>().idle.clone();

    let ground_id = world.resource_mut::<NextInstanceId>().allocate();
    let ground = world
        .spawn((
            Instance {
                id: ground_id,
                name: "Baseplate".into(),
                class_name: "Part".into(),
                parent: Some(workspace),
                children: Vec::new(),
            },
            Sprite {
                color: Color::srgb(0.3, 0.5, 0.3),
                custom_size: Some(Vec2::new(GROUND_W, GROUND_H)),
                ..default()
            },
            Transform::from_xyz(0.0, -200.0, 0.0),
            GroundPart,
        ))
        .id();

    let player_id = world.resource_mut::<NextInstanceId>().allocate();
    let player = world
        .spawn((
            Instance {
                id: player_id,
                name: "Player".into(),
                class_name: "Part".into(),
                parent: Some(workspace),
                children: Vec::new(),
            },
            Sprite {
                image: idle.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: idle.layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::splat(PLAYER_SPRITE_SCALE)),
                ..default()
            },
            Anchor(Vec2::new(
                0.0,
                (PLAYER_SIZE * 0.5) / PLAYER_SPRITE_SCALE - 0.5,
            )),
            Transform::from_xyz(0.0, 200.0, 0.0),
            PlayerPart,
        ))
        .id();

    if let Some(mut ws) = world.entity_mut(workspace).get_mut::<Instance>() {
        ws.children.push(ground);
        ws.children.push(player);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn enter_play_physics(
    mut commands: Commands,
    mut edit_transforms: ResMut<EditTransforms>,
    grounds: Query<(Entity, &Sprite), With<GroundPart>>,
    players: Query<(Entity, &Transform), With<PlayerPart>>,
) {
    edit_transforms.0.clear();

    for (entity, sprite) in &grounds {
        let size = sprite.custom_size.unwrap_or(Vec2::new(GROUND_W, GROUND_H));
        commands
            .entity(entity)
            .insert((RigidBody::Static, Collider::rectangle(size.x, size.y)));
    }

    for (entity, transform) in &players {
        edit_transforms.0.insert(entity, *transform);
        commands.entity(entity).insert((
            RigidBody::Dynamic,
            Collider::rectangle(PLAYER_SIZE, PLAYER_SIZE),
            LockedAxes::ROTATION_LOCKED,
            Player,
            PlayerAnimation::default(),
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn exit_play_physics(
    mut commands: Commands,
    assets: Res<EngineAssets>,
    mut edit_transforms: ResMut<EditTransforms>,
    grounds: Query<Entity, With<GroundPart>>,
    players: Query<Entity, With<PlayerPart>>,
    mut sprites: Query<&mut Sprite>,
    mut transforms: Query<&mut Transform>,
) {
    for entity in &grounds {
        commands.entity(entity).remove::<(RigidBody, Collider)>();
    }

    for entity in &players {
        commands.entity(entity).remove::<(
            RigidBody,
            Collider,
            LockedAxes,
            LinearVelocity,
            Player,
            PlayerAnimation,
        )>();

        
        if let Some(transform) = edit_transforms.0.get(&entity).copied()
            && let Ok(mut tr) = transforms.get_mut(entity)
        {
            *tr = transform;
        }
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            sprite.image = assets.idle.image.clone();
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.layout = assets.idle.layout.clone();
                atlas.index = 0;
            }
        }
    }

    edit_transforms.0.clear();
}
