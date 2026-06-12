#![allow(dead_code)]

use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use super::editor_state::{EditorState, FileContent, ViewportToolMode, close_tab, save_open_file};
use super::game_view::{GAME_VIEW_SIZE, GameView, GameViewCamera};
use super::theme::EditorTheme;
use crate::core::renderer::MainCamera;
use crate::instance::{Instance, ScriptSource};

#[derive(Component)]
pub struct ViewportPanelRoot;

#[derive(Component)]
pub struct ViewportToolbar;

#[derive(Component)]
pub struct ViewportTabBar;

#[derive(Component)]
pub struct ViewportContent;

#[derive(Component)]
pub struct ViewportContentState {
    pub active_tab: Option<usize>,
}

#[derive(Component)]
pub struct ViewportImageNode;

#[derive(Component)]
pub struct ToolModeButton(pub ViewportToolMode);

#[derive(Component)]
pub struct ZoomOutButton;

#[derive(Component)]
pub struct ZoomInButton;

#[derive(Component)]
pub struct ZoomResetButton;

#[derive(Component)]
pub struct ZoomLabel;

#[derive(Component)]
pub struct ViewportTabButton(pub Option<usize>);

#[derive(Component)]
pub struct ViewportTabCloseButton(pub usize);

#[derive(Component)]
pub struct ViewportTabBarState {
    pub file_count: usize,
    pub active_tab: Option<usize>,
}

#[derive(Component)]
pub struct AudioPlayButton(pub Handle<AudioSource>);

#[derive(Component)]
pub struct SaveButton;

pub fn spawn_viewport_panel(
    commands: &mut Commands,
    parent: Entity,
    theme: &EditorTheme,
) -> Entity {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ViewportPanelRoot,
        ))
        .id();
    commands.entity(parent).add_child(root);

    let tab_row = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(theme.sizes.tab_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme.colors.tab_bar_bg),
        ))
        .id();
    commands.entity(root).add_child(tab_row);

    let tab_bar = commands
        .spawn((
            Node {
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ViewportTabBar,
            ViewportTabBarState {
                file_count: usize::MAX,
                active_tab: None,
            },
        ))
        .id();
    commands.entity(tab_row).add_child(tab_bar);

    let new_tab = commands
        .spawn((
            Node {
                padding: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            },
            Text::new("+"),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size),
                ..default()
            },
            TextColor(theme.colors.text_dim),
        ))
        .id();
    commands.entity(tab_row).add_child(new_tab);

    let tab_spacer = commands
        .spawn(Node {
            flex_grow: 1.0,
            ..default()
        })
        .id();
    commands.entity(tab_row).add_child(tab_spacer);

    let expand = commands
        .spawn((
            Node {
                padding: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            },
            Text::new("\u{25A1}"),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size - 1.0),
                ..default()
            },
            TextColor(theme.colors.text_dim),
        ))
        .id();
    commands.entity(tab_row).add_child(expand);

    let toolbar = spawn_viewport_toolbar(commands, theme);
    commands.entity(root).add_child(toolbar);

    let content = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            },
            BackgroundColor(theme.colors.viewport_bg),
            ViewportContent,
            ViewportContentState { active_tab: None },
        ))
        .id();
    commands.entity(root).add_child(content);

    root
}

