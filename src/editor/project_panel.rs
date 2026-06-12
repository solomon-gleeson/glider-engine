#![allow(dead_code)]

use bevy::prelude::*;

use super::dock_tree::PanelId;
use super::editor_state::EditorState;
use super::fields::{self, FilterInput};
use super::panel::EditorPanel;
use super::theme::EditorTheme;
use crate::instance::{Instance, ReplicatedStorageRoot, ScriptSource, WorkspaceRoot};

const ROW_HEIGHT: f32 = 21.0;
const INDENT_WIDTH: f32 = 16.0;
const ARROW_RESERVE: f32 = 12.0;

const MAX_DEPTH: usize = 64;

const DOUBLE_CLICK_S: f64 = 0.30;

const SERVICES: &[(&str, ServiceRoot)] = &[
    ("Workspace", ServiceRoot::Workspace),
    ("ReplicatedStorage", ServiceRoot::ReplicatedStorage),
    ("ServerScriptService", ServiceRoot::None),
    ("StarterPlayer", ServiceRoot::None),
    ("StarterGui", ServiceRoot::None),
    ("SoundService", ServiceRoot::None),
    ("Lighting", ServiceRoot::None),
    ("Players", ServiceRoot::None),
    ("Audio", ServiceRoot::None),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ServiceRoot {
    None,
    Workspace,
    ReplicatedStorage,
}

fn service_root(name: &str, ws: Option<Entity>, rs: Option<Entity>) -> Option<Entity> {
    match name {
        "Workspace" => ws,
        "ReplicatedStorage" => rs,
        _ => None,
    }
}

#[derive(Component)]
pub struct ProjectPanelContent {
    pub rows_container: Entity,
}

#[derive(Component)]
pub struct ProjectFilterBar;

#[derive(Component, Clone)]
pub struct TreeRow {
    pub kind: TreeRowKind,
    pub depth: usize,
}

#[derive(Component, Clone, PartialEq, Eq)]
pub enum TreeRowKind {
    Service {
        class_name: String,
        backing: Option<Entity>,
    },

    Instance {
        entity: Entity,
    },
}

impl TreeRowKind {
    pub fn has_visual_children(&self) -> bool {
        matches!(
            self,
            TreeRowKind::Service {
                backing: Some(_),
                ..
            } | TreeRowKind::Instance { .. }
        )
    }
}

#[derive(Component, Clone, Copy)]
pub struct TreeRowLastClick {
    pub time: f64,
}

#[derive(Component, Clone, Default)]
struct LastDesiredSignature(Vec<RowSigEntry>);

#[derive(Clone)]
struct RowSigEntry {
    key: RowKey,
    depth: usize,
}

#[derive(Clone)]
enum RowKey {
    Service(String),
    Instance(Entity),
}

#[derive(Resource, Default, Clone)]
pub struct ExpandedNodes {
    pub instances: std::collections::HashSet<Entity>,
}

impl ExpandedNodes {
    pub fn is_open(&self, kind: &TreeRowKind) -> bool {
        match kind {
            TreeRowKind::Service { .. } => true,
            TreeRowKind::Instance { entity } => self.instances.contains(entity),
        }
    }

    pub fn toggle(&mut self, kind: &TreeRowKind) {
        if let TreeRowKind::Instance { entity } = kind
            && !self.instances.remove(entity)
        {
            self.instances.insert(*entity);
        }
    }
}

pub fn setup_project_panel(commands: &mut Commands, parent: Entity, theme: &EditorTheme) -> Entity {
    let rows_container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
        ))
        .id();

    let content = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
            ProjectPanelContent { rows_container },
        ))
        .id();

    let filter_bar = spawn_filter_bar(commands, theme);

    commands.entity(parent).add_child(content);
    commands.entity(content).add_child(filter_bar);
    commands.entity(content).add_child(rows_container);

    content
}

fn spawn_filter_bar(commands: &mut Commands, theme: &EditorTheme) -> Entity {
    let bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(6.0), Val::Px(6.0), Val::Px(5.0), Val::Px(5.0)),
                column_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(theme.colors.panel_bg),
            ProjectFilterBar,
        ))
        .id();

    for glyph in ["+", "\u{221E}"] {
        let btn = commands
            .spawn((
                Node {
                    padding: UiRect::horizontal(Val::Px(4.0)),
                    ..default()
                },
                Text::new(glyph),
                TextFont {
                    font_size: FontSize::from(theme.sizes.heading_size),
                    ..default()
                },
                TextColor(theme.colors.text),
            ))
            .id();
        commands.entity(bar).add_child(btn);
    }

    let field = commands
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
    commands.entity(bar).add_child(field);

    fields::add_filter_input(
        commands,
        theme,
        field,
        "Filter: name",
        theme.sizes.heading_size - 1.0,
        FilterInput::Scene,
    );

    bar
}

