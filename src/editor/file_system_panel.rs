#![allow(dead_code)]

use std::collections::HashSet;

use bevy::prelude::*;

use super::dock_tree::PanelId;
use super::editor_state::{EditorState, FileTreeNode};
use super::fields::{self, FilterInput};
use super::panel::EditorPanel;
use super::theme::EditorTheme;

const ROW_HEIGHT: f32 = 21.0;
const INDENT_WIDTH: f32 = 16.0;
const ARROW_RESERVE: f32 = 12.0;
const ICON_RESERVE: f32 = 20.0;

const DOUBLE_CLICK_S: f64 = 0.30;

#[derive(Component)]
pub struct FileSystemRoot {
    pub rows_container: Entity,
}

#[derive(Component, Clone)]
pub struct FileTreeRow {
    pub path: String,
    pub is_dir: bool,
}

#[derive(Component)]
pub struct FileTreeExpanded;

#[derive(Component)]
pub struct FileSystemBreadcrumb;

#[derive(Component)]
pub struct FileSystemFilterField;

#[derive(Component, Clone, Copy)]
pub struct LastClickTime {
    pub time: f64,
}

#[derive(Resource, Default, Clone)]
pub struct ExpandedDirs(pub HashSet<String>);

impl ExpandedDirs {
    pub fn is_open(&self, path: &str) -> bool {
        self.0.contains(path)
    }

    pub fn toggle(&mut self, path: &str) -> bool {
        if !self.0.remove(path) {
            self.0.insert(path.to_string());
            true
        } else {
            false
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct FileSystemSelection(pub Option<String>);

pub fn spawn_file_system_panel(
    commands: &mut Commands,
    parent: Entity,
    theme: &EditorTheme,
) -> Entity {
    let rows_container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
        ))
        .id();

    let content = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
            FileSystemRoot { rows_container },
        ))
        .id();

    let breadcrumb = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(5.0), Val::Px(3.0)),
                column_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
            FileSystemBreadcrumb,
        ))
        .id();

    for glyph in ["\u{2039}", "\u{203A}"] {
        let arrow = commands
            .spawn((
                Node {
                    padding: UiRect::horizontal(Val::Px(3.0)),
                    ..default()
                },
                Text::new(glyph),
                TextFont {
                    font_size: FontSize::from(theme.sizes.heading_size),
                    ..default()
                },
                TextColor(theme.colors.text_dim),
            ))
            .id();
        commands.entity(breadcrumb).add_child(arrow);
    }

    let path_box = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(3.0), Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(theme.colors.field_bg),
        ))
        .id();
    commands.entity(breadcrumb).add_child(path_box);

    let filter_row = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(3.0), Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
            FileSystemFilterField,
        ))
        .id();

    let filter_box = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(3.0), Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(theme.colors.field_bg),
        ))
        .id();
    commands.entity(filter_row).add_child(filter_box);

    fields::add_filter_input(
        commands,
        theme,
        filter_box,
        "Filter Files",
        theme.sizes.heading_size - 1.0,
        FilterInput::Files,
    );

    commands.entity(parent).add_child(content);
    commands.entity(content).add_child(breadcrumb);
    commands.entity(content).add_child(filter_row);
    commands.entity(content).add_child(rows_container);

    content
}

pub fn update_file_system_panel(world: &mut World) {
    let file_tree: Vec<FileTreeNode>;
    let theme: EditorTheme;
    let expanded: ExpandedDirs;
    let selected: FileSystemSelection;
    let filter: String;
    {
        file_tree = world.resource::<EditorState>().file_tree.clone();
        theme = world.resource::<EditorTheme>().clone();
        expanded = world.resource::<ExpandedDirs>().clone();
        selected = world.resource::<FileSystemSelection>().clone();
        filter = world.resource::<EditorState>().file_filter.clone();
    }

    let containers: Vec<Entity> = {
        let mut q = world.query::<&FileSystemRoot>();
        q.iter(world).map(|r| r.rows_container).collect()
    };

    let visible = build_visible_rows(&file_tree, &expanded, &filter);

    for rows_container in containers {
        let existing = collect_existing_rows(world, rows_container);
        let need_respawn = !visible_signatures_match(&existing, &visible);

        if need_respawn {
            despawn_all_rows(world, rows_container);
            let mut commands = world.commands();
            spawn_visible_rows(&mut commands, rows_container, &visible, &theme);
        } else {
            refresh_arrows(world, rows_container, &visible, &theme);
        }

        apply_selection_visuals(world, rows_container, &selected, &theme);
    }
}

#[derive(Clone)]
struct VisibleRow {
    path: String,
    name: String,
    depth: usize,
    is_dir: bool,
    has_children: bool,
}

