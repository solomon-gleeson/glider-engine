use super::dock::{DockSlot, DockTab};
use crate::instance::ScriptSource;
use bevy::asset::AssetServer;
use bevy::prelude::*;


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewportToolMode {
    Select = 0,
    Move = 1,
    Rotate = 2,
    Scale = 3,
}

impl ViewportToolMode {
    
    
    pub fn icon(&self) -> &str {
        match self {
            Self::Select => "\u{2196}",
            Self::Move => "\u{2295}",
            Self::Rotate => "\u{21BB}",
            Self::Scale => "\u{25C7}",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Select => "Select",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::Scale => "Scale",
        }
    }
}

#[derive(Clone)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub children: Vec<FileTreeNode>,
    pub is_dir: bool,
}

pub enum FileContent {
    Text(String),
    Image { handle: Handle<Image> },
    Audio { handle: Handle<AudioSource> },
}

pub struct OpenFile {
    pub path: String,
    pub name: String,
    pub content: FileContent,
    pub modified: bool,
}

#[derive(Resource)]
pub struct EditorState {
    pub open_files: Vec<OpenFile>,
    pub active_tab: Option<usize>,
    pub request_open: Option<String>,
    pub file_tree: Vec<FileTreeNode>,
    pub collapsed: Vec<String>,
    pub selected_entity: Option<Entity>,
    pub selected_service: Option<String>,
    pub synced_entity: Option<Entity>,
    pub left_scene_dock: crate::editor::dock::DockSlot,
    pub left_files_dock: crate::editor::dock::DockSlot,
    pub left_split_ratio: f32,
    pub right_dock: crate::editor::dock::DockSlot,
    pub bottom_dock: crate::editor::dock::DockSlot,
    pub pos_x: f32,
    pub pos_y: f32,
    pub drag_pointer_origin: Option<Vec2>,
    pub drag_obj_origin: Option<Vec2>,
    pub drag_screen_origin: Option<Vec2>,
    pub drag_rot_origin: Option<f32>,
    pub drag_size_origin: Option<Vec2>,
    pub size_w: f32,
    pub size_h: f32,
    pub rotation: f32,
    pub color: Color,
    pub build_requested: bool,
    pub play_audio: Option<Handle<AudioSource>>,
    pub viewport_zoom: f32,
    pub viewport_tool_mode: ViewportToolMode,
    pub grid_size: f32,
    
    
    
    
    pub project_panel_content: Option<Entity>,
}

impl Default for EditorState {
    fn default() -> Self {
        let file_tree = build_tree("assets");
        Self {
            open_files: Vec::new(),
            active_tab: None,
            request_open: None,
            file_tree,
            collapsed: Vec::new(),
            selected_entity: None,
            selected_service: None,
            synced_entity: None,
            left_scene_dock: DockSlot::from_single("Scene", "scene", "Scene"),
            left_files_dock: DockSlot::from_single("FileSystem", "filesystem", "FileSystem"),
            left_split_ratio: 0.55,
            right_dock: DockSlot::new(
                "Inspector",
                vec![DockTab {
                    id: "inspector".into(),
                    name: "Inspector".into(),
                    closeable: false,
                }],
            ),
            bottom_dock: DockSlot::new(
                "Bottom",
                vec![
                    DockTab {
                        id: "output".into(),
                        name: "Output".into(),
                        closeable: false,
                    },
                    DockTab {
                        id: "debugger".into(),
                        name: "Debugger".into(),
                        closeable: false,
                    },
                    DockTab {
                        id: "animation".into(),
                        name: "Animation".into(),
                        closeable: false,
                    },
                    DockTab {
                        id: "console".into(),
                        name: "Console".into(),
                        closeable: false,
                    },
                ],
            ),
            pos_x: 0.0,
            pos_y: 0.0,
            drag_pointer_origin: None,
            drag_obj_origin: None,
            drag_screen_origin: None,
            drag_rot_origin: None,
            drag_size_origin: None,
            size_w: 100.0,
            size_h: 100.0,
            rotation: 0.0,
            color: Color::WHITE,
            build_requested: false,
            play_audio: None,
            viewport_zoom: 1.0,
            viewport_tool_mode: ViewportToolMode::Select,
            grid_size: 50.0,
            project_panel_content: None,
        }
    }
}

fn build_tree(dir: &str) -> Vec<FileTreeNode> {
    let mut nodes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut dirs: Vec<_> = Vec::new();
        let mut files: Vec<_> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                let children = build_tree(&path.to_string_lossy());
                dirs.push(FileTreeNode {
                    name,
                    path: path.to_string_lossy().to_string(),
                    children,
                    is_dir: true,
                });
            } else if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_ascii_lowercase();
                if matches!(
                    ext.as_str(),
                    "luau"
                        | "lua"
                        | "png"
                        | "jpg"
                        | "jpeg"
                        | "wav"
                        | "ogg"
                        | "mp3"
                        | "flac"
                        | "ron"
                        | "txt"
                        | "md"
                        | "json"
                        | "toml"
                ) {
                    files.push(FileTreeNode {
                        name,
                        path: path.to_string_lossy().to_string(),
                        children: Vec::new(),
                        is_dir: false,
                    });
                }
            }
        }
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));
        nodes.extend(dirs);
        nodes.extend(files);
    }
    nodes
}

pub fn is_image_path(path: &str) -> bool {
    matches!(
        std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("png" | "jpg" | "jpeg")
    )
}

pub fn is_audio_path(path: &str) -> bool {
    matches!(
        std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("mp3" | "wav" | "ogg" | "flac")
    )
}

pub fn process_open(state: &mut EditorState, asset_server: &AssetServer, path: &str) {
    if let Some(idx) = state.open_files.iter().position(|f| f.path == path) {
        state.active_tab = Some(idx);
        return;
    }

    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string();

    let rel = || path.strip_prefix("assets/").unwrap_or(path).to_string();
    let content = if is_image_path(path) {
        let handle: Handle<Image> = asset_server.load(rel());
        FileContent::Image { handle }
    } else if is_audio_path(path) {
        FileContent::Audio {
            handle: asset_server.load(rel()),
        }
    } else {
        FileContent::Text(std::fs::read_to_string(path).unwrap_or_default())
    };

    state.open_files.push(OpenFile {
        path: path.to_string(),
        name,
        content,
        modified: false,
    });
    state.active_tab = Some(state.open_files.len() - 1);
}

pub fn save_open_file(idx: usize, state: &mut EditorState, scripts: &mut Query<&mut ScriptSource>) {
    let Some(file) = state.open_files.get_mut(idx) else {
        return;
    };
    let FileContent::Text(text) = &file.content else {
        return;
    };
    let text = text.clone();
    let path = file.path.clone();

    if let Err(e) = std::fs::write(&path, &text) {
        warn!("Failed to save {path}: {e}");
        return;
    }
    file.modified = false;

    for mut source in scripts.iter_mut() {
        if source.path.as_deref() == Some(path.as_str()) {
            source.source = text.clone();
        }
    }
    info!("Saved {path}");
}

pub fn close_tab(state: &mut EditorState, i: usize) {
    if i >= state.open_files.len() {
        return;
    }
    state.open_files.remove(i);
    state.active_tab = if state.open_files.is_empty() {
        None
    } else {
        match state.active_tab {
            Some(a) if a == i => Some(i.min(state.open_files.len() - 1)),
            Some(a) if a > i => Some(a - 1),
            other => other,
        }
    };
}
