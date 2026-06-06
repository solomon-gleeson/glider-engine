use avian2d::prelude::*;
use bevy::prelude::*;

use super::ecs::EngineState;
use super::physics::Player;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            player_movement.run_if(in_state(EngineState::Running)),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut LinearVelocity, &Transform), With<Player>>,
    spatial_query: SpatialQuery,
) {
    let speed = 300.0;
    let jump_impulse = 600.0;

    for (entity, mut velocity, transform) in &mut query {
        let ray_origin = transform.translation.truncate();
        let ray_direction = Dir2::NEG_Y;
        let max_distance = 26.0;
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let is_grounded = spatial_query
            .cast_ray(ray_origin, ray_direction, max_distance, true, &filter)
            .is_some();

        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            velocity.x = -speed;
        } else if keyboard_input.pressed(KeyCode::KeyD)
            || keyboard_input.pressed(KeyCode::ArrowRight)
        {
            velocity.x = speed;
        } else {
            velocity.x = 0.0;
        }

        if is_grounded
            && (keyboard_input.just_pressed(KeyCode::Space)
                || keyboard_input.just_pressed(KeyCode::KeyW))
        {
            velocity.y = jump_impulse;
        }
    }
}
