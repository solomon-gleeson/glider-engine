#![allow(dead_code)]

use bevy::prelude::*;

use super::editor_state::EditorState;
use super::theme::EditorTheme;
use crate::instance::Instance;

#[derive(Component)]
pub struct StatusBarZoom;

#[derive(Component)]
pub struct StatusBarParts;

#[derive(Component)]
pub struct StatusBarRoot;

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_bar)
            .add_systems(Update, update_status_bar);
    }
}

fn setup_status_bar(mut commands: Commands, theme: Res<EditorTheme>) {
    let bar_height = theme.sizes.statusbar_height;
    let bg = theme.colors.status_bg;
    let text = theme.colors.text;
    let text_dim = theme.colors.text_dim;
    let font = theme.sizes.heading_size;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(bar_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::horizontal(Val::Px(12.0)),
                column_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(bg),
            StatusBarRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(font),
                    ..default()
                },
                TextColor(text),
                StatusBarZoom,
            ));

            parent.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(font),
                    ..default()
                },
                TextColor(text_dim),
                StatusBarParts,
            ));
        });
}

#[allow(clippy::type_complexity)]
pub fn update_status_bar(
    state: Res<EditorState>,
    parts_q: Query<Entity, With<Instance>>,
    mut text_set: ParamSet<(
        Query<&mut Text, With<StatusBarZoom>>,
        Query<&mut Text, With<StatusBarParts>>,
    )>,
) {
    let zoom_pct = (state.viewport_zoom * 100.0) as i32;

    if let Ok(mut t) = text_set.p0().single_mut() {
        t.0 = format!("Zoom {zoom_pct}%");
    }
    if let Ok(mut t) = text_set.p1().single_mut() {
        let count = parts_q.iter().count();
        t.0 = format!("{count} parts");
    }
}
