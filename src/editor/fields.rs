#![allow(dead_code)]

use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle, TextEditChange};

use super::editor_state::EditorState;
use super::theme::EditorTheme;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum FilterInput {
    Properties,
    Console,
    Files,
    Scene,
    Hierarchy,
}

#[derive(Component)]
pub struct InputPlaceholder(pub Entity);

pub fn text_of(editable: &EditableText) -> String {
    let mut out = String::new();
    for chunk in editable.value() {
        out.push_str(chunk);
    }
    out
}

pub fn spawn_text_input(
    commands: &mut Commands,
    theme: &EditorTheme,
    initial: &str,
    font_size: f32,
) -> Entity {
    commands
        .spawn((
            Node {
                flex_grow: 1.0,
                ..default()
            },
            EditableText::new(initial),
            TextFont {
                font_size: FontSize::from(font_size),
                ..default()
            },
            TextColor(theme.colors.text),
            TextCursorStyle {
                color: theme.colors.text,
                selection_color: theme.colors.accent.with_alpha(0.35),
                unfocused_selection_color: theme.colors.selection,
                selected_text_color: None,
            },
        ))
        .id()
}

pub fn add_filter_input(
    commands: &mut Commands,
    theme: &EditorTheme,
    container: Entity,
    placeholder: &str,
    font_size: f32,
    marker: FilterInput,
) -> Entity {
    let input = spawn_text_input(commands, theme, "", font_size);
    commands.entity(input).insert(marker);
    commands.entity(container).add_child(input);

    let hint = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(8.0),
                ..default()
            },
            Text::new(placeholder),
            TextFont {
                font_size: FontSize::from(font_size),
                ..default()
            },
            TextColor(theme.colors.text_faint),
            Pickable::IGNORE,
            InputPlaceholder(input),
        ))
        .id();
    commands.entity(container).add_child(hint);

    let glyph = commands
        .spawn((
            Text::new("\u{25CB}"),
            TextFont {
                font_size: FontSize::from(font_size),
                ..default()
            },
            TextColor(theme.colors.text_dim),
        ))
        .id();
    commands.entity(container).add_child(glyph);

    input
}

pub fn on_filter_change(
    event: On<TextEditChange>,
    filters: Query<(&FilterInput, &EditableText)>,
    mut state: ResMut<EditorState>,
) {
    let Ok((filter, editable)) = filters.get(event.event_target()) else {
        return;
    };
    let value = text_of(editable);
    match filter {
        FilterInput::Properties => state.prop_filter = value,
        FilterInput::Console => state.console_filter = value,
        FilterInput::Files => state.file_filter = value,
        FilterInput::Scene => state.scene_filter = value,
        FilterInput::Hierarchy => state.hierarchy_filter = value,
    }
}

pub fn update_placeholders(
    buffers: Query<&EditableText>,
    mut placeholders: Query<(&InputPlaceholder, &mut Node)>,
) {
    for (placeholder, mut node) in placeholders.iter_mut() {
        let empty = buffers
            .get(placeholder.0)
            .map(|b| text_of(b).is_empty())
            .unwrap_or(false);
        let target = if empty { Display::Flex } else { Display::None };
        if node.display != target {
            node.display = target;
        }
    }
}