fn build_visible_rows(
    nodes: &[FileTreeNode],
    expanded: &ExpandedDirs,
    filter: &str,
) -> Vec<VisibleRow> {
    let mut out = Vec::new();
    let needle = filter.trim().to_lowercase();
    if needle.is_empty() {
        push_visible(nodes, 0, expanded, &mut out);
    } else {
        push_matching(nodes, &needle, &mut out);
    }
    out
}

fn push_matching(nodes: &[FileTreeNode], needle: &str, out: &mut Vec<VisibleRow>) {
    for node in nodes {
        if node.name.to_lowercase().contains(needle) {
            out.push(VisibleRow {
                path: node.path.clone(),
                name: node.name.clone(),
                depth: 0,
                is_dir: node.is_dir,
                has_children: false,
            });
        }
        push_matching(&node.children, needle, out);
    }
}

fn push_visible(
    nodes: &[FileTreeNode],
    depth: usize,
    expanded: &ExpandedDirs,
    out: &mut Vec<VisibleRow>,
) {
    for node in nodes {
        let has_children = !node.children.is_empty();
        out.push(VisibleRow {
            path: node.path.clone(),
            name: node.name.clone(),
            depth,
            is_dir: node.is_dir,
            has_children,
        });
        if node.is_dir && has_children && expanded.is_open(&node.path) {
            push_visible(&node.children, depth + 1, expanded, out);
        }
    }
}

#[derive(Clone)]
struct RowSig {
    path: String,
    depth: usize,
    is_dir: bool,
    has_children: bool,
}

impl RowSig {
    fn from_row(row: &VisibleRow) -> Self {
        Self {
            path: row.path.clone(),
            depth: row.depth,
            is_dir: row.is_dir,
            has_children: row.has_children,
        }
    }
}

fn collect_existing_rows(world: &World, rows_container: Entity) -> Vec<RowSig> {
    let mut out = Vec::new();
    let Some(children) = world.entity(rows_container).get::<Children>() else {
        return out;
    };
    for child in children.iter() {
        let Some(row) = world.entity(child).get::<FileTreeRow>() else {
            continue;
        };

        out.push(RowSig {
            path: row.path.clone(),
            depth: depth_of(world, child),
            is_dir: row.is_dir,
            has_children: row.is_dir,
        });
    }
    out
}

fn depth_of(world: &World, row_entity: Entity) -> usize {
    world
        .entity(row_entity)
        .get::<Node>()
        .and_then(|n| match n.padding.left {
            Val::Px(px) => Some((px / INDENT_WIDTH) as usize),
            _ => None,
        })
        .unwrap_or(0)
}

fn visible_signatures_match(existing: &[RowSig], visible: &[VisibleRow]) -> bool {
    if existing.len() != visible.len() {
        return false;
    }
    existing.iter().zip(visible.iter()).all(|(a, b)| {
        a.path == b.path
            && a.depth == b.depth
            && a.is_dir == b.is_dir
            && a.has_children == b.has_children
    })
}

fn despawn_all_rows(world: &mut World, rows_container: Entity) {
    let to_despawn: Vec<Entity> = world
        .entity(rows_container)
        .get::<Children>()
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    for e in to_despawn {
        if let Ok(mut ec) = world.commands().get_entity(e) {
            ec.try_despawn();
        }
    }
}

fn spawn_visible_rows(
    commands: &mut Commands,
    rows_container: Entity,
    visible: &[VisibleRow],
    theme: &EditorTheme,
) {
    for row in visible {
        let entity = spawn_single_row(commands, row, theme);
        commands.entity(rows_container).add_child(entity);
    }
}

