#![allow(dead_code)]

use std::collections::HashSet;

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use super::dock_tree::PanelId;
use super::editor_state::EditorState;
use super::fields::{self, FilterInput};
use super::panel::EditorPanel;
use super::theme::EditorTheme;
use crate::instance::{ConsoleLevel, ScriptConsole};
use crate::scenegraph::{NodeId, SceneGraph, commands, serialize};

const ROW_HEIGHT: f32 = 21.0;
const INDENT_WIDTH: f32 = 14.0;
const DRAG_THRESHOLD: f32 = 8.0;

#[derive(Component)]
pub struct HierarchyPanelRoot {
    pub rows_container: Entity,
}

#[derive(Component)]
pub struct HierarchyRow {
    pub node: u64,
}

#[derive(Component)]
pub struct HierarchyArrow(pub u64);

#[derive(Component, Clone, Copy)]
pub enum HierarchyAction {
    AddCollection,
    Undo,
    Redo,
    SaveScene,
    LoadScene,
    SavePrefab,
}

#[derive(Component, Default)]
pub struct HierarchySignature(Vec<(u64, String, usize, bool, bool, String)>);

#[derive(Resource, Default)]
pub struct HierarchyCollapsed(pub HashSet<u64>);

#[derive(Resource, Default)]
pub struct HierarchyDrag {
    pub source: Option<u64>,
    pub start: Vec2,
    pub dragging: bool,
}

pub fn spawn_hierarchy_panel(
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
            HierarchySignature::default(),
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
            HierarchyPanelRoot { rows_container },
        ))
        .id();
    commands.entity(parent).add_child(content);

    let toolbar = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(4.0), Val::Px(2.0)),
            column_gap: Val::Px(4.0),
            ..default()
        },))
        .id();
    commands.entity(content).add_child(toolbar);

    for (label, action) in [
        ("+Col", HierarchyAction::AddCollection),
        ("\u{21B6}", HierarchyAction::Undo),
        ("\u{21B7}", HierarchyAction::Redo),
        ("Save", HierarchyAction::SaveScene),
        ("Load", HierarchyAction::LoadScene),
        ("Prefab", HierarchyAction::SavePrefab),
    ] {
        let btn = commands
            .spawn((
                Node {
                    padding: UiRect::new(Val::Px(7.0), Val::Px(7.0), Val::Px(2.0), Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Px(theme.sizes.corner_radius)),
                    ..default()
                },
                BackgroundColor(theme.colors.button_bg),
                Text::new(label),
                TextFont {
                    font_size: FontSize::from(theme.sizes.heading_size - 2.0),
                    ..default()
                },
                TextColor(theme.colors.text_dim),
                Button,
                Interaction::None,
                action,
            ))
            .id();
        commands.entity(toolbar).add_child(btn);
    }

    let filter_row = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(3.0), Val::Px(5.0)),
            ..default()
        },))
        .id();
    commands.entity(content).add_child(filter_row);

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
        "Filter Nodes",
        theme.sizes.heading_size - 1.0,
        FilterInput::Hierarchy,
    );

    commands.entity(content).add_child(rows_container);

    content
}

struct VisibleRow {
    node: u64,
    name: String,
    depth: usize,
    is_collection: bool,
    has_children: bool,
    summary: String,
}

fn component_summary(world: &World, entity: Entity) -> String {
    let Ok(entity_ref) = world.get_entity(entity) else {
        return String::new();
    };
    const HIDDEN: &[&str] = &[
        "GlobalTransform",
        "Visibility",
        "InheritedVisibility",
        "ViewVisibility",
        "SyncToRenderWorld",
        "TransformTreeChanged",
        "SceneSpawned",
    ];
    let mut names: Vec<String> = entity_ref
        .archetype()
        .components()
        .iter()
        .filter_map(|cid| world.components().get_info(*cid))
        .map(|info| {
            let full = format!("{}", info.name());
            full.rsplit("::").next().unwrap_or("").to_string()
        })
        .filter(|n| !n.is_empty() && !HIDDEN.contains(&n.as_str()))
        .collect();
    names.sort();
    names.dedup();
    let extra = names.len().saturating_sub(3);
    let mut shown = names.into_iter().take(3).collect::<Vec<_>>().join(", ");
    if extra > 0 {
        shown.push_str(&format!(" +{extra}"));
    }
    shown
}