pub fn update_project_panel(world: &mut World) {
    let content_entity = world.resource::<EditorState>().project_panel_content;
    let Some(content_entity) = content_entity else {
        return;
    };
    let rows_container = match world.entity(content_entity).get::<ProjectPanelContent>() {
        Some(c) => c.rows_container,
        None => return,
    };

    let workspace_root: Option<Entity>;
    let replicated_root: Option<Entity>;
    let theme: EditorTheme;
    let expanded: ExpandedNodes;
    let selected_entity: Option<Entity>;
    let selected_service: Option<String>;
    let scene_filter: String;
    {
        workspace_root = world.resource::<WorkspaceRoot>().0;
        replicated_root = world.resource::<ReplicatedStorageRoot>().0;
        theme = world.resource::<EditorTheme>().clone();
        expanded = world.resource::<ExpandedNodes>().clone();
        selected_entity = world.resource::<EditorState>().selected_entity;
        selected_service = world.resource::<EditorState>().selected_service.clone();
        scene_filter = world.resource::<EditorState>().scene_filter.clone();
    }

    let mut desired = build_desired_rows(world, workspace_root, replicated_root);

    let needle = scene_filter.trim().to_lowercase();
    if !needle.is_empty() {
        desired.retain(|row| {
            matches!(row.kind, TreeRowKind::Service { .. })
                || row.name.to_lowercase().contains(&needle)
        });
        for row in &mut desired {
            if matches!(row.kind, TreeRowKind::Service { .. }) {
                row.has_children = false;
            } else {
                row.depth = 1;
                row.has_children = false;
            }
        }
    }

    let need_respawn = match world.entity(rows_container).get::<LastDesiredSignature>() {
        Some(prev) => !signature_matches(prev, &desired),
        None => true,
    };

    if need_respawn {
        despawn_all_rows(world, rows_container);
        let mut commands = world.commands();
        let sig = build_signature(&desired);
        spawn_row_tree(&mut commands, rows_container, &desired, &theme, &expanded);

        world.entity_mut(rows_container).insert(sig);
    } else {
        despawn_all_rows(world, rows_container);
        let mut commands = world.commands();
        spawn_row_tree(&mut commands, rows_container, &desired, &theme, &expanded);
    }

    apply_selection_visuals(
        world,
        rows_container,
        selected_entity,
        selected_service.as_deref(),
        &theme,
    );
}

struct DesiredRow {
    kind: TreeRowKind,
    depth: usize,
    name: String,
    has_children: bool,
}

fn build_desired_rows(
    world: &World,
    workspace_root: Option<Entity>,
    replicated_root: Option<Entity>,
) -> Vec<DesiredRow> {
    let mut out = Vec::new();

    for (name, _) in SERVICES {
        let backing = service_root(name, workspace_root, replicated_root);
        let has_children = backing
            .and_then(|e| world.entity(e).get::<Instance>())
            .map(|i| !i.children.is_empty())
            .unwrap_or(false);

        out.push(DesiredRow {
            kind: TreeRowKind::Service {
                class_name: (*name).to_string(),
                backing,
            },
            depth: 0,
            name: (*name).to_string(),
            has_children,
        });

        if let Some(backing) = backing
            && let Some(inst) = world.entity(backing).get::<Instance>()
        {
            for child in &inst.children {
                push_instance_subtree(world, *child, 1, &mut out);
            }
        }
    }

    out
}

fn push_instance_subtree(world: &World, entity: Entity, depth: usize, out: &mut Vec<DesiredRow>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Some(inst) = world.entity(entity).get::<Instance>() else {
        return;
    };

    out.push(DesiredRow {
        kind: TreeRowKind::Instance { entity },
        depth,
        name: inst.name.clone(),
        has_children: !inst.children.is_empty(),
    });

    for child in &inst.children {
        push_instance_subtree(world, *child, depth + 1, out);
    }
}

fn build_signature(desired: &[DesiredRow]) -> LastDesiredSignature {
    let entries = desired
        .iter()
        .map(|row| RowSigEntry {
            key: match &row.kind {
                TreeRowKind::Service { class_name, .. } => RowKey::Service(class_name.clone()),
                TreeRowKind::Instance { entity } => RowKey::Instance(*entity),
            },
            depth: row.depth,
        })
        .collect();
    LastDesiredSignature(entries)
}

