mod build;
mod console_panel;
mod dock;
mod dock_layout;
mod dock_tree;
mod editor_state;
mod editor_ui;
mod fields;
mod file_system_panel;
mod game_view;
mod icons;
mod panel;
mod persistence;
mod project_panel;
mod properties_panel;
mod script_editor;
mod splitter;
mod status_bar;
mod tab_bar;
mod theme;
mod toolbar;
mod viewport;

pub use editor_state::{EditorState, FileTreeNode};

use bevy::prelude::*;

use crate::editor::dock_tree::{DockNode, EditorLayout};
use crate::editor::tab_bar::TabSelectedEvent;
use crate::editor::theme::EditorTheme;

#[allow(dead_code)]
pub(crate) mod ui {
    pub const HEADING: f32 = 13.0;
    pub const PANEL_W: f32 = 260.0;
    pub const ROW_INDENT: f32 = 16.0;
    pub const BTN_H: f32 = 28.0;
    pub const TOOLBAR_H: f32 = 44.0;
    pub const STATUS_H: f32 = 38.0;
}

fn spawn_editor_chrome(
    mut commands: Commands,
    theme: Res<EditorTheme>,
    editor_layout: Res<EditorLayout>,
    panel_registry: Res<panel::PanelRegistry>,
) {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme.colors.viewport_bg),
            dock_layout::DockRoot,
        ))
        .id();

    toolbar::spawn_toolbar(&mut commands, root, &theme);

    let dock_area = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            },
            BackgroundColor(theme.colors.viewport_bg),
        ))
        .id();
    commands.entity(root).add_child(dock_area);

    let mut path = Vec::new();
    let tree_root = dock_layout::spawn_dock_node(
        &mut commands,
        &theme,
        dock_area,
        &editor_layout.dock_tree.root,
        (Val::Percent(100.0), Val::Percent(100.0)),
        &mut path,
        &panel_registry,
    );
    commands.entity(tree_root).insert(dock_layout::DockTreeRoot);

    let bar_height = theme.sizes.statusbar_height;
    let bg = theme.colors.status_bg;
    let text = theme.colors.text;
    let text_dim = theme.colors.text_dim;
    let font = theme.sizes.heading_size;

    let status_bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(bar_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::horizontal(Val::Px(12.0)),
                column_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(bg),
            status_bar::StatusBarRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(font),
                    ..default()
                },
                TextColor(text),
                status_bar::StatusBarZoom,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(font),
                    ..default()
                },
                TextColor(text_dim),
                status_bar::StatusBarParts,
            ));
        })
        .id();
    commands.entity(root).add_child(status_bar);
}

fn register_default_panels(mut registry: ResMut<panel::PanelRegistry>) {
    for panel in panel::create_default_panels() {
        registry.register(panel);
    }
}

fn navigate_to_tabs_mut<'a>(
    tree: &'a mut dock_tree::DockTree,
    path: &[usize],
) -> Option<&'a mut DockNode> {
    let mut current: &'a mut DockNode = &mut tree.root;
    for &idx in path {
        match current {
            DockNode::Split { first, second, .. } => {
                current = match idx {
                    0 => &mut **first,
                    1 => &mut **second,
                    _ => return None,
                };
            }
            DockNode::Tabs { .. } => return None,
        }
    }
    match current {
        DockNode::Tabs { .. } => Some(current),
        DockNode::Split { .. } => None,
    }
}

fn handle_tab_selected_event(event: On<TabSelectedEvent>, mut layout: ResMut<EditorLayout>) {
    let event = event.event();
    let Some(node) = navigate_to_tabs_mut(&mut layout.dock_tree, &event.path) else {
        return;
    };
    if let DockNode::Tabs { active, .. } = node {
        *active = event.new_active;
    }
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<editor_state::EditorState>()
            .init_resource::<build::BuildState>()
            .init_resource::<dock_tree::EditorLayout>()
            .init_resource::<project_panel::ExpandedNodes>()
            .init_resource::<theme::EditorTheme>()
            .init_resource::<splitter::SplitterDragState>()
            .init_resource::<panel::PanelRegistry>()
            .init_resource::<file_system_panel::ExpandedDirs>()
            .init_resource::<file_system_panel::FileSystemSelection>()
            .add_systems(
                Startup,
                (
                    persistence::load_layout_system,
                    icons::setup_editor_font,
                    icons::setup_editor_icons,
                    register_default_panels,
                    spawn_editor_chrome,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    game_view::setup_game_view,
                    build::drive_build,
                    game_view::apply_viewport_zoom,
                    persistence::save_layout_system,
                    editor_ui::editor_state_systems,
                    properties_panel::update_properties_panel,
                    file_system_panel::update_file_system_panel_system,
                    file_system_panel::file_tree_row_click_system,
                    dock_layout::update_split_sizes,
                    status_bar::update_status_bar,
                    fields::update_placeholders,
                    console_panel::console_action_system,
                ),
            )
            .add_systems(Update, console_panel::update_console_panel)
            .add_systems(
                Update,
                (
                    project_panel::tree_row_hover_system,
                    project_panel::tree_row_click_system,
                    project_panel::update_project_panel_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    splitter::splitter_drag_start_system,
                    splitter::splitter_drag_system,
                    splitter::splitter_drag_end_system,
                    splitter::splitter_hover_system,
                    splitter::splitter_cursor_system,
                    tab_bar::tab_click_system,
                    tab_bar::tab_appearance_system,
                    viewport::viewport_interaction_system,
                    viewport::update_viewport_panel_system,
                    viewport::viewport_tab_system,
                    viewport::viewport_toolbar_system,
                    viewport::viewport_audio_system,
                    viewport::viewport_save_system,
                    toolbar::toolbar_interaction_system,
                    toolbar::toolbar_sync_system,
                ),
            )
            .add_observer(handle_tab_selected_event)
            .add_observer(script_editor::on_text_edit_change)
            .add_observer(fields::on_filter_change)
            .add_observer(properties_panel::on_prop_field_change);
    }
}
