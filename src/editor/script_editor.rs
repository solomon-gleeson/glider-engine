#![allow(dead_code)]

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::text::{EditableText, LineBreak, TextCursorStyle, TextEditChange};

use super::editor_state::{EditorState, FileContent};
use super::icons::EditorFonts;
use super::theme::EditorTheme;
use super::viewport::SaveButton;

#[derive(Component)]
pub struct ScriptEditorBuffer {
    pub tab_index: usize,
}

#[derive(Component)]
pub struct ScriptEditorGutter;

pub fn spawn_script_editor(
    commands: &mut Commands,
    parent: Entity,
    theme: &EditorTheme,
    fonts: &EditorFonts,
    name: &str,
    text: &str,
    tab_index: usize,
) {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme.colors.field_bg),
        ))
        .id();
    commands.entity(parent).add_child(root);

    let header = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(10.0), Val::Px(8.0), Val::Px(5.0), Val::Px(5.0)),
                column_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
        ))
        .id();
    commands.entity(root).add_child(header);

    let name_label = commands
        .spawn((
            Text::new(format!("\u{25A1} {name}")),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(header).add_child(name_label);

    let header_spacer = commands
        .spawn(Node {
            flex_grow: 1.0,
            ..default()
        })
        .id();
    commands.entity(header).add_child(header_spacer);

    let save_btn = commands
        .spawn((
            Node {
                height: Val::Px(24.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(theme.sizes.corner_radius)),
                ..default()
            },
            BackgroundColor(theme.colors.button_bg),
            Button,
            Interaction::None,
            SaveButton,
        ))
        .id();
    commands.entity(header).add_child(save_btn);

    let save_text = commands
        .spawn((
            Text::new("Save"),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size - 1.0),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(save_btn).add_child(save_text);

    let body = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            overflow: Overflow::clip(),
            ..default()
        },))
        .id();
    commands.entity(root).add_child(body);

    let gutter = commands
        .spawn((
            Node {
                width: Val::Px(44.0),
                flex_shrink: 0.0,
                padding: UiRect::new(Val::Px(4.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                ..default()
            },
            Text::new(line_numbers(text)),
            TextFont {
                font: fonts.mono.clone().into(),
                font_size: FontSize::from(theme.sizes.heading_size),
                ..default()
            },
            TextLayout {
                justify: Justify::End,
                ..default()
            },
            TextColor(theme.colors.text_faint),
            ScriptEditorGutter,
        ))
        .id();
    commands.entity(body).add_child(gutter);

    let mut buffer = EditableText::new(text);
    buffer.visible_lines = None;
    buffer.allow_newlines = true;

    let code = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                padding: UiRect::new(Val::Px(2.0), Val::Px(8.0), Val::Px(6.0), Val::Px(6.0)),
                ..default()
            },
            buffer,
            TextFont {
                font: fonts.mono.clone().into(),
                font_size: FontSize::from(theme.sizes.heading_size),
                ..default()
            },
            TextLayout {
                linebreak: LineBreak::NoWrap,
                ..default()
            },
            TextColor(theme.colors.text),
            TextCursorStyle {
                color: theme.colors.text,
                selection_color: theme.colors.accent.with_alpha(0.35),
                unfocused_selection_color: theme.colors.selection,
                selected_text_color: None,
            },
            AutoFocus,
            ScriptEditorBuffer { tab_index },
        ))
        .id();
    commands.entity(body).add_child(code);
}

pub fn on_text_edit_change(
    event: On<TextEditChange>,
    buffers: Query<(&ScriptEditorBuffer, &EditableText)>,
    mut state: ResMut<EditorState>,
    mut gutters: Query<&mut Text, With<ScriptEditorGutter>>,
) {
    let Ok((buffer, editable)) = buffers.get(event.event_target()) else {
        return;
    };

    let mut current = String::new();
    for chunk in editable.value() {
        current.push_str(chunk);
    }

    {
        let Some(file) = state.open_files.get_mut(buffer.tab_index) else {
            return;
        };
        let FileContent::Text(stored) = &mut file.content else {
            return;
        };
        if *stored != current {
            *stored = current.clone();
            file.modified = true;
        }
    }

    let numbers = line_numbers(&current);
    for mut gutter in gutters.iter_mut() {
        if gutter.0 != numbers {
            gutter.0 = numbers.clone();
        }
    }
}

fn line_numbers(text: &str) -> String {
    let count = text.lines().count().max(1);
    let mut out = String::new();
    for n in 1..=count {
        if n > 1 {
            out.push('\n');
        }
        out.push_str(&n.to_string());
    }
    out
}