fn spawn_viewport_toolbar(commands: &mut Commands, theme: &EditorTheme) -> Entity {
    let toolbar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(theme.sizes.inner_toolbar_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(6.0)),
                column_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
            ViewportToolbar,
        ))
        .id();

    for mode in [
        ViewportToolMode::Select,
        ViewportToolMode::Move,
        ViewportToolMode::Rotate,
        ViewportToolMode::Scale,
    ] {
        let btn = spawn_toolbar_button(commands, theme, mode.icon(), ToolModeButton(mode));
        commands.entity(toolbar).add_child(btn);
    }

    let sep = commands
        .spawn((
            Node {
                width: Val::Px(1.0),
                height: Val::Px(20.0),
                margin: UiRect::horizontal(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(theme.colors.separator),
        ))
        .id();
    commands.entity(toolbar).add_child(sep);

    for glyph in ["#", "\u{221E}"] {
        let icon = commands
            .spawn((
                Node {
                    width: Val::Px(26.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Text::new(glyph),
                TextFont {
                    font_size: FontSize::from(theme.sizes.heading_size),
                    ..default()
                },
                TextColor(theme.colors.text_faint),
            ))
            .id();
        commands.entity(toolbar).add_child(icon);
    }

    let sep2 = commands
        .spawn((
            Node {
                width: Val::Px(1.0),
                height: Val::Px(20.0),
                margin: UiRect::horizontal(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(theme.colors.separator),
        ))
        .id();
    commands.entity(toolbar).add_child(sep2);

    let view_menu = commands
        .spawn((
            Node {
                padding: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            },
            Text::new("View"),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(toolbar).add_child(view_menu);

    let spacer = commands
        .spawn(Node {
            width: Val::Auto,
            height: Val::Percent(100.0),
            flex_grow: 1.0,
            ..default()
        })
        .id();
    commands.entity(toolbar).add_child(spacer);

    let zoom_out = spawn_toolbar_button(commands, theme, "-", ZoomOutButton);
    commands.entity(toolbar).add_child(zoom_out);

    let zoom_in = spawn_toolbar_button(commands, theme, "+", ZoomInButton);
    commands.entity(toolbar).add_child(zoom_in);

    let zoom_reset = spawn_toolbar_button(commands, theme, "Reset", ZoomResetButton);
    commands.entity(toolbar).add_child(zoom_reset);

    let zoom_label = commands
        .spawn((
            Node {
                padding: UiRect::horizontal(Val::Px(6.0)),
                ..default()
            },
            Text::new("100%"),
            TextFont {
                font_size: FontSize::from(theme.sizes.heading_size),
                ..default()
            },
            TextColor(theme.colors.text_dim),
            ZoomLabel,
        ))
        .id();
    commands.entity(toolbar).add_child(zoom_label);

    toolbar
}

fn spawn_toolbar_button<T: Component>(
    commands: &mut Commands,
    theme: &EditorTheme,
    label: &str,
    marker: T,
) -> Entity {
    let btn = commands
        .spawn((
            Node {
                width: Val::Auto,
                height: Val::Px(theme.sizes.btn_height),
                padding: UiRect::horizontal(Val::Px(8.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(theme.sizes.corner_radius)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Button,
            Interaction::None,
            marker,
        ))
        .id();

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
    commands.entity(btn).add_child(text);

    btn
}

fn spawn_tab_button(
    commands: &mut Commands,
    theme: &EditorTheme,
    label: &str,
    tab_idx: Option<usize>,
    is_active: bool,
) -> Entity {
    let bg = if is_active {
        theme.colors.active_tab_bg
    } else {
        Color::NONE
    };
    let text_color = if is_active {
        theme.colors.text
    } else {
        theme.colors.text_dim
    };

    let tab = commands
        .spawn((
            Node {
                width: Val::Auto,
                height: Val::Percent(100.0),
                padding: UiRect::horizontal(Val::Px(14.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius {
                    top_left: Val::Px(theme.sizes.corner_radius),
                    top_right: Val::Px(theme.sizes.corner_radius),
                    ..BorderRadius::ZERO
                },
                ..default()
            },
            BackgroundColor(bg),
            Button,
            Interaction::None,
            ViewportTabButton(tab_idx),
        ))
        .id();

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
    commands.entity(tab).add_child(text);

    tab
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn viewport_toolbar_system(
    mut state: ResMut<EditorState>,
    theme: Res<EditorTheme>,
    mut tool_buttons: Query<(&ToolModeButton, &Interaction, &mut BackgroundColor)>,
    zoom_out: Query<&Interaction, (With<ZoomOutButton>, Changed<Interaction>)>,
    zoom_in: Query<&Interaction, (With<ZoomInButton>, Changed<Interaction>)>,
    zoom_reset: Query<&Interaction, (With<ZoomResetButton>, Changed<Interaction>)>,
    mut zoom_label: Query<&mut Text, With<ZoomLabel>>,
) {
    for mut text in zoom_label.iter_mut() {
        let pct = (state.viewport_zoom * 100.0) as i32;
        text.0 = format!("{pct}%");
    }

    for (btn, interaction, mut bg) in tool_buttons.iter_mut() {
        let is_active = state.viewport_tool_mode == btn.0;
        *bg = if is_active {
            BackgroundColor(theme.colors.accent)
        } else {
            BackgroundColor(Color::NONE)
        };
        if *interaction == Interaction::Pressed {
            state.viewport_tool_mode = btn.0;
        }
    }

    for interaction in zoom_out.iter() {
        if *interaction == Interaction::Pressed {
            state.viewport_zoom = (state.viewport_zoom / 1.2).max(0.1);
        }
    }
    for interaction in zoom_in.iter() {
        if *interaction == Interaction::Pressed {
            state.viewport_zoom = (state.viewport_zoom * 1.2).min(10.0);
        }
    }
    for interaction in zoom_reset.iter() {
        if *interaction == Interaction::Pressed {
            state.viewport_zoom = 1.0;
        }
    }
}

pub fn viewport_tab_system(
    mut commands: Commands,
    mut state: ResMut<EditorState>,
    theme: Res<EditorTheme>,
    mut tab_bar_query: Query<(Entity, &mut ViewportTabBarState), With<ViewportTabBar>>,
    mut tab_buttons: Query<(&ViewportTabButton, &Interaction, &mut BackgroundColor)>,
    tab_close_buttons: Query<(&ViewportTabCloseButton, &Interaction), Changed<Interaction>>,
    children_query: Query<&Children>,
) {
    for (tab_btn, interaction, mut bg) in tab_buttons.iter_mut() {
        let is_active = state.active_tab == tab_btn.0;
        *bg = if is_active {
            BackgroundColor(theme.colors.active_tab_bg)
        } else {
            BackgroundColor(Color::NONE)
        };
        if *interaction == Interaction::Pressed {
            state.active_tab = tab_btn.0;
        }
    }

    for (close_btn, interaction) in tab_close_buttons.iter() {
        if *interaction == Interaction::Pressed {
            close_tab(&mut state, close_btn.0);
        }
    }

    for (tab_bar_entity, mut tab_bar_state) in tab_bar_query.iter_mut() {
        let needs_rebuild = tab_bar_state.file_count != state.open_files.len()
            || tab_bar_state.active_tab != state.active_tab;

        if !needs_rebuild {
            continue;
        }

        if let Ok(children) = children_query.get(tab_bar_entity) {
            for child in children.iter() {
                commands.entity(child).try_despawn();
            }
        }

        let viewport_tab = spawn_tab_button(
            &mut commands,
            &theme,
            "\u{25CB} Viewport",
            None,
            state.active_tab.is_none(),
        );
        commands.entity(tab_bar_entity).add_child(viewport_tab);

        for (idx, file) in state.open_files.iter().enumerate() {
            let label = if file.modified {
                format!("{} *", file.name)
            } else {
                file.name.clone()
            };
            let is_active = state.active_tab == Some(idx);
            let tab = spawn_tab_button(&mut commands, &theme, &label, Some(idx), is_active);
            commands.entity(tab_bar_entity).add_child(tab);

            let close = commands
                .spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        margin: UiRect::left(Val::Px(6.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    Button,
                    Interaction::None,
                    ViewportTabCloseButton(idx),
                ))
                .id();
            commands.entity(tab).add_child(close);

            let close_text = commands
                .spawn((
                    Text::new("\u{00D7}"),
                    TextFont {
                        font_size: FontSize::from(13.0_f32),
                        ..default()
                    },
                    TextColor(theme.colors.text),
                ))
                .id();
            commands.entity(close).add_child(close_text);
        }

        tab_bar_state.file_count = state.open_files.len();
        tab_bar_state.active_tab = state.active_tab;
    }
}

#[allow(
    clippy::type_complexity,
    clippy::collapsible_if,
    clippy::too_many_arguments
)]
pub fn viewport_interaction_system(
    mut state: ResMut<EditorState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    viewport_nodes: Query<(&GlobalTransform, &ComputedNode), With<ViewportImageNode>>,
    main_cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    game_cam: Query<(&Camera, &GlobalTransform), With<GameViewCamera>>,
    mut set: ParamSet<(
        Query<(Entity, &Transform, &Sprite), With<Instance>>,
        Query<&mut Transform>,
        Query<&mut Sprite>,
    )>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let (viewport_transform, viewport_computed) = match viewport_nodes.single() {
        Ok(n) => n,
        Err(_) => return,
    };
    let Ok((main_camera, main_camera_transform)) = main_cam.single() else {
        return;
    };
    let Ok((game_camera, game_camera_transform)) = game_cam.single() else {
        return;
    };

    let cursor_world = match main_camera.viewport_to_world_2d(main_camera_transform, cursor) {
        Ok(pos) => pos,
        Err(_) => return,
    };

    let center = viewport_transform.translation().truncate();
    let size = viewport_computed.size();
    let half = size / 2.0;

    let in_viewport = size.x > 0.0
        && size.y > 0.0
        && cursor_world.x >= center.x - half.x
        && cursor_world.x <= center.x + half.x
        && cursor_world.y >= center.y - half.y
        && cursor_world.y <= center.y + half.y;

    let normalized = if size.x > 0.0 && size.y > 0.0 {
        Some(Vec2::new(
            (cursor_world.x - (center.x - half.x)) / size.x,
            (cursor_world.y - (center.y - half.y)) / size.y,
        ))
    } else {
        None
    };

    let norm_to_game_world = |n: Vec2| -> Option<Vec2> {
        let pixel = Vec2::new(
            n.x * GAME_VIEW_SIZE.x as f32,
            (1.0 - n.y) * GAME_VIEW_SIZE.y as f32,
        );
        game_camera
            .viewport_to_world_2d(game_camera_transform, pixel)
            .ok()
    };

    if in_viewport {
        for event in mouse_wheel_events.read() {
            state.viewport_zoom = (state.viewport_zoom * (1.0 + event.y * 0.0015)).clamp(0.1, 10.0);
        }
    }

    if mouse_button.just_pressed(MouseButton::Left) && in_viewport {
        if let Some(n) = normalized {
            if let Some(world) = norm_to_game_world(n) {
                let parts = set.p0();
                let parts_vec: Vec<(Entity, Vec2, Vec2, f32)> = parts
                    .iter()
                    .filter_map(|(e, t, s)| {
                        let size = s.custom_size?;
                        let angle = t.rotation.to_euler(EulerRot::XYZ).2;
                        Some((e, t.translation.truncate(), size, angle))
                    })
                    .collect();
                if let Some((entity, ent_center, ent_size, ent_angle)) = parts_vec
                    .into_iter()
                    .rev()
                    .find_map(|(e, ent_center, ent_size, ent_angle)| {
                        let local = Vec2::from_angle(-ent_angle).rotate(world - ent_center);
                        let half = ent_size * 0.5;
                        if local.x.abs() <= half.x && local.y.abs() <= half.y {
                            Some((e, ent_center, ent_size, ent_angle))
                        } else {
                            None
                        }
                    })
                {
                    state.pos_x = ent_center.x;
                    state.pos_y = ent_center.y;
                    state.size_w = ent_size.x;
                    state.size_h = ent_size.y;
                    state.rotation = ent_angle.to_degrees();
                    state.selected_entity = Some(entity);
                    state.drag_pointer_origin = Some(world);
                    state.drag_obj_origin = Some(ent_center);
                    state.drag_screen_origin = Some(cursor);
                    state.drag_rot_origin = Some(ent_angle);
                    state.drag_size_origin = Some(ent_size);
                } else if state.viewport_tool_mode == ViewportToolMode::Select {
                    state.selected_entity = None;
                }
            }
        }
    }

    if mouse_button.pressed(MouseButton::Left)
        && let Some(origin) = state.drag_pointer_origin
        && let Some(obj0) = state.drag_obj_origin
        && state.drag_screen_origin.is_some_and(|screen_origin| {
            (cursor - screen_origin).length() >= 4.0
        })
        && let Some(entity) = state.selected_entity
        && let Some(n) = normalized
        && let Some(world) = norm_to_game_world(n)
    {
        match state.viewport_tool_mode {
            ViewportToolMode::Select => {}
            ViewportToolMode::Move => {
                let x = obj0.x + (world.x - origin.x);
                let y = obj0.y + (world.y - origin.y);
                state.pos_x = x;
                state.pos_y = y;
                let mut transforms = set.p1();
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    transform.translation.x = x;
                    transform.translation.y = y;
                }
            }
            ViewportToolMode::Rotate => {
                let v0 = origin - obj0;
                let v1 = world - obj0;
                if let Some(rot0) = state.drag_rot_origin
                    && v0.length_squared() > f32::EPSILON
                    && v1.length_squared() > f32::EPSILON
                {
                    let angle = rot0 + (v1.y.atan2(v1.x) - v0.y.atan2(v0.x));
                    state.rotation = angle.to_degrees();
                    let mut transforms = set.p1();
                    if let Ok(mut transform) = transforms.get_mut(entity) {
                        transform.rotation = Quat::from_rotation_z(angle);
                    }
                }
            }
            ViewportToolMode::Scale => {
                let d0 = (origin - obj0).length();
                let d1 = (world - obj0).length();
                if let Some(size0) = state.drag_size_origin
                    && d0 > 1.0
                {
                    let size = (size0 * (d1 / d0)).max(Vec2::splat(1.0));
                    state.size_w = size.x;
                    state.size_h = size.y;
                    let mut sprites = set.p2();
                    if let Ok(mut sprite) = sprites.get_mut(entity) {
                        sprite.custom_size = Some(size);
                    }
                }
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        state.drag_pointer_origin = None;
        state.drag_obj_origin = None;
        state.drag_screen_origin = None;
        state.drag_rot_origin = None;
        state.drag_size_origin = None;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_viewport_panel_system(
    mut commands: Commands,
    state: Res<EditorState>,
    theme: Res<EditorTheme>,
    game_view: Option<Res<GameView>>,
    mut content_query: Query<(Entity, &mut ViewportContentState), With<ViewportContent>>,
    children_query: Query<&Children>,
    images: Res<Assets<Image>>,
    fonts: Res<super::icons::EditorFonts>,
) {
    let Some(game_view) = game_view else { return };
    for (content_entity, mut content_state) in content_query.iter_mut() {
        let has_content = match children_query.get(content_entity) {
            Ok(children) => !children.is_empty(),
            Err(_) => false,
        };
        if has_content && content_state.active_tab == state.active_tab {
            continue;
        }

        if let Ok(children) = children_query.get(content_entity) {
            for child in children.iter() {
                commands.entity(child).try_despawn();
            }
        }

        content_state.active_tab = state.active_tab;

        match state.active_tab {
            None => {
                spawn_game_view_content(&mut commands, content_entity, &game_view, &theme);
            }
            Some(idx) => {
                if let Some(file) = state.open_files.get(idx) {
                    match &file.content {
                        FileContent::Image { handle, .. } => {
                            spawn_image_preview(
                                &mut commands,
                                content_entity,
                                handle,
                                &images,
                                &theme,
                            );
                        }
                        FileContent::Audio { handle } => {
                            spawn_audio_player(
                                &mut commands,
                                content_entity,
                                handle.clone(),
                                &file.name,
                                &theme,
                            );
                        }
                        FileContent::Text(text) => {
                            super::script_editor::spawn_script_editor(
                                &mut commands,
                                content_entity,
                                &theme,
                                &fonts,
                                &file.name,
                                text,
                                idx,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn viewport_audio_system(
    mut state: ResMut<EditorState>,
    buttons: Query<(&AudioPlayButton, &Interaction), Changed<Interaction>>,
) {
    for (btn, interaction) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            state.play_audio = Some(btn.0.clone());
        }
    }
}

pub fn viewport_save_system(
    mut state: ResMut<EditorState>,
    buttons: Query<&Interaction, (With<SaveButton>, Changed<Interaction>)>,
    mut scripts: Query<&mut ScriptSource>,
) {
    for interaction in buttons.iter() {
        if *interaction == Interaction::Pressed
            && let Some(idx) = state.active_tab
        {
            save_open_file(idx, &mut state, &mut scripts);
        }
    }
}

pub fn apply_viewport_zoom(
    state: Res<EditorState>,
    mut cameras: Query<&mut Projection, With<GameViewCamera>>,
) {
    for mut projection in &mut cameras {
        if let Projection::Orthographic(ortho) = &mut *projection
            && (ortho.scale - state.viewport_zoom).abs() > f32::EPSILON
        {
            ortho.scale = state.viewport_zoom;
        }
    }
}

fn spawn_game_view_content(
    commands: &mut Commands,
    parent: Entity,
    game_view: &GameView,
    theme: &EditorTheme,
) {
    let container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme.colors.viewport_bg),
        ))
        .id();
    commands.entity(parent).add_child(container);

    let image = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ImageNode::new(game_view.image.clone()),
            Transform::default(),
            Interaction::None,
            ViewportImageNode,
        ))
        .id();
    commands.entity(container).add_child(image);
}

fn spawn_image_preview(
    commands: &mut Commands,
    parent: Entity,
    handle: &Handle<Image>,
    images: &Res<Assets<Image>>,
    theme: &EditorTheme,
) {
    let container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme.colors.viewport_bg),
        ))
        .id();
    commands.entity(parent).add_child(container);

    let size = images
        .get(handle)
        .map(|im| im.size())
        .unwrap_or(UVec2::splat(256));

    let image = commands
        .spawn((
            Node {
                width: Val::Px(size.x as f32),
                height: Val::Px(size.y as f32),
                ..default()
            },
            ImageNode::new(handle.clone()),
        ))
        .id();
    commands.entity(container).add_child(image);
}

fn spawn_audio_player(
    commands: &mut Commands,
    parent: Entity,
    handle: Handle<AudioSource>,
    name: &str,
    theme: &EditorTheme,
) {
    let container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme.colors.viewport_bg),
        ))
        .id();
    commands.entity(parent).add_child(container);

    let label = commands
        .spawn((
            Text::new(format!("Audio: {name}")),
            TextFont {
                font_size: FontSize::from(16.0_f32),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(container).add_child(label);

    let play_btn = commands
        .spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Px(theme.sizes.btn_height),
                margin: UiRect::top(Val::Px(12.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(theme.sizes.corner_radius)),
                ..default()
            },
            BackgroundColor(theme.colors.button_bg),
            Button,
            Interaction::None,
            AudioPlayButton(handle),
        ))
        .id();
    commands.entity(container).add_child(play_btn);

    let play_text = commands
        .spawn((
            Text::new("Play"),
            TextFont {
                font_size: FontSize::from(15.0_f32),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(play_btn).add_child(play_text);
}

