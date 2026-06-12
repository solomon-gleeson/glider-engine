#![allow(dead_code)]

use bevy::prelude::*;

use crate::editor::dock_tree::{DockNode, EditorLayout, PanelId};
use crate::editor::panel::PanelRegistry;
use crate::editor::theme::EditorTheme;

#[derive(Component, Clone, Debug)]
pub struct TabBar {
    pub path: Vec<usize>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TabButton {
    pub panel_id: PanelId,
    pub tab_index: usize,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TabCloseButton {
    pub panel_id: PanelId,
    pub tab_index: usize,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TabContentArea;

#[derive(Event, Clone, Debug)]
pub struct TabSelectedEvent {
    pub path: Vec<usize>,
    pub new_active: usize,
}

pub fn spawn_tab_bar(
    commands: &mut Commands,
    parent: Entity,
    tabs: &[PanelId],
    active: usize,
    theme: &EditorTheme,
    registry: &PanelRegistry,
) -> Entity {
    let bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(theme.sizes.tab_height),
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(theme.colors.tab_bar_bg),
        ))
        .id();
    commands.entity(parent).add_child(bar);

    spawn_tab_buttons(commands, bar, tabs, active, theme, registry);
    spawn_content_area(commands, parent, theme)
}

pub fn spawn_tab_bar_at(
    commands: &mut Commands,
    parent: Entity,
    tabs: &[PanelId],
    active: usize,
    path: Vec<usize>,
    theme: &EditorTheme,
    registry: &PanelRegistry,
) -> Entity {
    let bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(theme.sizes.tab_height),
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(theme.colors.tab_bar_bg),
            TabBar { path },
        ))
        .id();
    commands.entity(parent).add_child(bar);

    spawn_tab_buttons(commands, bar, tabs, active, theme, registry);
    spawn_content_area(commands, parent, theme)
}

fn spawn_tab_buttons(
    commands: &mut Commands,
    bar: Entity,
    tabs: &[PanelId],
    active: usize,
    theme: &EditorTheme,
    registry: &PanelRegistry,
) {
    for (idx, panel_id) in tabs.iter().enumerate() {
        let is_active = idx == active;
        let title = get_tab_title(*panel_id, registry);

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

        let tab_button = commands
            .spawn((
                Node {
                    width: Val::Auto,
                    height: Val::Percent(100.0),
                    padding: UiRect::horizontal(Val::Px(14.0)),
                    flex_direction: FlexDirection::Row,
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
                Interaction::None,
                TabButton {
                    panel_id: *panel_id,
                    tab_index: idx,
                },
            ))
            .id();
        commands.entity(bar).add_child(tab_button);

        let title_entity = commands
            .spawn((
                Text::new(title),
                TextFont {
                    font_size: FontSize::from(theme.sizes.heading_size),
                    ..default()
                },
                TextColor(text_color),
            ))
            .id();
        commands.entity(tab_button).add_child(title_entity);

        if panel_is_closeable(*panel_id, registry) {
            let close_entity = commands
                .spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        margin: UiRect::left(Val::Px(6.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    Interaction::None,
                    TabCloseButton {
                        panel_id: *panel_id,
                        tab_index: idx,
                    },
                ))
                .id();
            commands.entity(tab_button).add_child(close_entity);
        }
    }
}

fn spawn_content_area(commands: &mut Commands, parent: Entity, theme: &EditorTheme) -> Entity {
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
            TabContentArea,
        ))
        .id();
    commands.entity(parent).add_child(content);
    content
}

pub fn get_tab_title(panel_id: PanelId, registry: &PanelRegistry) -> String {
    let registered = registry.title(panel_id);
    if registered == "Unknown" {
        panel_id.name().to_string()
    } else {
        registered.to_string()
    }
}

fn panel_is_closeable(panel_id: PanelId, registry: &PanelRegistry) -> bool {
    registry
        .get(panel_id)
        .map(|p| p.closeable())
        .unwrap_or(false)
}

pub fn tab_click_system(
    interactions: Query<(Entity, &Interaction, &TabButton), Changed<Interaction>>,
    tab_bars: Query<&TabBar>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (tab_entity, interaction, tab) in interactions.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(path) = path_to_tab_bar(tab_entity, &tab_bars, &parents) else {
            continue;
        };

        commands.trigger(TabSelectedEvent {
            path,
            new_active: tab.tab_index,
        });
    }
}

pub fn tab_hover_system(
    mut tabs: Query<(&Interaction, &TabButton, &mut BackgroundColor), Changed<Interaction>>,
    layout: Option<Res<EditorLayout>>,
    theme: Option<Res<EditorTheme>>,
) {
    let Some(theme) = theme else { return };
    let Some(layout) = layout else { return };

    for (interaction, tab, mut bg) in tabs.iter_mut() {
        if is_tab_active(tab, &layout) {
            continue;
        }
        match *interaction {
            Interaction::Hovered => {
                *bg = BackgroundColor(theme.colors.tab_hover_bg);
            }
            Interaction::None | Interaction::Pressed => {
                *bg = BackgroundColor(Color::NONE);
            }
        }
    }
}

pub fn tab_appearance_system(
    mut tabs: Query<(&TabButton, &mut BackgroundColor, &Children)>,
    mut texts: Query<&mut TextColor>,
    layout: Option<Res<EditorLayout>>,
    theme: Option<Res<EditorTheme>>,
) {
    let Some(layout) = layout else { return };
    let Some(theme) = theme else { return };

    for (tab, mut bg, children) in tabs.iter_mut() {
        let active = is_tab_active(tab, &layout);

        let (target_bg, target_text) = if active {
            (theme.colors.active_tab_bg, theme.colors.text)
        } else {
            (Color::NONE, theme.colors.text_dim)
        };
        *bg = BackgroundColor(target_bg);

        for child in children.iter() {
            if let Ok(mut tc) = texts.get_mut(child) {
                tc.0 = target_text;
            }
        }
    }
}

fn path_to_tab_bar(
    mut entity: Entity,
    tab_bars: &Query<&TabBar>,
    parents: &Query<&ChildOf>,
) -> Option<Vec<usize>> {
    for _ in 0..32 {
        let parent = parents.get(entity).ok()?.parent();
        if let Ok(bar) = tab_bars.get(parent) {
            return Some(bar.path.clone());
        }
        entity = parent;
    }
    None
}

fn is_tab_active(tab: &TabButton, layout: &EditorLayout) -> bool {
    find_active_for_tab(tab, &layout.dock_tree.root)
        .map(|active| active == tab.tab_index)
        .unwrap_or(false)
}

fn find_active_for_tab(tab: &TabButton, root: &DockNode) -> Option<usize> {
    match root {
        DockNode::Split { first, second, .. } => {
            find_active_for_tab(tab, first).or_else(|| find_active_for_tab(tab, second))
        }
        DockNode::Tabs { tabs, active } => {
            if tabs.get(tab.tab_index) == Some(&tab.panel_id) {
                Some(*active)
            } else {
                None
            }
        }
    }
}
