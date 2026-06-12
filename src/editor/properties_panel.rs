#![allow(dead_code)]

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::{EditableText, EditableTextFilter, TextEditChange};

use super::editor_state::EditorState;
use super::fields::{self, FilterInput};
use super::theme::EditorTheme;
use crate::instance::Instance;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    Name,
    PosX,
    PosY,
    SizeW,
    SizeH,
    Rotation,
}

#[derive(Component)]
pub struct PropField(pub PropKind);

#[derive(Component)]
pub struct PropRow(pub &'static str);

#[derive(Component)]
pub struct PropClassText;

#[derive(Component)]
pub struct PropColorRect;

#[derive(Component)]
pub struct PropEmptyMessage;

#[derive(Component)]
pub struct PropFilterField;

#[derive(Component)]
pub struct PropRowsContainer;

pub fn spawn_properties_panel(commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
    let text = theme.colors.text;
    let text_dim = theme.colors.text_dim;
    let text_faint = theme.colors.text_faint;
    let field_bg = theme.colors.field_bg;
    let value_size = theme.sizes.heading_size - 1.0;

    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
        ))
        .id();
    commands.entity(parent).add_child(root);

    let filter_field = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(3.0), Val::Px(3.0)),
                margin: UiRect::bottom(Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(field_bg),
            PropFilterField,
        ))
        .id();
    commands.entity(root).add_child(filter_field);
    fields::add_filter_input(
        commands,
        theme,
        filter_field,
        "Filter Properties",
        value_size,
        FilterInput::Properties,
    );

    commands.entity(root).with_children(|p| {
        p.spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(5.0)),
                margin: UiRect::bottom(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(theme.colors.header_bg),
        ))
        .with_children(|h| {
            h.spawn((
                Text::new("Properties"),
                TextFont {
                    font_size: FontSize::Px(theme.sizes.heading_size),
                    ..default()
                },
                TextColor(text),
            ));
        });
    });

    let empty_id = commands
        .spawn((
            Text::new("No entity selected"),
            TextFont {
                font_size: FontSize::Px(value_size),
                ..default()
            },
            TextColor(text_faint),
            PropEmptyMessage,
        ))
        .id();
    commands.entity(root).add_child(empty_id);

    let rows_id = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                display: Display::None,
                ..default()
            },
            PropRowsContainer,
        ))
        .id();
    commands.entity(root).add_child(rows_id);

    spawn_input_row(
        commands, theme, rows_id, "Name", PropKind::Name, false, value_size,
    );
    spawn_class_row(commands, rows_id, text_dim, text, field_bg, value_size);

    let sep = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(1.0),
                margin: UiRect::vertical(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(theme.colors.separator),
        ))
        .id();
    commands.entity(rows_id).add_child(sep);

    spawn_input_row(
        commands, theme, rows_id, "Position X", PropKind::PosX, true, value_size,
    );
    spawn_input_row(
        commands, theme, rows_id, "Position Y", PropKind::PosY, true, value_size,
    );
    spawn_input_row(
        commands, theme, rows_id, "Width", PropKind::SizeW, true, value_size,
    );
    spawn_input_row(
        commands, theme, rows_id, "Height", PropKind::SizeH, true, value_size,
    );
    spawn_input_row(
        commands, theme, rows_id, "Rotation", PropKind::Rotation, true, value_size,
    );

    let color_row = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            PropRow("Colour"),
        ))
        .id();
    commands.entity(rows_id).add_child(color_row);

    let color_label = commands
        .spawn((
            Text::new("Colour"),
            TextFont {
                font_size: FontSize::Px(value_size),
                ..default()
            },
            TextColor(text_dim),
            Node {
                width: Val::Px(96.0),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    commands.entity(color_row).add_child(color_label);

    let color_swatch = commands
        .spawn((
            Node {
                width: Val::Px(24.0),
                height: Val::Px(16.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::WHITE),
            BorderColor::all(theme.colors.separator),
            PropColorRect,
        ))
        .id();
    commands.entity(color_row).add_child(color_swatch);
}

fn spawn_class_row(
    commands: &mut Commands,
    parent: Entity,
    label_color: Color,
    value_color: Color,
    field_bg: Color,
    font_size: f32,
) {
    let row = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            PropRow("Class"),
        ))
        .id();
    commands.entity(parent).add_child(row);

    let label_entity = commands
        .spawn((
            Text::new("Class"),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(label_color),
            Node {
                width: Val::Px(96.0),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    commands.entity(row).add_child(label_entity);

    let value_box = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(2.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(field_bg),
        ))
        .id();
    commands.entity(row).add_child(value_box);

    let value_entity = commands
        .spawn((
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(value_color),
            PropClassText,
        ))
        .id();
    commands.entity(value_box).add_child(value_entity);
}

fn spawn_input_row(
    commands: &mut Commands,
    theme: &EditorTheme,
    parent: Entity,
    label: &'static str,
    kind: PropKind,
    numeric: bool,
    font_size: f32,
) {
    let row = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            PropRow(label),
        ))
        .id();
    commands.entity(parent).add_child(row);

    let label_entity = commands
        .spawn((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(theme.colors.text_dim),
            Node {
                width: Val::Px(96.0),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    commands.entity(row).add_child(label_entity);

    let value_box = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(2.0), Val::Px(2.0)),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(theme.colors.field_bg),
        ))
        .id();
    commands.entity(row).add_child(value_box);

    let input = fields::spawn_text_input(commands, theme, "", font_size);
    commands.entity(input).insert(PropField(kind));
    if numeric {
        commands.entity(input).insert(EditableTextFilter::new(|c| {
            c.is_ascii_digit() || c == '-' || c == '.'
        }));
    }
    commands.entity(value_box).add_child(input);
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_properties_panel(
    state: Res<EditorState>,
    focus: Option<Res<InputFocus>>,
    instances: Query<&Instance>,
    transforms: Query<&Transform>,
    sprites: Query<&Sprite>,
    mut color_rect: Query<&mut BackgroundColor, With<PropColorRect>>,
    mut rows_container: Query<&mut Node, (With<PropRowsContainer>, Without<PropRow>)>,
    mut rows: Query<(&PropRow, &mut Node), Without<PropRowsContainer>>,
    mut fields_q: Query<(Entity, &PropField, &mut EditableText)>,
    mut class_text: Query<&mut Text, (With<PropClassText>, Without<PropEmptyMessage>)>,
    mut empty_msg: Query<&mut Text, (With<PropEmptyMessage>, Without<PropClassText>)>,
) {
    let focused = focus.and_then(|f| f.get());

    let needle = state.prop_filter.trim().to_lowercase();
    for (row, mut node) in rows.iter_mut() {
        let target = if needle.is_empty() || row.0.to_lowercase().contains(&needle) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != target {
            node.display = target;
        }
    }

    let selected = state
        .selected_entity
        .and_then(|e| instances.get(e).ok().map(|i| (e, i)));

    let Some((entity, inst)) = selected else {
        let msg = if state.selected_entity.is_some() {
            "Selected entity has no Instance component"
        } else {
            "No entity selected"
        };
        if let Ok(mut n) = rows_container.single_mut() {
            n.display = Display::None;
        }
        if let Ok(mut t) = empty_msg.single_mut()
            && t.0 != msg
        {
            t.0 = msg.to_string();
        }
        return;
    };

    if let Ok(mut n) = rows_container.single_mut() {
        n.display = Display::Flex;
    }
    if let Ok(mut t) = empty_msg.single_mut()
        && !t.0.is_empty()
    {
        t.0 = String::new();
    }

    if let Ok(mut t) = class_text.single_mut()
        && t.0 != inst.class_name
    {
        t.0 = inst.class_name.clone();
    }

    let transform = transforms.get(entity).ok();
    let sprite = sprites.get(entity).ok();
    let size = sprite.and_then(|s| s.custom_size);

    for (field_entity, field, mut editable) in fields_q.iter_mut() {
        if focused == Some(field_entity) {
            continue;
        }
        let desired = match field.0 {
            PropKind::Name => inst.name.clone(),
            PropKind::PosX => transform
                .map(|t| fmt_f32(t.translation.x))
                .unwrap_or_default(),
            PropKind::PosY => transform
                .map(|t| fmt_f32(t.translation.y))
                .unwrap_or_default(),
            PropKind::SizeW => size.map(|s| fmt_f32(s.x)).unwrap_or_default(),
            PropKind::SizeH => size.map(|s| fmt_f32(s.y)).unwrap_or_default(),
            PropKind::Rotation => transform
                .map(|t| fmt_f32(t.rotation.to_euler(EulerRot::XYZ).2.to_degrees()))
                .unwrap_or_default(),
        };
        if fields::text_of(&editable) != desired {
            editable.editor.set_text(&desired);
        }
    }

    if let Ok(mut c) = color_rect.single_mut() {
        let target = sprite.map(|s| s.color).unwrap_or(Color::NONE);
        if c.0 != target {
            *c = BackgroundColor(target);
        }
    }
}

pub fn on_prop_field_change(
    event: On<TextEditChange>,
    fields_q: Query<(&PropField, &EditableText)>,
    mut state: ResMut<EditorState>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    mut instances: Query<&mut Instance>,
) {
    let Ok((field, editable)) = fields_q.get(event.event_target()) else {
        return;
    };
    let Some(entity) = state.selected_entity else {
        return;
    };
    let value = fields::text_of(editable);
    let value = value.trim();

    if field.0 == PropKind::Name {
        if !value.is_empty()
            && let Ok(mut inst) = instances.get_mut(entity)
            && inst.name != value
        {
            inst.name = value.to_string();
        }
        return;
    }

    let Ok(parsed) = value.parse::<f32>() else {
        return;
    };
    if !parsed.is_finite() {
        return;
    }

    match field.0 {
        PropKind::PosX => {
            state.pos_x = parsed;
            if let Ok(mut t) = transforms.get_mut(entity) {
                t.translation.x = parsed;
            }
        }
        PropKind::PosY => {
            state.pos_y = parsed;
            if let Ok(mut t) = transforms.get_mut(entity) {
                t.translation.y = parsed;
            }
        }
        PropKind::Rotation => {
            state.rotation = parsed;
            if let Ok(mut t) = transforms.get_mut(entity) {
                t.rotation = Quat::from_rotation_z(parsed.to_radians());
            }
        }
        PropKind::SizeW => {
            if parsed > 0.0 {
                state.size_w = parsed;
                if let Ok(mut s) = sprites.get_mut(entity) {
                    let mut sz = s
                        .custom_size
                        .unwrap_or(Vec2::new(parsed, state.size_h.max(1.0)));
                    sz.x = parsed;
                    s.custom_size = Some(sz);
                }
            }
        }
        PropKind::SizeH => {
            if parsed > 0.0 {
                state.size_h = parsed;
                if let Ok(mut s) = sprites.get_mut(entity) {
                    let mut sz = s
                        .custom_size
                        .unwrap_or(Vec2::new(state.size_w.max(1.0), parsed));
                    sz.y = parsed;
                    s.custom_size = Some(sz);
                }
            }
        }
        PropKind::Name => {}
    }
}

fn fmt_f32(v: f32) -> String {
    let rounded = (v * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded:.2}")
    }
}
