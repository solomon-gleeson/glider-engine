#![allow(dead_code)]

use bevy::clipboard::Clipboard;
use bevy::prelude::*;

use super::editor_state::EditorState;
use super::fields::{self, FilterInput};
use super::panel::EditorPanel;
use super::theme::EditorTheme;
use crate::editor::dock_tree::PanelId;
use crate::instance::{ConsoleLevel, ConsoleLine, ScriptConsole};

#[derive(Component)]
pub struct ConsoleContainer;

#[derive(Component)]
pub struct ConsoleLineMarker {
    pub index: usize,
}

#[derive(Component)]
pub struct ConsoleEmptyMarker;

#[derive(Component)]
pub struct ConsoleToolbar;

#[derive(Component)]
pub struct ConsoleFilterField;

#[derive(Component, Clone, Copy)]
pub enum ConsoleAction {
    Copy,
    Clear,
}

pub fn spawn_console_panel(commands: &mut Commands, parent: Entity, theme: &EditorTheme) -> Entity {
    let container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme.colors.console_bg),
            ConsoleContainer,
        ))
        .id();
    commands.entity(parent).add_child(container);

    let toolbar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(5.0), Val::Px(5.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
            ConsoleToolbar,
        ))
        .id();
    commands.entity(container).add_child(toolbar);

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
            ConsoleFilterField,
        ))
        .id();
    commands.entity(toolbar).add_child(filter_box);

    fields::add_filter_input(
        commands,
        theme,
        filter_box,
        "Filter Messages",
        theme.sizes.heading_size - 1.0,
        FilterInput::Console,
    );

    for (label, action) in [("Copy", ConsoleAction::Copy), ("Clear", ConsoleAction::Clear)] {
        let btn = commands
            .spawn((
                Node {
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(theme.sizes.corner_radius)),
                    ..default()
                },
                Text::new(label),
                TextFont {
                    font_size: FontSize::from(theme.sizes.heading_size - 1.0),
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

    let empty = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(4.0), Val::Px(0.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ConsoleEmptyMarker,
        ))
        .id();
    commands.entity(container).add_child(empty);

    let empty_text = commands
        .spawn((
            Text::new("No output \u{2014} press Play to run scripts."),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size - 1.0),
                ..default()
            },
            TextColor(theme.colors.text_faint),
        ))
        .id();
    commands.entity(empty).add_child(empty_text);

    container
}

fn spawn_line_node(
    commands: &mut Commands,
    container: Entity,
    index: usize,
    text: &str,
    color: Color,
) {
    let row = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(2.0), Val::Px(2.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ConsoleLineMarker { index },
        ))
        .id();
    commands.entity(container).add_child(row);

    let line = commands
        .spawn((
            Text::new(text),
            TextFont {
                font_size: FontSize::from(13.0_f32),
                ..default()
            },
            TextColor(color),
        ))
        .id();
    commands.entity(row).add_child(line);
}

pub fn console_action_system(
    actions: Query<(&ConsoleAction, &Interaction), Changed<Interaction>>,
    mut console: ResMut<ScriptConsole>,
    mut clipboard: ResMut<Clipboard>,
) {
    for (action, interaction) in actions.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            ConsoleAction::Copy => {
                let text = console
                    .lines
                    .iter()
                    .map(|l| l.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Err(e) = clipboard.set_text(text) {
                    warn!("Copy to clipboard failed: {e:?}");
                }
            }
            ConsoleAction::Clear => console.clear(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_console_panel(
    mut commands: Commands,
    console: Res<ScriptConsole>,
    state: Res<EditorState>,
    theme: Option<Res<EditorTheme>>,
    containers: Query<Entity, With<ConsoleContainer>>,
    line_markers: Query<(Entity, &ConsoleLineMarker)>,
    mut line_texts: Query<(&mut Text, &mut TextColor), With<ConsoleLineMarker>>,
    mut empty_nodes: Query<&mut Node, With<ConsoleEmptyMarker>>,
    children_q: Query<&Children>,
) {
    let Some(theme) = theme else { return };
    let needle = state.console_filter.trim().to_lowercase();
    let lines: Vec<&ConsoleLine> = console
        .lines
        .iter()
        .filter(|l| needle.is_empty() || l.text.to_lowercase().contains(&needle))
        .collect();
    let lines = &lines;

    for container in containers.iter() {
        let mut scoped_lines: Vec<Entity> = Vec::new();
        let mut scoped_empty: Option<Entity> = None;
        collect_marker_descendants(
            &children_q,
            &line_markers,
            &empty_nodes,
            container,
            &mut scoped_lines,
            &mut scoped_empty,
        );

        if let Some(empty) = scoped_empty
            && let Ok(mut node) = empty_nodes.get_mut(empty)
        {
            node.display = if lines.is_empty() {
                Display::Flex
            } else {
                Display::None
            };
        }

        if scoped_lines.len() != lines.len() {
            for entity in &scoped_lines {
                commands.entity(*entity).try_despawn();
            }
            for (idx, line) in lines.iter().enumerate() {
                let color = line_color(&theme, line.level);
                spawn_line_node(&mut commands, container, idx, &line.text, color);
            }
            continue;
        }

        let mut indexed: Vec<(Entity, usize)> = scoped_lines
            .iter()
            .filter_map(|&e| line_markers.get(e).ok().map(|(_, m)| (e, m.index)))
            .collect();
        indexed.sort_by_key(|&(_, i)| i);

        let contiguous = indexed.iter().enumerate().all(|(i, (_, idx))| *idx == i);
        if !contiguous {
            for entity in &scoped_lines {
                commands.entity(*entity).try_despawn();
            }
            for (idx, line) in lines.iter().enumerate() {
                let color = line_color(&theme, line.level);
                spawn_line_node(&mut commands, container, idx, &line.text, color);
            }
            continue;
        }

        for entity in &scoped_lines {
            let Ok((_, marker)) = line_markers.get(*entity) else {
                continue;
            };
            let Some(line) = lines.get(marker.index) else {
                continue;
            };
            if let Ok((mut text, mut color)) = line_texts.get_mut(*entity) {
                *text = Text::new(line.text.clone());
                color.0 = line_color(&theme, line.level);
            }
        }
    }
}

fn line_color(theme: &EditorTheme, level: ConsoleLevel) -> Color {
    match level {
        ConsoleLevel::Info => theme.colors.text,
        ConsoleLevel::Error => theme.colors.error,
    }
}

fn collect_marker_descendants(
    children_q: &Query<&Children>,
    line_markers: &Query<(Entity, &ConsoleLineMarker)>,
    empty_markers: &Query<&mut Node, With<ConsoleEmptyMarker>>,
    root: Entity,
    out_lines: &mut Vec<Entity>,
    out_empty: &mut Option<Entity>,
) {
    let Ok(children) = children_q.get(root) else {
        return;
    };

    let count = children.len();
    for i in 0..count {
        let child = children[i];
        if line_markers.get(child).is_ok() {
            out_lines.push(child);
        } else if empty_markers.get(child).is_ok() && out_empty.is_none() {
            *out_empty = Some(child);
        }
        collect_marker_descendants(
            children_q,
            line_markers,
            empty_markers,
            child,
            out_lines,
            out_empty,
        );
    }
}

pub struct ConsolePanel;

impl EditorPanel for ConsolePanel {
    fn id(&self) -> PanelId {
        PanelId::Console
    }

    fn title(&self) -> &str {
        "Console"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_console_panel(commands, parent, theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}