fn build_visible_rows(
    world: &World,
    graph: &SceneGraph,
    collapsed: &HierarchyCollapsed,
    filter: &str,
) -> Vec<VisibleRow> {
    let mut out = Vec::new();
    let needle = filter.trim().to_lowercase();

    let row_for = |world: &World, graph: &SceneGraph, id: NodeId, depth: usize, flat: bool| {
        let node = graph.get(id)?;
        let summary = match node.entity() {
            Some(e) => component_summary(world, e),
            None => "Collection".to_string(),
        };
        Some(VisibleRow {
            node: id.0,
            name: node.name.clone(),
            depth,
            is_collection: node.is_collection(),
            has_children: !flat && !node.children.is_empty(),
            summary,
        })
    };

    if needle.is_empty() {
        let mut stack = vec![(graph.root(), 0usize)];
        while let Some((id, depth)) = stack.pop() {
            let Some(row) = row_for(world, graph, id, depth, false) else {
                continue;
            };
            out.push(row);
            if !collapsed.0.contains(&id.0) {
                let mut children = graph.children_of(id);
                children.reverse();
                for child in children {
                    stack.push((child, depth + 1));
                }
            }
        }
    } else {
        let mut nodes: Vec<NodeId> = graph.iter().map(|n| n.id).collect();
        nodes.sort();
        for id in nodes {
            let Some(node) = graph.get(id) else { continue };
            if node.name.to_lowercase().contains(&needle)
                && let Some(row) = row_for(world, graph, id, 0, true)
            {
                out.push(row);
            }
        }
    }
    out
}

pub fn update_hierarchy_panel_system(world: &mut World) {
    let theme = world.resource::<EditorTheme>().clone();
    let filter = world.resource::<EditorState>().hierarchy_filter.clone();

    let containers: Vec<Entity> = {
        let mut q = world.query::<&HierarchyPanelRoot>();
        q.iter(world).map(|r| r.rows_container).collect()
    };
    if containers.is_empty() {
        return;
    }

    let visible = world.resource_scope::<SceneGraph, _>(|world, graph| {
        world.resource_scope::<HierarchyCollapsed, _>(|world, collapsed| {
            build_visible_rows(world, &graph, &collapsed, &filter)
        })
    });

    let signature: Vec<(u64, String, usize, bool, bool, String)> = visible
        .iter()
        .map(|r| {
            (
                r.node,
                r.name.clone(),
                r.depth,
                r.is_collection,
                r.has_children,
                r.summary.clone(),
            )
        })
        .collect();

    for container in containers {
        let unchanged = world
            .entity(container)
            .get::<HierarchySignature>()
            .is_some_and(|s| s.0 == signature);
        if unchanged {
            continue;
        }

        let to_despawn: Vec<Entity> = world
            .entity(container)
            .get::<Children>()
            .map(|c| c.iter().collect())
            .unwrap_or_default();
        for e in to_despawn {
            if let Ok(mut ec) = world.commands().get_entity(e) {
                ec.try_despawn();
            }
        }

        {
            let mut commands = world.commands();
            for row in &visible {
                let entity = spawn_row(&mut commands, row, &theme);
                commands.entity(container).add_child(entity);
            }
        }

        world
            .entity_mut(container)
            .insert(HierarchySignature(signature.clone()));
        world.flush();
    }
}

