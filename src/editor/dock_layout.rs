#![allow(dead_code)]

use bevy::prelude::*;

use crate::editor::dock_tree::{DockNode, EditorLayout, PanelId, SplitDirection};
use crate::editor::panel::PanelRegistry;
use crate::editor::splitter::Splitter;
use crate::editor::tab_bar;
use crate::editor::theme::EditorTheme;

#[derive(Component)]
pub struct DockRoot;

#[derive(Component)]
pub struct DockContainer;

#[derive(Component)]
pub struct DockTreeRoot;

pub(crate) fn spawn_dock_node(
    commands: &mut Commands,
    theme: &EditorTheme,
    parent: Entity,
    node: &DockNode,
    size: (Val, Val),
    path: &mut Vec<usize>,
    registry: &PanelRegistry,
) -> Entity {
    match node {
        DockNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let flex_direction = match direction {
                SplitDirection::Horizontal => FlexDirection::Row,
                SplitDirection::Vertical => FlexDirection::Column,
            };

            let container = commands
                .spawn((
                    Node {
                        width: size.0,
                        height: size.1,
                        flex_direction,
                        ..default()
                    },
                    BackgroundColor(theme.colors.panel_bg),
                    DockContainer,
                ))
                .id();
            commands.entity(parent).add_child(container);

            let first_size = Val::Percent(*ratio * 100.0);
            let first_wh = match direction {
                SplitDirection::Horizontal => (first_size, Val::Percent(100.0)),
                SplitDirection::Vertical => (Val::Percent(100.0), first_size),
            };

            path.push(0);
            let first_entity =
                spawn_dock_node(commands, theme, container, first, first_wh, path, registry);
            path.pop();

            let splitter_px = Val::Px(theme.sizes.splitter_thickness);
            let (splitter_w, splitter_h) = match direction {
                SplitDirection::Horizontal => (splitter_px, Val::Percent(100.0)),
                SplitDirection::Vertical => (Val::Percent(100.0), splitter_px),
            };

            let handle = commands
                .spawn((
                    Node {
                        width: splitter_w,
                        height: splitter_h,
                        ..default()
                    },
                    BackgroundColor(theme.colors.splitter_idle),
                    Interaction::None,
                    Splitter {
                        direction: *direction,
                        split_path: path.clone(),
                    },
                ))
                .id();
            commands.entity(container).add_child(handle);

            let second_size = Val::Percent((1.0 - *ratio) * 100.0);
            let second_wh = match direction {
                SplitDirection::Horizontal => (second_size, Val::Percent(100.0)),
                SplitDirection::Vertical => (Val::Percent(100.0), second_size),
            };

            path.push(1);
            let second_entity = spawn_dock_node(
                commands, theme, container, second, second_wh, path, registry,
            );
            path.pop();

            let _ = first_entity;
            let _ = second_entity;

            container
        }
        DockNode::Tabs { tabs, active } => {
            let container = commands
                .spawn((
                    Node {
                        width: size.0,
                        height: size.1,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(theme.colors.panel_bg),
                    DockContainer,
                ))
                .id();
            commands.entity(parent).add_child(container);

            let single_viewport = tabs.len() == 1 && tabs[0] == PanelId::Viewport;

            let content = if single_viewport {
                let content = commands
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(theme.colors.panel_bg),
                        tab_bar::TabContentArea,
                    ))
                    .id();
                commands.entity(container).add_child(content);
                content
            } else {
                tab_bar::spawn_tab_bar_at(
                    commands,
                    container,
                    tabs,
                    *active,
                    path.clone(),
                    theme,
                    registry,
                )
            };

            if let Some(panel) = registry.get(tabs[*active]) {
                panel.spawn(commands, content, theme);
            }

            container
        }
    }
}

#[allow(dead_code)]
pub fn update_dock_layout(_editor_layout: Res<EditorLayout>) {}

pub fn update_split_sizes(
    editor_layout: Res<EditorLayout>,
    root_query: Query<Entity, With<DockTreeRoot>>,
    mut node_query: Query<&mut Node>,
    children_query: Query<&Children>,
    splitter_query: Query<&Splitter>,
) {
    if !editor_layout.is_changed() {
        return;
    }

    let Ok(root) = root_query.single() else {
        return;
    };

    update_split_node_recursive(
        &editor_layout.dock_tree.root,
        root,
        &mut node_query,
        &children_query,
        &splitter_query,
    );
}

fn update_split_node_recursive(
    node: &DockNode,
    entity: Entity,
    node_query: &mut Query<&mut Node>,
    children_query: &Query<&Children>,
    splitter_query: &Query<&Splitter>,
) {
    match node {
        DockNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let Ok(children) = children_query.get(entity) else {
                return;
            };
            if children.len() != 3 {
                return;
            }

            let first_entity = children[0];
            let handle_entity = children[1];
            let second_entity = children[2];

            if !splitter_query.contains(handle_entity) {
                return;
            }

            let first_size = Val::Percent(*ratio * 100.0);
            let second_size = Val::Percent((1.0 - *ratio) * 100.0);

            match direction {
                SplitDirection::Horizontal => {
                    if let Ok(mut node) = node_query.get_mut(first_entity)
                        && node.width != first_size
                    {
                        node.width = first_size;
                    }
                    if let Ok(mut node) = node_query.get_mut(second_entity)
                        && node.width != second_size
                    {
                        node.width = second_size;
                    }
                }
                SplitDirection::Vertical => {
                    if let Ok(mut node) = node_query.get_mut(first_entity)
                        && node.height != first_size
                    {
                        node.height = first_size;
                    }
                    if let Ok(mut node) = node_query.get_mut(second_entity)
                        && node.height != second_size
                    {
                        node.height = second_size;
                    }
                }
            }

            update_split_node_recursive(
                first,
                first_entity,
                node_query,
                children_query,
                splitter_query,
            );
            update_split_node_recursive(
                second,
                second_entity,
                node_query,
                children_query,
                splitter_query,
            );
        }
        DockNode::Tabs { .. } => {}
    }
}
