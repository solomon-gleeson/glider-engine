#![allow(dead_code)]

use bevy::asset::AssetId;
use bevy::prelude::*;

const EDITOR_FONT_DATA: &[u8] = include_bytes!("../../assets/editor/fonts/AdwaitaSans-Regular.ttf");

pub fn setup_editor_font(mut fonts: ResMut<Assets<Font>>) {
    let font = Font::from_bytes(EDITOR_FONT_DATA.to_vec());
    fonts.insert(AssetId::default(), font).ok();
}

#[derive(Resource)]
pub struct EditorFonts {
    pub mono: Handle<Font>,
}

#[derive(Resource)]
pub struct EditorIcons {
    pub play: Handle<Image>,
    pub stop: Handle<Image>,
    pub settings: Handle<Image>,
    pub output: Handle<Image>,
    pub properties: Handle<Image>,
    pub explorer: Handle<Image>,
    pub file: Handle<Image>,
    pub lock: Handle<Image>,
    pub toolbox: Handle<Image>,
    pub snap: Handle<Image>,
    pub folder: Handle<Image>,
    pub workspace: Handle<Image>,
    pub replicated_storage: Handle<Image>,
    pub part: Handle<Image>,
    pub script: Handle<Image>,
    pub model: Handle<Image>,
    pub sound: Handle<Image>,
    pub decal: Handle<Image>,
    pub server_script_service: Handle<Image>,
    pub starter_player: Handle<Image>,
    pub starter_gui: Handle<Image>,
    pub sound_service: Handle<Image>,
    pub lighting: Handle<Image>,
    pub players: Handle<Image>,
    pub audio: Handle<Image>,
}

macro_rules! load_one {
    ($asset_server:ident, $path:expr) => {{ $asset_server.load($path) }};
}

pub fn setup_editor_icons(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icons = EditorIcons {
        play: load_one!(asset_server, "editor/icons/UI/Play.png"),
        stop: load_one!(asset_server, "editor/icons/UI/Stop.png"),
        settings: load_one!(asset_server, "editor/icons/UI/Settings.png"),
        output: load_one!(asset_server, "editor/icons/UI/Output.png"),
        properties: load_one!(asset_server, "editor/icons/UI/Properties.png"),
        explorer: load_one!(asset_server, "editor/icons/UI/Explorer.png"),
        file: load_one!(asset_server, "editor/icons/UI/File.png"),
        lock: load_one!(asset_server, "editor/icons/UI/Lock.png"),
        toolbox: load_one!(asset_server, "editor/icons/UI/Toolbox.png"),
        snap: load_one!(asset_server, "editor/icons/UI/Snap.png"),
        folder: load_one!(asset_server, "editor/icons/UI/Folder.png"),
        workspace: load_one!(asset_server, "editor/icons/Service/Workspace.png"),
        replicated_storage: load_one!(asset_server, "editor/icons/Service/ReplicatedStorage.png"),
        part: load_one!(asset_server, "editor/icons/Part/Part.png"),
        script: load_one!(asset_server, "editor/icons/Script/Script.png"),
        model: load_one!(asset_server, "editor/icons/Model/Model.png"),
        sound: load_one!(asset_server, "editor/icons/Audio/Sound.png"),
        decal: load_one!(asset_server, "editor/icons/UI/Decal.png"),
        server_script_service: load_one!(
            asset_server,
            "editor/icons/Service/ServerScriptService.png"
        ),
        starter_player: load_one!(asset_server, "editor/icons/Service/StarterPlayer.png"),
        starter_gui: load_one!(asset_server, "editor/icons/Service/StarterGui.png"),
        sound_service: load_one!(asset_server, "editor/icons/Service/SoundService.png"),
        lighting: load_one!(asset_server, "editor/icons/Service/Lighting.png"),
        players: load_one!(asset_server, "editor/icons/Service/Players.png"),
        audio: load_one!(asset_server, "editor/icons/Service/Audio.png"),
    };

    commands.insert_resource(icons);

    commands.insert_resource(EditorFonts {
        mono: asset_server.load("editor/fonts/JetBrainsMono-Regular.otf"),
    });
}