fn spawn_row(commands: &mut Commands, row: &VisibleRow, theme: &EditorTheme) -> Entity {
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
                column_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Button,
            Interaction::None,
            HierarchyRow { node: row.node },
        ))
        .id();

    let arrow_glyph = if row.has_children { "\u{25BC}" } else { "" };
    let arrow = commands
        .spawn((
            Node {
                width: Val::Px(12.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text::new(arrow_glyph),
            TextFont {
                font_size: FontSize::from(9.0_f32),
                ..default()
            },
            TextColor(theme.colors.text_dim),
            Button,
            Interaction::None,
            HierarchyArrow(row.node),
        ))
        .id();
    commands.entity(row_entity).add_child(arrow);

    let icon_glyph = if row.is_collection {
        "\u{25A3}"
    } else {
        "\u{25A1}"
    };
    let icon = commands
        .spawn((
            Text::new(icon_glyph),
            TextFont {
                font_size: FontSize::from(11.0_f32),
                ..default()
            },
            TextColor(if row.is_collection {
                theme.colors.accent
            } else {
                theme.colors.text
            }),
        ))
        .id();
    commands.entity(row_entity).add_child(icon);

    let name = commands
        .spawn((
            Text::new(row.name.clone()),
            TextFont {
                font_size: FontSize::from(13.0_f32),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(row_entity).add_child(name);

    let summary = commands
        .spawn((
            Node {
                margin: UiRect::left(Val::Px(6.0)),
                ..default()
            },
            Text::new(row.summary.clone()),
            TextFont {
                font_size: FontSize::from(10.0_f32),
                ..default()
            },
            TextColor(theme.colors.text_faint),
        ))
        .id();
    commands.entity(row_entity).add_child(summary);

    row_entity
}

pub fn hierarchy_interaction_system(world: &mut World) {
    let left_just_pressed = world
        .resource::<ButtonInput<MouseButton>>()
        .just_pressed(MouseButton::Left);

    let pressed_actions: Vec<HierarchyAction> = if left_just_pressed {
        let mut q = world.query::<(&Interaction, &HierarchyAction)>();
        q.iter(world)
            .filter(|(i, _)| **i == Interaction::Pressed)
            .map(|(_, a)| *a)
            .collect()
    } else {
        Vec::new()
    };

    let toggled_arrows: Vec<u64> = if left_just_pressed {
        let mut q = world.query::<(&Interaction, &HierarchyArrow)>();
        q.iter(world)
            .filter(|(i, _)| **i == Interaction::Pressed)
            .map(|(_, a)| a.0)
            .collect()
    } else {
        Vec::new()
    };

    let pressed_rows: Vec<u64> = if left_just_pressed {
        let mut q = world.query::<(&Interaction, &HierarchyRow)>();
        q.iter(world)
            .filter(|(i, _)| **i == Interaction::Pressed)
            .map(|(_, r)| r.node)
            .collect()
    } else {
        Vec::new()
    };

    let hovered_row: Option<u64> = {
        let mut q = world.query::<(&Interaction, &HierarchyRow)>();
        q.iter(world)
            .find(|(i, _)| **i == Interaction::Hovered)
            .map(|(_, r)| r.node)
    };

    let cursor = world
        .iter_entities()
        .find_map(|e| e.get::<Window>().and_then(|w| w.cursor_position()))
        .unwrap_or(Vec2::ZERO);

    let left_released = world
        .resource::<ButtonInput<MouseButton>>()
        .just_released(MouseButton::Left);

    let focus_free = world
        .get_resource::<InputFocus>()
        .map(|f| f.get().is_none())
        .unwrap_or(true);
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let undo_key = focus_free && ctrl && !shift && keys.just_pressed(KeyCode::KeyZ);
    let redo_key = focus_free
        && ctrl
        && (keys.just_pressed(KeyCode::KeyY) || (shift && keys.just_pressed(KeyCode::KeyZ)));

    for node in toggled_arrows {
        let mut collapsed = world.resource_mut::<HierarchyCollapsed>();
        if !collapsed.0.remove(&node) {
            collapsed.0.insert(node);
        }
    }

    for node in &pressed_rows {
        let entity = world.resource::<SceneGraph>().entity_of(NodeId(*node));
        {
            let mut state = world.resource_mut::<EditorState>();
            state.selected_node = Some(*node);
            state.selected_entity = entity;
        }
        let mut drag = world.resource_mut::<HierarchyDrag>();
        drag.source = Some(*node);
        drag.start = cursor;
        drag.dragging = false;
    }

    if pressed_rows.is_empty() {
        let mut drag = world.resource_mut::<HierarchyDrag>();
        if drag.source.is_some() && (cursor - drag.start).length() > DRAG_THRESHOLD {
            drag.dragging = true;
        }
    }

    if left_released {
        let (source, dragging) = {
            let mut drag = world.resource_mut::<HierarchyDrag>();
            let source = drag.source.take();
            let dragging = drag.dragging;
            drag.dragging = false;
            (source, dragging)
        };
        if dragging
            && let Some(source) = source
            && let Some(target) = hovered_row
            && source != target
        {
            commands::do_reparent(world, NodeId(source), NodeId(target));
        }
    }

    if undo_key {
        commands::undo(world);
    }
    if redo_key {
        commands::redo(world);
    }

    for action in pressed_actions {
        apply_action(world, action);
    }
}

fn apply_action(world: &mut World, action: HierarchyAction) {
    match action {
        HierarchyAction::AddCollection => {
            let parent = world
                .resource::<EditorState>()
                .selected_node
                .map(NodeId)
                .filter(|n| world.resource::<SceneGraph>().contains(*n));
            commands::do_add_collection(world, "Collection", parent);
        }
        HierarchyAction::Undo => commands::undo(world),
        HierarchyAction::Redo => commands::redo(world),
        HierarchyAction::SaveScene => {
            let result = world.resource_scope::<SceneGraph, _>(|world, graph| {
                serialize::write_scene(world, &graph, serialize::SCENE_PATH)
            });
            log_result(world, result.map(|_| format!("Scene saved to {}", serialize::SCENE_PATH)));
        }
        HierarchyAction::LoadScene => {
            let result = serialize::load_scene(world, serialize::SCENE_PATH)
                .map(|n| format!("Scene loaded ({n} nodes)"));
            log_result(world, result);
        }
        HierarchyAction::SavePrefab => {
            let selected = world.resource::<EditorState>().selected_node.map(NodeId);
            let result = match selected {
                Some(node) if node != world.resource::<SceneGraph>().root() => {
                    let name = world
                        .resource::<SceneGraph>()
                        .get(node)
                        .map(|n| n.name.clone())
                        .unwrap_or_else(|| "prefab".to_string());
                    let path = format!("assets/prefabs/{}.prefab.ron", name.to_lowercase());
                    world.resource_scope::<SceneGraph, _>(|world, graph| {
                        serialize::save_prefab(world, &graph, node, &path)
                            .map(|_| format!("Prefab saved to {path}"))
                    })
                }
                _ => Err("select a node below the root first".to_string()),
            };
            log_result(world, result);
        }
    }
}

fn log_result(world: &mut World, result: Result<String, String>) {
    let Some(mut console) = world.get_resource_mut::<ScriptConsole>() else {
        return;
    };
    match result {
        Ok(msg) => console.push(ConsoleLevel::Info, msg),
        Err(msg) => console.push(ConsoleLevel::Error, msg),
    }
}

#[allow(clippy::type_complexity)]
pub fn hierarchy_row_visuals_system(
    state: Res<EditorState>,
    drag: Res<HierarchyDrag>,
    theme: Res<EditorTheme>,
    mut rows: Query<(&HierarchyRow, &Interaction, &mut BackgroundColor)>,
) {
    for (row, interaction, mut bg) in rows.iter_mut() {
        let is_selected = state.selected_node == Some(row.node);
        let is_drop_target = drag.dragging
            && drag.source.is_some_and(|s| s != row.node)
            && *interaction == Interaction::Hovered;
        let target = if is_drop_target {
            theme.colors.accent.with_alpha(0.35)
        } else if is_selected {
            theme.colors.selection
        } else if *interaction == Interaction::Hovered {
            theme.colors.tab_hover_bg
        } else {
            Color::NONE
        };
        if bg.0 != target {
            bg.0 = target;
        }
    }
}

pub struct HierarchyPanel;

impl EditorPanel for HierarchyPanel {
    fn id(&self) -> PanelId {
        PanelId::Hierarchy
    }

    fn title(&self) -> &str {
        "Hierarchy"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_hierarchy_panel(commands, parent, theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}
