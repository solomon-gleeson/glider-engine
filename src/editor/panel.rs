#![allow(dead_code)]

use std::collections::HashMap;

use bevy::prelude::*;

use super::console_panel::ConsolePanel;
use super::dock_tree::PanelId;
use super::file_system_panel::FileSystemPanel;
use super::project_panel::ProjectPanel;
use super::properties_panel;
use super::theme::EditorTheme;
use super::viewport;

impl PanelId {
    pub fn name(&self) -> &'static str {
        match self {
            PanelId::Viewport => "Viewport",
            PanelId::Project => "Project",
            PanelId::FileSystem => "FileSystem",
            PanelId::Inspector => "Inspector",
            PanelId::Console => "Console",
            PanelId::Output => "Output",
            PanelId::Debugger => "Debugger",
            PanelId::Animation => "Animation",
            PanelId::Toolbar => "Toolbar",
            PanelId::Menubar => "Menubar",
            PanelId::StatusBar => "StatusBar",
        }
    }

    pub fn default_panel_order() -> Vec<PanelId> {
        vec![
            PanelId::Viewport,
            PanelId::Project,
            PanelId::Inspector,
            PanelId::FileSystem,
            PanelId::Console,
            PanelId::Output,
            PanelId::Debugger,
            PanelId::Animation,
            PanelId::Toolbar,
            PanelId::Menubar,
            PanelId::StatusBar,
        ]
    }
}

pub trait EditorPanel: Send + Sync + 'static {
    fn id(&self) -> PanelId;

    fn title(&self) -> &str;

    fn closeable(&self) -> bool {
        false
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme);

    fn update(&self, world: &mut World, panel_entity: Entity);
}

#[derive(Resource, Default)]
pub struct PanelRegistry {
    panels: HashMap<PanelId, Box<dyn EditorPanel>>,
}

impl PanelRegistry {
    pub fn register(&mut self, panel: Box<dyn EditorPanel>) {
        self.panels.insert(panel.id(), panel);
    }

    pub fn get(&self, id: PanelId) -> Option<&dyn EditorPanel> {
        self.panels.get(&id).map(|p| p.as_ref())
    }

    pub fn title(&self, id: PanelId) -> &str {
        self.panels.get(&id).map(|p| p.title()).unwrap_or("Unknown")
    }

    pub fn ids(&self) -> Vec<PanelId> {
        self.panels.keys().copied().collect()
    }
}

fn spawn_stub_label(commands: &mut Commands, parent: Entity, title: &str, theme: &EditorTheme) {
    commands.entity(parent).with_children(|parent| {
        parent.spawn((
            Text::new(title),
            TextFont {
                font_size: FontSize::Px(theme.sizes.heading_size),
                ..default()
            },
            TextColor(theme.colors.text_dim),
            Node {
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
        ));
    });
}

pub struct ViewportPanel;

impl EditorPanel for ViewportPanel {
    fn id(&self) -> PanelId {
        PanelId::Viewport
    }

    fn title(&self) -> &str {
        "Viewport"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        viewport::spawn_viewport_panel(commands, parent, theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub struct InspectorPanel;

impl EditorPanel for InspectorPanel {
    fn id(&self) -> PanelId {
        PanelId::Inspector
    }

    fn title(&self) -> &str {
        "Inspector"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        properties_panel::spawn_properties_panel(commands, parent, theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub struct OutputPanel;

impl EditorPanel for OutputPanel {
    fn id(&self) -> PanelId {
        PanelId::Output
    }

    fn title(&self) -> &str {
        "Output"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_stub_label(commands, parent, self.title(), theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub struct DebuggerPanel;

impl EditorPanel for DebuggerPanel {
    fn id(&self) -> PanelId {
        PanelId::Debugger
    }

    fn title(&self) -> &str {
        "Debugger"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_stub_label(commands, parent, self.title(), theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub struct AnimationPanel;

impl EditorPanel for AnimationPanel {
    fn id(&self) -> PanelId {
        PanelId::Animation
    }

    fn title(&self) -> &str {
        "Animation"
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_stub_label(commands, parent, self.title(), theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub struct ToolbarPanel;

impl EditorPanel for ToolbarPanel {
    fn id(&self) -> PanelId {
        PanelId::Toolbar
    }

    fn title(&self) -> &str {
        "Toolbar"
    }

    fn closeable(&self) -> bool {
        false
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_stub_label(commands, parent, self.title(), theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub struct MenubarPanel;

impl EditorPanel for MenubarPanel {
    fn id(&self) -> PanelId {
        PanelId::Menubar
    }

    fn title(&self) -> &str {
        "Menubar"
    }

    fn closeable(&self) -> bool {
        false
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_stub_label(commands, parent, self.title(), theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub struct StatusBarPanel;

impl EditorPanel for StatusBarPanel {
    fn id(&self) -> PanelId {
        PanelId::StatusBar
    }

    fn title(&self) -> &str {
        "StatusBar"
    }

    fn closeable(&self) -> bool {
        false
    }

    fn spawn(&self, commands: &mut Commands, parent: Entity, theme: &EditorTheme) {
        spawn_stub_label(commands, parent, self.title(), theme);
    }

    fn update(&self, _world: &mut World, _panel_entity: Entity) {}
}

pub fn create_default_panels() -> Vec<Box<dyn EditorPanel>> {
    vec![
        Box::new(ViewportPanel),
        Box::new(ProjectPanel),
        Box::new(InspectorPanel),
        Box::new(FileSystemPanel),
        Box::new(ConsolePanel),
        Box::new(OutputPanel),
        Box::new(DebuggerPanel),
        Box::new(AnimationPanel),
        Box::new(ToolbarPanel),
        Box::new(MenubarPanel),
        Box::new(StatusBarPanel),
    ]
}