fn spawn_single_row(commands: &mut Commands, row: &VisibleRow, theme: &EditorTheme) -> Entity {
    let row_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(ROW_HEIGHT),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect {
                    left: Val::Px(6.0 + row.depth as f32 * INDENT_WIDTH),
                    right: Val::Px(4.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                },
                column_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Interaction::None,
            FileTreeRow {
                path: row.path.clone(),
                is_dir: row.is_dir,
            },
        ))
        .id();

    if row.is_dir && components_are_expanded(&row.path, commands) {
        commands.entity(row_entity).insert(FileTreeExpanded);
    }

    let arrow_text = if row.is_dir {
        if row.has_children && is_path_expanded_for(&row.path, commands) {
            "\u{25BC}"
        } else {
            "\u{25B6}"
        }
    } else {
        ""
    };
    let arrow = commands
        .spawn((
            Node {
                width: Val::Px(ARROW_RESERVE),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text::new(arrow_text),
            TextFont {
                font_size: FontSize::from(10.0_f32),
                ..default()
            },
            TextColor(theme.colors.text_dim),
        ))
        .id();
    commands.entity(row_entity).add_child(arrow);

    let icon_glyph = if row.is_dir { "\u{25A0}" } else { "\u{25A1}" };
    let icon = commands
        .spawn((
            Node {
                width: Val::Px(ICON_RESERVE),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text::new(icon_glyph),
            TextFont {
                font_size: FontSize::from(12.0_f32),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(row_entity).add_child(icon);

    let name = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                ..default()
            },
            Text::new(row.name.clone()),
            TextFont {
                font_size: FontSize::from(13.0_f32),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(row_entity).add_child(name);

    row_entity
}

fn components_are_expanded(_path: &str, _commands: &Commands) -> bool {
    false
}
fn is_path_expanded_for(_path: &str, _commands: &Commands) -> bool {
    false
}

fn refresh_arrows(
    world: &mut World,
    rows_container: Entity,
    visible: &[VisibleRow],
    _theme: &EditorTheme,
) {
    let Some(children) = world.entity(rows_container).get::<Children>() else {
        return;
    };
    let children: Vec<Entity> = children.iter().collect();

    for (i, &child) in children.iter().enumerate() {
        let Some(row) = world.entity(child).get::<FileTreeRow>() else {
            continue;
        };
        let Some(visible_row) = visible.get(i) else {
            continue;
        };
        if row.path != visible_row.path {
            continue;
        }
        let new_arrow = if row.is_dir {
            if visible_row.has_children {
                "\u{25BC}"
            } else {
                "\u{25B6}"
            }
        } else {
            ""
        };

        if let Some(row_children) = world.entity(child).get::<Children>() {
            let row_children: Vec<Entity> = row_children.iter().collect();
            for rc in row_children {
                if let Some(mut text) = world.entity_mut(rc).get_mut::<Text>() {
                    if *text != new_arrow.into() {
                        *text = Text::new(new_arrow);
                    }
                    break;
                }
            }
        }
    }
}

fn apply_selection_visuals(
    world: &mut World,
    rows_container: Entity,
    selected: &FileSystemSelection,
    theme: &EditorTheme,
) {
    let Some(children) = world.entity(rows_container).get::<Children>() else {
        return;
    };
    let children: Vec<Entity> = children.iter().collect();
    let sel_color = theme.colors.selection;
    let text_color = theme.colors.text;
    let selected_text = Color::WHITE;

    for child in children {
        let Some(row) = world.entity(child).get::<FileTreeRow>().cloned() else {
            continue;
        };
        let target_bg = if selected.0.as_deref() == Some(row.path.as_str()) {
            sel_color
        } else {
            Color::NONE
        };
        let target_text = if target_bg == sel_color {
            selected_text
        } else {
            text_color
        };

        if let Some(mut bg) = world.entity_mut(child).get_mut::<BackgroundColor>()
            && bg.0 != target_bg
        {
            *bg = BackgroundColor(target_bg);
        }
        recolor_row_text(world, child, target_text);
    }
}

fn recolor_row_text(world: &mut World, row: Entity, color: Color) {
    let Some(children) = world.entity(row).get::<Children>() else {
        return;
    };
    let children: Vec<Entity> = children.iter().collect();
    for child in children {
        if let Some(mut tc) = world.entity_mut(child).get_mut::<TextColor>() {
            tc.0 = color;
        }
    }
}

pub fn file_tree_row_click_system(
    mut commands: Commands,
    time: Res<Time>,
    interactions: Query<(Entity, &Interaction, &FileTreeRow), Changed<Interaction>>,
    last_clicks: Query<&LastClickTime>,
    mut state: ResMut<EditorState>,
    mut expanded: ResMut<ExpandedDirs>,
    mut selection: ResMut<FileSystemSelection>,
) {
    for (entity, interaction, row) in interactions.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let now = time.elapsed_secs_f64();
        let is_double_click = last_clicks
            .get(entity)
            .map(|prev| now - prev.time < DOUBLE_CLICK_S)
            .unwrap_or(false);

        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_insert(LastClickTime { time: now });
        }

        if is_double_click {
            if row.is_dir {
                expanded.toggle(&row.path);
                selection.0 = Some(row.path.clone());
            } else {
                state.request_open = Some(row.path.clone());
                selection.0 = Some(row.path.clone());
            }
            continue;
        }

        if row.is_dir {
            selection.0 = Some(row.path.clone());

            if !row.path.is_empty() {
                expanded.toggle(&row.path);
            }
        } else {
            selection.0 = Some(row.path.clone());
        }
    }
}

pub struct FileSystemPanel;

impl EditorPanel for FileSystemPanel {
    fn id(&self) -> PanelId {
        PanelId::FileSystem
    }

    fn title(&self) -> &str {
        "FileSystem"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_file_system_panel(commands, parent, theme);
    }

    fn update(&self, world: &mut World, _panel_entity: Entity) {
        update_file_system_panel(world);
    }
}

pub fn update_file_system_panel_system(world: &mut World) {
    update_file_system_panel(world);
}
