use bevy::prelude::*;
use ron::ser::PrettyConfig;

use super::dock_tree::EditorLayout;

const LAYOUT_PATH: &str = ".glider/layout.ron";

pub fn save_layout_system(layout: Res<EditorLayout>) {
    if !layout.is_changed() {
        return;
    }

    let dir = std::path::Path::new(".glider");
    if let Err(e) = std::fs::create_dir_all(dir) {
        warn!("Cannot create .glider dir: {e}");
        return;
    }

    let pretty = PrettyConfig::default();
    let serialized = match ron::ser::to_string_pretty(&*layout, pretty) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to serialize EditorLayout: {e}");
            return;
        }
    };

    if let Err(e) = std::fs::write(LAYOUT_PATH, serialized) {
        warn!("Failed to write {}: {e}", LAYOUT_PATH);
    }
}

pub fn load_layout_system(mut layout: ResMut<EditorLayout>) {
    let path = std::path::Path::new(LAYOUT_PATH);
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => return,
    };

    let mut loaded = match ron::from_str::<EditorLayout>(&text) {
        Ok(layout) => {
            info!("Loaded editor layout from {LAYOUT_PATH}");
            layout
        }
        Err(e) => {
            warn!("Failed to parse {LAYOUT_PATH} ({e}); keeping defaults");
            return;
        }
    };

    use super::dock_tree::PanelId;
    if !loaded.dock_tree.root.contains_tab(PanelId::Hierarchy) {
        loaded
            .dock_tree
            .root
            .add_tab_beside(PanelId::Project, PanelId::Hierarchy);
    }

    *layout.bypass_change_detection() = loaded;
}
