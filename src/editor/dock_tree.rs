#![allow(dead_code)]

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum PanelId {
    Viewport,
    Project,
    Hierarchy,
    FileSystem,
    Inspector,
    Console,
    Output,
    Debugger,
    Animation,
    Toolbar,
    Menubar,
    StatusBar,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum DockNode {
    Split {
        direction: SplitDirection,
        ratio: f32,
        #[reflect(ignore)]
        first: Box<DockNode>,
        #[reflect(ignore)]
        second: Box<DockNode>,
    },
    Tabs {
        tabs: Vec<PanelId>,
        active: usize,
    },
}

impl Default for DockNode {
    fn default() -> Self {
        Self::Tabs {
            tabs: Vec::new(),
            active: 0,
        }
    }
}

impl DockNode {
    pub fn split(direction: SplitDirection, ratio: f32, first: DockNode, second: DockNode) -> Self {
        Self::Split {
            direction,
            ratio: ratio.clamp(0.0, 1.0),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    pub fn tabs(tabs: Vec<PanelId>, active: usize) -> Self {
        let active = if tabs.is_empty() {
            0
        } else {
            active.min(tabs.len() - 1)
        };
        Self::Tabs { tabs, active }
    }

    pub fn find(&self, mut predicate: impl FnMut(&DockNode) -> bool) -> Option<&DockNode> {
        if predicate(self) {
            return Some(self);
        }
        match self {
            DockNode::Split { first, second, .. } => first
                .find(&mut predicate)
                .or_else(|| second.find(&mut predicate)),
            DockNode::Tabs { .. } => None,
        }
    }

    pub fn find_mut(
        &mut self,
        mut predicate: impl FnMut(&DockNode) -> bool,
    ) -> Option<&mut DockNode> {
        if predicate(self) {
            return Some(self);
        }
        match self {
            DockNode::Split { first, second, .. } => first
                .find_mut(&mut predicate)
                .or_else(|| second.find_mut(&mut predicate)),
            DockNode::Tabs { .. } => None,
        }
    }

    pub fn walk_mut(&mut self, f: &mut impl FnMut(&mut DockNode)) {
        f(self);
        match self {
            DockNode::Split { first, second, .. } => {
                first.walk_mut(f);
                second.walk_mut(f);
            }
            DockNode::Tabs { .. } => {}
        }
    }

    pub fn contains_tab(&self, id: PanelId) -> bool {
        match self {
            DockNode::Split { first, second, .. } => {
                first.contains_tab(id) || second.contains_tab(id)
            }
            DockNode::Tabs { tabs, .. } => tabs.contains(&id),
        }
    }

    pub fn add_tab_beside(&mut self, anchor: PanelId, new: PanelId) -> bool {
        match self {
            DockNode::Split { first, second, .. } => {
                first.add_tab_beside(anchor, new) || second.add_tab_beside(anchor, new)
            }
            DockNode::Tabs { tabs, .. } => {
                if tabs.contains(&anchor) && !tabs.contains(&new) {
                    tabs.push(new);
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct DockTree {
    pub root: DockNode,
}

impl DockTree {
    pub fn find_node_mut(
        &mut self,
        predicate: impl Fn(&DockNode) -> bool,
    ) -> Option<&mut DockNode> {
        self.root.find_mut(predicate)
    }

    pub fn walk_mut(&mut self, f: &mut impl FnMut(&mut DockNode)) {
        self.root.walk_mut(f);
    }
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct PanelVisibility(pub HashMap<PanelId, bool>);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Reflect, Serialize, Deserialize)]
pub enum ViewportToolMode {
    Select,
    Move,
    Rotate,
    Scale,
}

impl ViewportToolMode {
    pub fn icon(&self) -> &str {
        match self {
            Self::Select => "\u{2B9B}",
            Self::Move => "\u{2726}",
            Self::Rotate => "\u{21BB}",
            Self::Scale => "\u{29FA}",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Select => "Select",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::Scale => "Scale",
        }
    }
}

#[derive(Resource, Serialize, Deserialize)]
pub struct EditorLayout {
    pub dock_tree: DockTree,
    pub visibility: PanelVisibility,
    pub active_tabs: HashMap<PanelId, usize>,
    pub viewport_zoom: f32,
    pub tool_mode: ViewportToolMode,
}

impl Default for EditorLayout {
    fn default() -> Self {
        let left_dock = DockNode::split(
            SplitDirection::Vertical,
            0.55,
            DockNode::tabs(vec![PanelId::Project, PanelId::Hierarchy], 0),
            DockNode::tabs(vec![PanelId::FileSystem], 0),
        );

        let bottom = DockNode::tabs(
            vec![
                PanelId::Output,
                PanelId::Debugger,
                PanelId::Animation,
                PanelId::Console,
            ],
            0,
        );

        let center = DockNode::split(
            SplitDirection::Vertical,
            0.956,
            DockNode::tabs(vec![PanelId::Viewport], 0),
            bottom,
        );

        let center_right = DockNode::split(
            SplitDirection::Horizontal,
            0.78,
            center,
            DockNode::tabs(vec![PanelId::Inspector], 0),
        );

        let root = DockNode::split(SplitDirection::Horizontal, 0.2, left_dock, center_right);

        Self {
            dock_tree: DockTree { root },
            visibility: PanelVisibility::default(),
            active_tabs: HashMap::new(),
            viewport_zoom: 1.0,
            tool_mode: ViewportToolMode::Select,
        }
    }
}