fn signature_matches(last: &LastDesiredSignature, desired: &[DesiredRow]) -> bool {
    let want = build_signature(desired);
    if want.0.len() != last.0.len() {
        return false;
    }
    want.0.iter().zip(last.0.iter()).all(|(a, b)| {
        a.depth == b.depth
            && match (&a.key, &b.key) {
                (RowKey::Service(a), RowKey::Service(b)) => a == b,
                (RowKey::Instance(a), RowKey::Instance(b)) => a == b,
                _ => false,
            }
    })
}

fn despawn_all_rows(world: &mut World, rows_container: Entity) {
    let to_despawn: Vec<Entity> = world
        .entity(rows_container)
        .get::<Children>()
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    for e in to_despawn {
        if let Ok(mut ec) = world.commands().get_entity(e) {
            ec.try_despawn();
        }
    }
}

fn spawn_row_tree(
    commands: &mut Commands,
    rows_container: Entity,
    desired: &[DesiredRow],
    theme: &EditorTheme,
    expanded: &ExpandedNodes,
) {
    let mut spawned_ids: Vec<Entity> = Vec::with_capacity(desired.len());
    for row in desired {
        let entity = spawn_single_row(commands, row, theme);
        commands.entity(rows_container).add_child(entity);
        spawned_ids.push(entity);
    }

    let mut active_ancestors: Vec<usize> = Vec::new();
    let mut hidden: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (i, row) in desired.iter().enumerate() {
        while let Some(&top) = active_ancestors.last() {
            if top >= i || desired[top].depth >= row.depth {
                active_ancestors.pop();
            } else {
                break;
            }
        }

        let ancestor_collapsed = active_ancestors
            .iter()
            .rev()
            .any(|&idx| !expanded.is_open(&desired[idx].kind));

        if ancestor_collapsed {
            hidden.insert(i);
        }

        if row.has_children && !hidden.contains(&i) {
            active_ancestors.push(i);
        }
    }

    for (i, &entity) in spawned_ids.iter().enumerate() {
        if hidden.contains(&i)
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.try_despawn();
        }
    }
}

fn spawn_single_row(commands: &mut Commands, row: &DesiredRow, theme: &EditorTheme) -> Entity {
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
                column_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Interaction::None,
            TreeRow {
                kind: row.kind.clone(),
                depth: row.depth,
            },
        ))
        .id();

    let arrow_text = if row.has_children { "\u{25BC}" } else { "" };
    let arrow = commands
        .spawn((
            Node {
                width: Val::Px(ARROW_RESERVE),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text::new(arrow_text),
            TextFont {
                font_size: FontSize::from(10.0_f32),
                ..default()
            },
            TextColor(theme.colors.text_dim),
        ))
        .id();
    commands.entity(row_entity).add_child(arrow);

    let icon_glyph = icon_glyph_for(&row.kind);
    let icon = commands
        .spawn((
            Node {
                width: Val::Px(26.0),
                justify_content: JustifyContent::Start,
                ..default()
            },
            Text::new(icon_glyph),
            TextFont {
                font_size: FontSize::from(11.0_f32),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(row_entity).add_child(icon);

    let name = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                ..default()
            },
            Text::new(row.name.clone()),
            TextFont {
                font_size: FontSize::from(13.0_f32),
                ..default()
            },
            TextColor(theme.colors.text),
        ))
        .id();
    commands.entity(row_entity).add_child(name);

    row_entity
}

fn icon_glyph_for(kind: &TreeRowKind) -> &'static str {
    let class = match kind {
        TreeRowKind::Service { class_name, .. } => class_name.as_str(),
        TreeRowKind::Instance { .. } => "Instance",
    };
    match class {
        "Workspace" => "[W]",
        "ReplicatedStorage" => "[R]",
        "ServerScriptService" => "[S]",
        "StarterPlayer" => "[P]",
        "StarterGui" => "[G]",
        "SoundService" => "[~]",
        "Lighting" => "[*]",
        "Players" => "[U]",
        "Audio" => "[A]",
        "Model" => "[M]",
        "Part" => "[\u{25A0}]",
        "Script" => "</>",
        _ => "[D]",
    }
}

