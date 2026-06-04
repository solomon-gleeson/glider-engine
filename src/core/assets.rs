use bevy::asset::LoadState;
use bevy::prelude::*;

use super::ecs::EngineState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(EngineState::Loading), begin_loading)
            .add_systems(
                Update,
                check_loading_progress.run_if(in_state(EngineState::Loading)),
            );
    }
}

#[derive(Clone)]
pub struct SpriteSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub frames: usize,
}

#[derive(Resource)]
pub struct EngineAssets {
    pub idle: SpriteSheet,
    pub walk: SpriteSheet,
    pub jump: SpriteSheet,
}

fn begin_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let frame = UVec2::splat(128);
    let mut load = |path: &'static str, frames: u32| SpriteSheet {
        image: asset_server.load(path),
        layout: layouts.add(TextureAtlasLayout::from_grid(frame, frames, 1, None, None)),
        frames: frames as usize,
    };

    commands.insert_resource(EngineAssets {
        idle: load("textures/Idle.png", 6),
        walk: load("textures/Walk.png", 8),
        jump: load("textures/Jump.png", 10),
    });
}

fn check_loading_progress(
    asset_server: Res<AssetServer>,
    engine_assets: Res<EngineAssets>,
    mut next_state: ResMut<NextState<EngineState>>,
) {
    let images = [
        &engine_assets.idle.image,
        &engine_assets.walk.image,
        &engine_assets.jump.image,
    ];

    let mut all_loaded = true;
    for image in images {
        match asset_server.get_load_state(image) {
            Some(LoadState::Loaded) => {}
            Some(LoadState::Failed(error)) => {
                error!("Failed to load the requested asset: {:?}", error);
                next_state.set(EngineState::Running);
                return;
            }
            _ => all_loaded = false,
        }
    }

    if all_loaded {
        info!("Assets successfully loaded into memory.");
        next_state.set(EngineState::Running);
    }
}
