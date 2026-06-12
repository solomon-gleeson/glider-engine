#![allow(dead_code)]

pub mod commands;
pub mod luau;
pub mod serialize;

use std::collections::HashMap;

use bevy::prelude::*;

use crate::instance::Instance;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SceneNodeKind {
    Entity(Entity),
    Collection,
}

#[derive(Clone, Debug)]
pub struct SceneNode {
    pub id: NodeId,
    pub name: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: SceneNodeKind,
}

impl SceneNode {
    pub fn is_collection(&self) -> bool {
        matches!(self.kind, SceneNodeKind::Collection)
    }

    pub fn entity(&self) -> Option<Entity> {
        match self.kind {
            SceneNodeKind::Entity(e) => Some(e),
            SceneNodeKind::Collection => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SceneGraphEvent {
    ChildAdded { parent: NodeId, child: NodeId },
    ChildRemoved { parent: NodeId, child: NodeId },
    ParentChanged {
        node: NodeId,
        old: Option<NodeId>,
        new: Option<NodeId>,
    },
    PathChanged { node: NodeId, path: String },
}

#[derive(Component)]
pub struct SceneSpawned;

#[derive(Resource)]
pub struct SceneGraph {
    nodes: HashMap<NodeId, SceneNode>,
    root: NodeId,
    by_entity: HashMap<Entity, NodeId>,
    path_cache: HashMap<String, NodeId>,
    next_id: u64,
    pending: Vec<SceneGraphEvent>,
}

impl SceneGraph {
    pub fn new() -> Self {
        let root = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(
            root,
            SceneNode {
                id: root,
                name: "Scene".to_string(),
                parent: None,
                children: Vec::new(),
                kind: SceneNodeKind::Collection,
            },
        );
        Self {
            nodes,
            root,
            by_entity: HashMap::new(),
            path_cache: HashMap::new(),
            next_id: 1,
            pending: Vec::new(),
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn get(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn node_of_entity(&self, entity: Entity) -> Option<NodeId> {
        self.by_entity.get(&entity).copied()
    }

    pub fn entity_of(&self, id: NodeId) -> Option<Entity> {
        self.nodes.get(&id).and_then(|n| n.entity())
    }

    pub fn children_of(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(&id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SceneNode> {
        self.nodes.values()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1
    }

    pub fn take_events(&mut self) -> Vec<SceneGraphEvent> {
        std::mem::take(&mut self.pending)
    }

    pub fn ensure_next_id_above(&mut self, id: u64) {
        if self.next_id <= id {
            self.next_id = id + 1;
        }
    }

    pub(crate) fn allocate_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    fn sanitise_name(name: &str) -> String {
        let cleaned: String = name.replace('/', "_");
        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            "Node".to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn unique_name(&self, parent: NodeId, base: &str, exclude: Option<NodeId>) -> String {
        let base = Self::sanitise_name(base);
        let siblings: Vec<String> = self
            .children_of(parent)
            .iter()
            .filter(|c| Some(**c) != exclude)
            .filter_map(|c| self.nodes.get(c).map(|n| n.name.clone()))
            .collect();
        if !siblings.contains(&base) {
            return base;
        }
        for i in 2..u32::MAX {
            let candidate = format!("{base}_{i}");
            if !siblings.contains(&candidate) {
                return candidate;
            }
        }
        base
    }

    pub fn add_collection(&mut self, name: &str, parent: Option<NodeId>) -> NodeId {
        let parent = parent.filter(|p| self.contains(*p)).unwrap_or(self.root);
        let id = self.allocate_id();
        self.insert_node(SceneNode {
            id,
            name: name.to_string(),
            parent: Some(parent),
            children: Vec::new(),
            kind: SceneNodeKind::Collection,
        });
        id
    }

    pub fn adopt_entity(&mut self, entity: Entity, name: &str, parent: Option<NodeId>) -> NodeId {
        if let Some(existing) = self.node_of_entity(entity) {
            return existing;
        }
        let parent = parent.filter(|p| self.contains(*p)).unwrap_or(self.root);
        let id = self.allocate_id();
        self.insert_node(SceneNode {
            id,
            name: name.to_string(),
            parent: Some(parent),
            children: Vec::new(),
            kind: SceneNodeKind::Entity(entity),
        });
        id
    }

    pub fn insert_node(&mut self, mut node: SceneNode) {
        let parent = node.parent.filter(|p| self.contains(*p)).unwrap_or(self.root);
        node.parent = Some(parent);
        node.name = self.unique_name(parent, &node.name, Some(node.id));
        node.children.retain(|c| self.contains(*c));
        let id = node.id;
        self.ensure_next_id_above(id.0);
        if let SceneNodeKind::Entity(e) = node.kind {
            self.by_entity.insert(e, id);
        }
        self.nodes.insert(id, node);
        if let Some(p) = self.nodes.get_mut(&parent)
            && !p.children.contains(&id)
        {
            p.children.push(id);
        }
        self.path_cache.clear();
        self.pending.push(SceneGraphEvent::ChildAdded {
            parent,
            child: id,
        });
        self.emit_path_changed(id);
    }

    pub fn remove(&mut self, id: NodeId) -> Vec<Entity> {
        if id == self.root || !self.contains(id) {
            return Vec::new();
        }
        let mut removed_entities = Vec::new();
        let mut stack = vec![id];
        let mut order = Vec::new();
        while let Some(n) = stack.pop() {
            order.push(n);
            stack.extend(self.children_of(n));
        }
        let parent = self.nodes.get(&id).and_then(|n| n.parent);
        for n in order.iter().rev() {
            if let Some(node) = self.nodes.remove(n)
                && let Some(e) = node.entity()
            {
                self.by_entity.remove(&e);
                removed_entities.push(e);
            }
        }
        if let Some(p) = parent
            && let Some(pn) = self.nodes.get_mut(&p)
        {
            pn.children.retain(|c| *c != id);
        }
        self.path_cache.clear();
        if let Some(p) = parent {
            self.pending.push(SceneGraphEvent::ChildRemoved {
                parent: p,
                child: id,
            });
        }
        removed_entities
    }

    pub fn reparent(&mut self, id: NodeId, new_parent: NodeId) -> Result<(), &'static str> {
        if id == self.root {
            return Err("cannot reparent the scene root");
        }
        if !self.contains(id) || !self.contains(new_parent) {
            return Err("unknown node");
        }
        if id == new_parent || self.is_ancestor(id, new_parent) {
            return Err("cannot parent a node beneath itself");
        }
        let old_parent = self.nodes.get(&id).and_then(|n| n.parent);
        if old_parent == Some(new_parent) {
            return Ok(());
        }
        if let Some(old) = old_parent
            && let Some(p) = self.nodes.get_mut(&old)
        {
            p.children.retain(|c| *c != id);
        }
        let name = self
            .nodes
            .get(&id)
            .map(|n| n.name.clone())
            .unwrap_or_default();
        let unique = self.unique_name(new_parent, &name, Some(id));
        if let Some(node) = self.nodes.get_mut(&id) {
            node.parent = Some(new_parent);
            node.name = unique;
        }
        if let Some(p) = self.nodes.get_mut(&new_parent) {
            p.children.push(id);
        }
        self.path_cache.clear();
        if let Some(old) = old_parent {
            self.pending.push(SceneGraphEvent::ChildRemoved {
                parent: old,
                child: id,
            });
        }
        self.pending.push(SceneGraphEvent::ChildAdded {
            parent: new_parent,
            child: id,
        });
        self.pending.push(SceneGraphEvent::ParentChanged {
            node: id,
            old: old_parent,
            new: Some(new_parent),
        });
        self.emit_path_changed(id);
        Ok(())
    }

    pub fn rename(&mut self, id: NodeId, new_name: &str) {
        if !self.contains(id) {
            return;
        }
        let parent = self.nodes.get(&id).and_then(|n| n.parent);
        let unique = match parent {
            Some(p) => self.unique_name(p, new_name, Some(id)),
            None => Self::sanitise_name(new_name),
        };
        if let Some(node) = self.nodes.get_mut(&id) {
            if node.name == unique {
                return;
            }
            node.name = unique;
        }
        self.path_cache.clear();
        self.emit_path_changed(id);
    }

    fn is_ancestor(&self, candidate: NodeId, of: NodeId) -> bool {
        let mut current = self.nodes.get(&of).and_then(|n| n.parent);
        while let Some(p) = current {
            if p == candidate {
                return true;
            }
            current = self.nodes.get(&p).and_then(|n| n.parent);
        }
        false
    }

    pub fn path_of(&self, id: NodeId) -> String {
        let mut parts = Vec::new();
        let mut current = Some(id);
        while let Some(c) = current {
            let Some(node) = self.nodes.get(&c) else {
                break;
            };
            parts.push(node.name.clone());
            current = node.parent;
        }
        parts.reverse();
        parts.join("/")
    }

    pub fn find_path(&mut self, path: &str) -> Option<NodeId> {
        let normalised = path.trim().trim_matches('/');
        if normalised.is_empty() {
            return Some(self.root);
        }
        if let Some(&cached) = self.path_cache.get(normalised) {
            if self.contains(cached) && self.path_of(cached) == self.absolute(normalised) {
                return Some(cached);
            }
            self.path_cache.remove(normalised);
        }
        let found = self.resolve_path(self.root, normalised);
        if let Some(id) = found {
            self.path_cache.insert(normalised.to_string(), id);
        }
        found
    }

    fn absolute(&self, normalised: &str) -> String {
        let root_name = self
            .nodes
            .get(&self.root)
            .map(|n| n.name.as_str())
            .unwrap_or("Scene");
        if normalised == root_name || normalised.starts_with(&format!("{root_name}/")) {
            normalised.to_string()
        } else {
            format!("{root_name}/{normalised}")
        }
    }

    pub fn resolve_path(&self, from: NodeId, path: &str) -> Option<NodeId> {
        let mut current = from;
        let mut segments = path.split('/').filter(|s| !s.is_empty()).peekable();
        if let Some(first) = segments.peek()
            && current == self.root
            && self.nodes.get(&self.root).is_some_and(|r| r.name == *first)
        {
            segments.next();
        }
        for segment in segments {
            let children = self.children_of(current);
            let next = children
                .iter()
                .find(|c| self.nodes.get(c).is_some_and(|n| n.name == segment));
            match next {
                Some(n) => current = *n,
                None => return None,
            }
        }
        Some(current)
    }

    pub fn find_child(&self, parent: NodeId, name: &str) -> Option<NodeId> {
        self.children_of(parent)
            .into_iter()
            .find(|c| self.nodes.get(c).is_some_and(|n| n.name == name))
    }

    fn emit_path_changed(&mut self, id: NodeId) {
        let mut stack = vec![id];
        while let Some(n) = stack.pop() {
            let path = self.path_of(n);
            self.pending.push(SceneGraphEvent::PathChanged { node: n, path });
            stack.extend(self.children_of(n));
        }
    }

    pub fn descendants(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = self.children_of(id);
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(self.children_of(n));
        }
        out
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sync_scenegraph(world: &mut World) {
    world.resource_scope::<SceneGraph, _>(|world, mut graph| {
        let dead: Vec<NodeId> = graph
            .iter()
            .filter_map(|n| {
                n.entity()
                    .filter(|e| world.get_entity(*e).is_err())
                    .map(|_| n.id)
            })
            .collect();
        for id in dead {
            if graph.contains(id) {
                graph.remove(id);
            }
        }

        let mut instances: Vec<(Entity, String, String, Option<Entity>)> = Vec::new();
        let mut query = world.query::<(Entity, &Instance)>();
        for (entity, inst) in query.iter(world) {
            instances.push((
                entity,
                inst.name.clone(),
                inst.class_name.clone(),
                inst.parent,
            ));
        }

        let class_of: HashMap<Entity, &str> = instances
            .iter()
            .map(|(e, _, c, _)| (*e, c.as_str()))
            .collect();

        let mut progress = true;
        while progress {
            progress = false;
            for (entity, name, class, parent) in &instances {
                if class == "DataModel" || graph.node_of_entity(*entity).is_some() {
                    continue;
                }
                let parent_node = match parent {
                    None => Some(graph.root()),
                    Some(p) => match class_of.get(p) {
                        Some(&"DataModel") | None => Some(graph.root()),
                        Some(_) => graph.node_of_entity(*p),
                    },
                };
                if let Some(pn) = parent_node {
                    graph.adopt_entity(*entity, name, Some(pn));
                    progress = true;
                }
            }
        }

        for (entity, name, class, _) in &instances {
            if class == "DataModel" {
                continue;
            }
            if let Some(id) = graph.node_of_entity(*entity)
                && let Some(node) = graph.get(id)
                && node.name != *name
            {
                graph.rename(id, name);
            }
        }

        let orphan_sprites: Vec<Entity> = {
            let mut q = world.query_filtered::<Entity, (With<Sprite>, Without<Instance>)>();
            q.iter(world).collect()
        };
        for entity in orphan_sprites {
            if graph.node_of_entity(entity).is_none() {
                graph.adopt_entity(entity, "Sprite", None);
            }
        }
    });
}

fn drain_events_when_idle(world: &mut World) {
    let runtime_active = world
        .get_non_send::<crate::instance::script_runtime::ScriptRuntime>()
        .is_some();
    if !runtime_active {
        world.resource_mut::<SceneGraph>().take_events();
    }
}

pub struct SceneGraphPlugin;

impl Plugin for SceneGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneGraph>()
            .init_resource::<commands::GraphHistory>()
            .add_systems(Update, (sync_scenegraph, drain_events_when_idle).chain());
    }
}
