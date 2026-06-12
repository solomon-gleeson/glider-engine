#![allow(dead_code)]

use bevy::prelude::*;

use super::build::BuildState;
use super::editor_state::EditorState;
use super::theme::EditorTheme;
use crate::core::ecs::EngineState;

#[derive(Component)]
pub struct ToolbarRoot;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct BuildButton;

#[derive(Component)]
pub struct MenuBarItem;

pub fn spawn_toolbar(commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
    let toolbar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(theme.sizes.toolbar_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(theme.colors.bar_bg),
            ToolbarRoot,
        ))
        .id();
    commands.entity(parent).add_child(toolbar);

    for label in ["File", "Edit", "Add", "Help"] {
        let item = commands
            .spawn((
                Node {
                    height: Val::Px(theme.sizes.btn_height),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Px(theme.sizes.corner_radius)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                MenuBarItem,
            ))
            .id();
        commands.entity(toolbar).add_child(item);

        let text = commands
            .spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::from(theme.sizes.heading_size),
                    ..default()
                },
                TextColor(theme.colors.text),
            ))
            .id();
        commands.entity(item).add_child(text);
    }

    let spacer = commands
        .spawn((Node {
            flex_grow: 1.0,
            ..default()
        },))
        .id();
    commands.entity(toolbar).add_child(spacer);

    let play_btn = spawn_toolbar_button(
        commands,
        toolbar,
        "▶ Play",
        theme.colors.success,
        theme.colors.text,
        theme,
    );
    commands.entity(play_btn).insert(PlayButton);

    let build_btn = spawn_toolbar_button(
        commands,
        toolbar,
        "❖ Build",
        theme.colors.build,
        theme.colors.text,
        theme,
    );
    commands.entity(build_btn).insert(BuildButton);
}

fn spawn_toolbar_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    bg: Color,
    text_color: Color,
    theme: &EditorTheme,
) -> Entity {
    let btn = commands
        .spawn((
            Node {
                width: Val::Auto,
                height: Val::Px(theme.sizes.btn_height),
                padding: UiRect::horizontal(Val::Px(12.0)),
                margin: UiRect::right(Val::Px(4.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(theme.sizes.corner_radius)),
                ..default()
            },
            BackgroundColor(bg),
            Button,
            Interaction::None,
        ))
        .id();
    commands.entity(parent).add_child(btn);

    let text = commands
        .spawn((
            Text::new(label),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size),
                ..default()
            },
            TextColor(text_color),
        ))
        .id();
    commands.entity(btn).add_child(text);

    btn
}

#[allow(clippy::too_many_arguments)]
pub fn toolbar_interaction_system(
    interactions: Query<(Entity, &Interaction), Changed<Interaction>>,
    play_btns: Query<Entity, With<PlayButton>>,
    build_btns: Query<Entity, With<BuildButton>>,
    mut editor_state: ResMut<EditorState>,
    engine_state: Res<State<EngineState>>,
    mut next_engine_state: ResMut<NextState<EngineState>>,
    build_state: Res<BuildState>,
) {
    for (entity, interaction) in interactions.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if play_btns.get(entity).is_ok() {
            let is_playing = *engine_state.get() == EngineState::Running;
            let can_play = *engine_state.get() == EngineState::Editing;
            if is_playing || can_play {
                next_engine_state.set(if is_playing {
                    EngineState::Editing
                } else {
                    EngineState::Running
                });
            }
        } else if build_btns.get(entity).is_ok() && !build_state.running {
            editor_state.build_requested = true;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn toolbar_sync_system(
    mut play_btns: Query<
        (&mut BackgroundColor, &Children),
        (With<PlayButton>, Without<BuildButton>),
    >,
    mut build_btns: Query<
        (&mut BackgroundColor, &Children),
        (With<BuildButton>, Without<PlayButton>),
    >,
    mut texts: Query<&mut Text>,
    engine_state: Res<State<EngineState>>,
    build_state: Res<BuildState>,
    theme: Res<EditorTheme>,
) {
    for (mut bg, children) in play_btns.iter_mut() {
        let is_playing = *engine_state.get() == EngineState::Running;
        let (target_bg, label) = if is_playing {
            (theme.colors.stop, "■ Stop")
        } else {
            (theme.colors.success, "▶ Play")
        };
        *bg = BackgroundColor(target_bg);

        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = label.to_string();
            }
        }
    }

    for (mut bg, children) in build_btns.iter_mut() {
        let target_bg = if build_state.running {
            theme.colors.text_faint
        } else {
            theme.colors.build
        };
        *bg = BackgroundColor(target_bg);

        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = "❖ Build".to_string();
            }
        }
    }
}
