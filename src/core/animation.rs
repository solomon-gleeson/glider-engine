use avian2d::prelude::*;
use bevy::prelude::*;

use super::assets::{EngineAssets, SpriteSheet};
use super::ecs::EngineState;
use super::physics::Player;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_player.run_if(in_state(EngineState::Running)),
        );
    }
}

const MOVE_THRESHOLD: f32 = 10.0;
const AIRBORNE_THRESHOLD: f32 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Walk,
    Jump,
}

#[derive(Component)]
pub struct PlayerAnimation {
    state: AnimationState,
    timer: Timer,
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self {
            state: AnimationState::Idle,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

const fn sheet_for(assets: &EngineAssets, state: AnimationState) -> &SpriteSheet {
    match state {
        AnimationState::Idle => &assets.idle,
        AnimationState::Walk => &assets.walk,
        AnimationState::Jump => &assets.jump,
    }
}

#[allow(clippy::needless_pass_by_value)]
fn animate_player(
    time: Res<Time>,
    assets: Res<EngineAssets>,
    player: Single<(&LinearVelocity, &mut Sprite, &mut PlayerAnimation), With<Player>>,
) {
    let (velocity, mut sprite, mut animation) = player.into_inner();

    let desired = if velocity.y.abs() > AIRBORNE_THRESHOLD {
        AnimationState::Jump
    } else if velocity.x.abs() > MOVE_THRESHOLD {
        AnimationState::Walk
    } else {
        AnimationState::Idle
    };

    if velocity.x.abs() > MOVE_THRESHOLD {
        sprite.flip_x = velocity.x < 0.0;
    }

    if desired != animation.state {
        animation.state = desired;
        animation.timer.reset();
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.layout = sheet_for(&assets, desired).layout.clone();
            atlas.index = 0;
        }
        sprite.image = sheet_for(&assets, desired).image.clone();
    }

    animation.timer.tick(time.delta());
    if animation.timer.just_finished() {
        let frames = sheet_for(&assets, animation.state).frames;
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = (atlas.index + 1) % frames;
        }
    }
}
