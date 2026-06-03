
use bevy::prelude::*;
use bevy::asset::LoadState;

use super::ecs::EngineState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(EngineState::Loading), begin_loading)
            .add_systems(Update, check_loading_progress.run_if(in_state(EngineState::Loading)));
    }
}

#[derive(Resource)]
pub struct EngineAssets {
    pub placeholder_texture: Handle<Image>,
}

fn begin_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let engine_assets = EngineAssets {
        placeholder_texture: asset_server.load("textures/player.png"),
    };

    commands.insert_resource(engine_assets);
}

fn check_loading_progress(
    asset_server: Res<AssetServer>,
    engine_assets: Res<EngineAssets>,
    mut next_state: ResMut<NextState<EngineState>>,
) {
    if let Some(load_state) = asset_server.get_load_state(&engine_assets.placeholder_texture) {
        match load_state {
            LoadState::Loaded => {
                info!("Assets successfully loaded into memory.");
                next_state.set(EngineState::Running);
            }
            LoadState::Failed(error) => {
                error!("Failed to load the requested asset: {:?}", error);
                // Fallback to Running even if assets fail, to prevent system lockup from log spam
                next_state.set(EngineState::Running);
            }
            _ => {}
        }
    }
}