fn apply_selection_visuals(
    world: &mut World,
    rows_container: Entity,
    selected_entity: Option<Entity>,
    selected_service: Option<&str>,
    theme: &EditorTheme,
) {
    let children: Vec<Entity> = world
        .entity(rows_container)
        .get::<Children>()
        .map(|c| c.iter().collect())
        .unwrap_or_default();

    let sel_color = theme.colors.selection;
    let text_color = theme.colors.text;

    let mut kinds: Vec<(Entity, TreeRowKind)> = Vec::with_capacity(children.len());
    {
        let mut q = world.query::<&TreeRow>();
        for &child in &children {
            if let Ok(row) = q.get(world, child) {
                kinds.push((child, row.kind.clone()));
            }
        }
    }

    for (entity, kind) in kinds {
        let should_be_selected = match &kind {
            TreeRowKind::Instance { entity: e } => selected_entity == Some(*e),
            TreeRowKind::Service {
                class_name,
                backing,
            } => {
                if let Some(b) = backing {
                    selected_entity == Some(*b)
                } else {
                    selected_service == Some(class_name.as_str())
                }
            }
        };

        let target_bg = if should_be_selected {
            sel_color
        } else {
            Color::NONE
        };
        let target_text = if should_be_selected {
            Color::WHITE
        } else {
            text_color
        };

        if let Some(mut bg) = world.entity_mut(entity).get_mut::<BackgroundColor>()
            && bg.0 != target_bg
        {
            *bg = BackgroundColor(target_bg);
        }
        recolor_row_text(world, entity, target_text);
    }
}

fn recolor_row_text(world: &mut World, row: Entity, color: Color) {
    let children: Vec<Entity> = world
        .entity(row)
        .get::<Children>()
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    for child in children {
        if let Some(mut tc) = world.entity_mut(child).get_mut::<TextColor>() {
            tc.0 = color;
        }
    }
}

pub fn tree_row_click_system(
    mut commands: Commands,
    time: Res<Time>,
    interactions: Query<(Entity, &Interaction, &TreeRow), Changed<Interaction>>,
    last_clicks: Query<&TreeRowLastClick>,
    script_sources: Query<&ScriptSource>,
    mut state: ResMut<EditorState>,
    mut expanded: ResMut<ExpandedNodes>,
) {
    for (entity, interaction, row) in interactions.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let now = time.elapsed_secs_f64();
        let is_double_click = last_clicks
            .get(entity)
            .map(|prev| now - prev.time < DOUBLE_CLICK_S)
            .unwrap_or(false);

        commands
            .entity(entity)
            .insert(TreeRowLastClick { time: now });

        if is_double_click {
            if let TreeRowKind::Instance {
                entity: inst_entity,
            } = row.kind
                && let Ok(source) = script_sources.get(inst_entity)
                && let Some(path) = source.path.clone()
            {
                state.request_open = Some(path);
            }
            continue;
        }

        match &row.kind {
            TreeRowKind::Instance {
                entity: inst_entity,
            } => {
                state.selected_entity = Some(*inst_entity);
                state.selected_service = None;
            }
            TreeRowKind::Service {
                class_name,
                backing,
            } => {
                if let Some(b) = backing {
                    state.selected_entity = Some(*b);
                    state.selected_service = None;
                } else {
                    state.selected_entity = None;
                    state.selected_service = Some(class_name.clone());
                }
            }
        }

        if row.kind.has_visual_children() {
            expanded.toggle(&row.kind);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn tree_row_hover_system(
    mut rows: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<TreeRow>)>,
    theme: Res<EditorTheme>,
) {
    for (interaction, mut bg) in rows.iter_mut() {
        let selected_colour = theme.colors.selection;
        match *interaction {
            Interaction::Hovered => {
                if bg.0 != selected_colour {
                    *bg = BackgroundColor(theme.colors.tab_hover_bg);
                }
            }
            Interaction::None => {
                if bg.0 == theme.colors.tab_hover_bg {
                    *bg = BackgroundColor(Color::NONE);
                }
            }
            Interaction::Pressed => {}
        }
    }
}

pub struct ProjectPanel;

impl EditorPanel for ProjectPanel {
    fn id(&self) -> PanelId {
        PanelId::Project
    }

    fn title(&self) -> &str {
        "Project"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        let content = setup_project_panel(commands, parent, theme);
        commands.queue(move |world: &mut World| {
            if let Some(mut state) = world.get_resource_mut::<EditorState>() {
                state.project_panel_content = Some(content);
            }
        });
    }

    fn update(&self, world: &mut World, _panel_entity: Entity) {
        update_project_panel(world);
    }
}

pub fn update_project_panel_system(world: &mut World) {
    update_project_panel(world);
}
