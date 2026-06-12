#![allow(dead_code)]

use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};

use crate::editor::dock_tree::{DockNode, DockTree, EditorLayout, SplitDirection};

#[derive(Component, Clone)]
pub struct Splitter {
    pub direction: SplitDirection,
    pub split_path: Vec<usize>,
}

#[derive(Component)]
pub struct SplitterActive;

#[derive(Resource, Default)]
pub struct SplitterDragState {
    pub active: bool,
    pub start_pointer: Vec2,
    pub start_ratio: f32,
    pub split_node_path: Vec<usize>,
}

pub fn navigate_to_split_mut<'a>(
    tree: &'a mut DockTree,
    path: &[usize],
) -> Option<&'a mut DockNode> {
    if path.is_empty() {
        return None;
    }

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
        DockNode::Split { .. } => Some(current),
        DockNode::Tabs { .. } => None,
    }
}

pub fn read_split_ratio(tree: &DockTree, path: &[usize]) -> Option<f32> {
    if path.is_empty() {
        return None;
    }

    let mut current: &DockNode = &tree.root;
    for &idx in path {
        match current {
            DockNode::Split { first, second, .. } => {
                current = match idx {
                    0 => &**first,
                    1 => &**second,
                    _ => return None,
                };
            }
            DockNode::Tabs { .. } => return None,
        }
    }

    match current {
        DockNode::Split { ratio, .. } => Some(*ratio),
        DockNode::Tabs { .. } => None,
    }
}

pub fn splitter_hover_system(
    theme: Res<crate::editor::theme::EditorTheme>,
    mut query: Query<(&Interaction, &mut BackgroundColor), With<Splitter>>,
) {
    for (interaction, mut bg) in &mut query {
        let target = match interaction {
            Interaction::Hovered | Interaction::Pressed => theme.colors.splitter_hover,
            Interaction::None => theme.colors.splitter_idle,
        };
        bg.0 = target;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn splitter_drag_start_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    splitter_entities: Query<Entity, With<Splitter>>,
    interactions: Query<&Interaction, With<Splitter>>,
    splitter_meta: Query<&Splitter>,
    mut drag: ResMut<SplitterDragState>,
    layout: Res<EditorLayout>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if drag.active {
        return;
    }

    let mut pressed_entity: Option<Entity> = None;
    for (entity, interaction) in splitter_entities.iter().zip(interactions.iter()) {
        if *interaction == Interaction::Pressed {
            pressed_entity = Some(entity);
            break;
        }
    }

    let Some(entity) = pressed_entity else {
        return;
    };

    let Ok(splitter) = splitter_meta.get(entity) else {
        return;
    };

    let pointer = windows
        .iter()
        .next()
        .and_then(|w| w.cursor_position())
        .unwrap_or(Vec2::ZERO);

    let start_ratio = read_split_ratio(&layout.dock_tree, &splitter.split_path).unwrap_or(0.5);

    drag.active = true;
    drag.start_pointer = pointer;
    drag.start_ratio = start_ratio;
    drag.split_node_path = splitter.split_path.clone();

    commands.entity(entity).insert(SplitterActive);
}

pub fn splitter_drag_system(
    windows: Query<&Window>,
    splitters: Query<&Splitter>,
    mut drag: ResMut<SplitterDragState>,
    mut editor_layout: ResMut<EditorLayout>,
) {
    if !drag.active {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(pointer) = window.cursor_position() else {
        return;
    };

    let Some(splitter) = splitters
        .iter()
        .find(|s| s.split_path == drag.split_node_path)
    else {
        drag.active = false;
        drag.split_node_path.clear();
        return;
    };

    let Some(node) = navigate_to_split_mut(&mut editor_layout.dock_tree, &drag.split_node_path)
    else {
        drag.active = false;
        drag.split_node_path.clear();
        return;
    };

    let delta = match splitter.direction {
        SplitDirection::Horizontal => pointer.x - drag.start_pointer.x,
        SplitDirection::Vertical => pointer.y - drag.start_pointer.y,
    };

    let container_size = match splitter.direction {
        SplitDirection::Horizontal => window.width(),
        SplitDirection::Vertical => window.height(),
    };

    if container_size <= f32::EPSILON {
        return;
    }

    let new_ratio = (drag.start_ratio + delta / container_size).clamp(0.15, 0.85);

    if let DockNode::Split { ratio, .. } = node {
        *ratio = new_ratio;
    }

    editor_layout.set_changed();
}

pub fn splitter_drag_end_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<SplitterDragState>,
    active: Query<Entity, With<SplitterActive>>,
    mut commands: Commands,
) {
    if !drag.active {
        return;
    }
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    drag.active = false;
    drag.start_pointer = Vec2::ZERO;
    drag.start_ratio = 0.0;
    drag.split_node_path.clear();

    for entity in &active {
        commands.entity(entity).remove::<SplitterActive>();
    }
}

pub fn splitter_cursor_system(
    mut cursor_icons: Query<&mut CursorIcon, With<Window>>,
    hovered: Query<(&Interaction, &Splitter)>,
) {
    let Ok(mut cursor_icon) = cursor_icons.single_mut() else {
        return;
    };

    let icon = hovered
        .iter()
        .find(|(interaction, _)| matches!(*interaction, Interaction::Hovered))
        .map(|(_, splitter)| cursor_for_direction(splitter.direction))
        .unwrap_or(SystemCursorIcon::Default);

    *cursor_icon = CursorIcon::System(icon);
}

fn cursor_for_direction(direction: SplitDirection) -> SystemCursorIcon {
    match direction {
        SplitDirection::Horizontal => SystemCursorIcon::ColResize,
        SplitDirection::Vertical => SystemCursorIcon::RowResize,
    }
}

pub struct SplitterPlugin;

impl Plugin for SplitterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SplitterDragState>().add_systems(
            Update,
            (
                splitter_hover_system,
                splitter_drag_start_system,
                splitter_drag_system,
                splitter_drag_end_system,
                splitter_cursor_system,
            )
                .chain(),
        );
    }
}
