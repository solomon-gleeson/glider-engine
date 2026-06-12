#![allow(dead_code)]

use bevy::prelude::*;

use super::editor_state::EditorState;
use super::theme::EditorTheme;
use crate::instance::Instance;

#[derive(Component)]
pub struct PropNameText;
#[derive(Component)]
pub struct PropClassText;
#[derive(Component)]
pub struct PropPosXText;
#[derive(Component)]
pub struct PropPosYText;
#[derive(Component)]
pub struct PropSizeWText;
#[derive(Component)]
pub struct PropSizeHText;
#[derive(Component)]
pub struct PropRotationText;

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

    commands.entity(root).with_children(|p| {
        p.spawn((
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
        .with_children(|f| {
            f.spawn((
                Text::new("Filter Properties"),
                TextFont {
                    font_size: FontSize::Px(value_size),
                    ..default()
                },
                TextColor(text_faint),
            ));
            f.spawn((
                Text::new("\u{25CB}"),
                TextFont {
                    font_size: FontSize::Px(value_size),
                    ..default()
                },
                TextColor(text_dim),
            ));
        });
    });

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

    spawn_label_value_row(
        commands,
        rows_id,
        "Name",
        "",
        text_dim,
        text,
        field_bg,
        value_size,
        PropNameText,
    );
    spawn_label_value_row(
        commands,
        rows_id,
        "Class",
        "",
        text_dim,
        text,
        field_bg,
        value_size,
        PropClassText,
    );

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

    spawn_label_value_row(
        commands,
        rows_id,
        "Position X",
        "0.0",
        text_dim,
        text,
        field_bg,
        value_size,
        PropPosXText,
    );
    spawn_label_value_row(
        commands,
        rows_id,
        "Position Y",
        "0.0",
        text_dim,
        text,
        field_bg,
        value_size,
        PropPosYText,
    );
    spawn_label_value_row(
        commands,
        rows_id,
        "Width",
        "0.0",
        text_dim,
        text,
        field_bg,
        value_size,
        PropSizeWText,
    );
    spawn_label_value_row(
        commands,
        rows_id,
        "Height",
        "0.0",
        text_dim,
        text,
        field_bg,
        value_size,
        PropSizeHText,
    );
    spawn_label_value_row(
        commands,
        rows_id,
        "Rotation",
        "0\u{00B0}",
        text_dim,
        text,
        field_bg,
        value_size,
        PropRotationText,
    );

    let color_row = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        },))
        .id();
    commands.entity(rows_id).add_child(color_row);

    let color_label = commands
        .spawn((
            Text::new("Color"),
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

#[allow(clippy::too_many_arguments)]
fn spawn_label_value_row<M: Component>(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    initial_value: &str,
    label_color: Color,
    value_color: Color,
    field_bg: Color,
    font_size: f32,
    value_marker: M,
) {
    let row = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        },))
        .id();
    commands.entity(parent).add_child(row);

    let label_entity = commands
        .spawn((
            Text::new(label),
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
            Text::new(initial_value),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(value_color),
            value_marker,
        ))
        .id();
    commands.entity(value_box).add_child(value_entity);
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn update_properties_panel(
    state: Res<EditorState>,
    instances: Query<&Instance>,
    transforms: Query<&Transform>,
    sprites: Query<&Sprite>,
    mut color_rect: Query<&mut BackgroundColor, With<PropColorRect>>,
    mut rows_node: Query<&mut Node, With<PropRowsContainer>>,
    mut text_set: ParamSet<(
        Query<&mut Text, With<PropNameText>>,
        Query<&mut Text, With<PropClassText>>,
        Query<&mut Text, With<PropPosXText>>,
        Query<&mut Text, With<PropPosYText>>,
        Query<&mut Text, With<PropSizeWText>>,
        Query<&mut Text, With<PropSizeHText>>,
        Query<&mut Text, With<PropRotationText>>,
        Query<&mut Text, With<PropEmptyMessage>>,
    )>,
) {
    let Some(entity) = state.selected_entity else {
        set_empty(&mut text_set.p7(), &mut rows_node, "No entity selected");
        return;
    };

    let Ok(inst) = instances.get(entity) else {
        set_empty(
            &mut text_set.p7(),
            &mut rows_node,
            "Selected entity has no Instance component",
        );
        return;
    };

    if let Ok(mut n) = rows_node.single_mut() {
        n.display = Display::Flex;
    }
    if let Ok(mut t) = text_set.p7().single_mut() {
        t.0 = String::new();
    }

    if let Ok(mut t) = text_set.p0().single_mut() {
        t.0 = inst.name.clone();
    }
    if let Ok(mut t) = text_set.p1().single_mut() {
        t.0 = inst.class_name.clone();
    }

    let Ok(transform) = transforms.get(entity) else {
        if let Ok(mut t) = text_set.p2().single_mut() {
            t.0 = "\u{2014}".to_string();
        }
        if let Ok(mut t) = text_set.p3().single_mut() {
            t.0 = "\u{2014}".to_string();
        }
        if let Ok(mut t) = text_set.p4().single_mut() {
            t.0 = "\u{2014}".to_string();
        }
        if let Ok(mut t) = text_set.p5().single_mut() {
            t.0 = "\u{2014}".to_string();
        }
        if let Ok(mut t) = text_set.p6().single_mut() {
            t.0 = "\u{2014}".to_string();
        }
        if let Ok(mut c) = color_rect.single_mut() {
            *c = BackgroundColor(Color::NONE);
        }
        return;
    };

    if let Ok(mut t) = text_set.p2().single_mut() {
        t.0 = fmt_f32(transform.translation.x);
    }
    if let Ok(mut t) = text_set.p3().single_mut() {
        t.0 = fmt_f32(transform.translation.y);
    }
    let rot_deg = transform
        .rotation
        .to_euler(bevy::math::EulerRot::XYZ)
        .2
        .to_degrees();
    if let Ok(mut t) = text_set.p6().single_mut() {
        t.0 = format!("{rot_deg:.1}\u{00B0}");
    }

    if let Ok(sprite) = sprites.get(entity) {
        if let Some(size) = sprite.custom_size {
            if let Ok(mut t) = text_set.p4().single_mut() {
                t.0 = fmt_f32(size.x);
            }
            if let Ok(mut t) = text_set.p5().single_mut() {
                t.0 = fmt_f32(size.y);
            }
        } else {
            if let Ok(mut t) = text_set.p4().single_mut() {
                t.0 = "\u{2014}".to_string();
            }
            if let Ok(mut t) = text_set.p5().single_mut() {
                t.0 = "\u{2014}".to_string();
            }
        }
        if let Ok(mut c) = color_rect.single_mut() {
            *c = BackgroundColor(sprite.color);
        }
    } else {
        if let Ok(mut t) = text_set.p4().single_mut() {
            t.0 = "\u{2014}".to_string();
        }
        if let Ok(mut t) = text_set.p5().single_mut() {
            t.0 = "\u{2014}".to_string();
        }
        if let Ok(mut c) = color_rect.single_mut() {
            *c = BackgroundColor(Color::NONE);
        }
    }
}

fn set_empty(
    empty_msg: &mut Query<&mut Text, With<PropEmptyMessage>>,
    rows_node: &mut Query<&mut Node, With<PropRowsContainer>>,
    msg: &str,
) {
    if let Ok(mut n) = rows_node.single_mut() {
        n.display = Display::None;
    }
    if let Ok(mut t) = empty_msg.single_mut() {
        t.0 = msg.to_string();
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

pub fn apply_property_edits(
    state: Res<EditorState>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
) {
    let Some(entity) = state.selected_entity else {
        return;
    };
    if state.synced_entity != Some(entity) {
        return;
    }

    if let Ok(mut transform) = transforms.get_mut(entity) {
        transform.translation.x = state.pos_x;
        transform.translation.y = state.pos_y;
        transform.rotation = Quat::from_rotation_z(state.rotation.to_radians());
    }

    if let Ok(mut sprite) = sprites.get_mut(entity) {
        sprite.color = Color::srgba(
            state.color.to_srgba().red,
            state.color.to_srgba().green,
            state.color.to_srgba().blue,
            state.color.to_srgba().alpha,
        );
        if let Some(size) = sprite.custom_size.as_mut() {
            size.x = state.size_w;
            size.y = state.size_h;
        }
    }
}
