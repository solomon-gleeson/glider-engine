use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::prelude::*;

use super::editor_state::{EditorState, process_open};

pub fn editor_state_systems(
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if let Some(path) = state.request_open.take() {
        process_open(&mut state, &asset_server, &path);
    }

    if let Some(handle) = state.play_audio.take() {
        commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN));
    }
}
