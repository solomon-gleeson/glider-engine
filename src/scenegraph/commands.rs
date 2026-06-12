#![allow(dead_code)]

use bevy::prelude::*;

use super::serialize::{self, SubtreeSnapshot};
use super::{NodeId, SceneGraph};
use crate::instance::Instance;

pub enum GraphOp {
    Rename {
        node: NodeId,
        from: String,
        to: String,
    },
    Reparent {
        node: NodeId,
        from: NodeId,
        to: NodeId,
    },
    AddCollection {
        node: NodeId,
        parent: NodeId,
        name: String,
    },
    Remove {
        snapshot: SubtreeSnapshot,
        parent: NodeId,
    },
}

#[derive(Resource, Default)]
pub struct GraphHistory {
    undo: Vec<GraphOp>,
    redo: Vec<GraphOp>,
}

impl GraphHistory {
    const LIMIT: usize = 256;

    fn record(&mut self, op: GraphOp) {
        self.undo.push(op);
        if self.undo.len() > Self::LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

fn sync_instance_name(world: &mut World, graph: &SceneGraph, node: NodeId) {
    let Some(entity) = graph.entity_of(node) else {
        return;
    };
    let Some(name) = graph.get(node).map(|n| n.name.clone()) else {
        return;
    };
    if let Ok(mut entity_mut) = world.get_entity_mut(entity)
        && let Some(mut inst) = entity_mut.get_mut::<Instance>()
        && inst.name != name
    {
        inst.name = name;
    }
}

fn sync_instance_parent(world: &mut World, graph: &SceneGraph, node: NodeId, new_parent: NodeId) {
    let Some(child_entity) = graph.entity_of(node) else {
        return;
    };
    let Some(parent_entity) = graph.entity_of(new_parent) else {
        return;
    };
    let has_instance = world
        .get_entity(parent_entity)
        .is_ok_and(|e| e.contains::<Instance>())
        && world
            .get_entity(child_entity)
            .is_ok_and(|e| e.contains::<Instance>());
    if !has_instance {
        return;
    }
    let old_parent = world
        .entity(child_entity)
        .get::<Instance>()
        .and_then(|i| i.parent);
    if old_parent == Some(parent_entity) {
        return;
    }
    if let Some(old) = old_parent
        && let Some(mut p) = world.entity_mut(old).get_mut::<Instance>()
    {
        p.children.retain(|c| *c != child_entity);
    }
    if let Some(mut p) = world.entity_mut(parent_entity).get_mut::<Instance>() {
        p.children.push(child_entity);
    }
    if let Some(mut c) = world.entity_mut(child_entity).get_mut::<Instance>() {
        c.parent = Some(parent_entity);
    }
}

pub fn do_rename(world: &mut World, node: NodeId, new_name: &str) {
    let (from, to) = {
        let mut graph = world.resource_mut::<SceneGraph>();
        let Some(from) = graph.get(node).map(|n| n.name.clone()) else {
            return;
        };
        graph.rename(node, new_name);
        let Some(to) = graph.get(node).map(|n| n.name.clone()) else {
            return;
        };
        (from, to)
    };
    if from == to {
        return;
    }
    world.resource_scope::<SceneGraph, _>(|world, graph| {
        sync_instance_name(world, &graph, node);
    });
    world
        .resource_mut::<GraphHistory>()
        .record(GraphOp::Rename { node, from, to });
}

pub fn do_reparent(world: &mut World, node: NodeId, new_parent: NodeId) -> bool {
    let from = {
        let mut graph = world.resource_mut::<SceneGraph>();
        let Some(from) = graph.get(node).and_then(|n| n.parent) else {
            return false;
        };
        if from == new_parent {
            return false;
        }
        if graph.reparent(node, new_parent).is_err() {
            return false;
        }
        from
    };
    world.resource_scope::<SceneGraph, _>(|world, graph| {
        sync_instance_parent(world, &graph, node, new_parent);
    });
    world.resource_mut::<GraphHistory>().record(GraphOp::Reparent {
        node,
        from,
        to: new_parent,
    });
    true
}

pub fn do_add_collection(world: &mut World, name: &str, parent: Option<NodeId>) -> NodeId {
    let (node, parent, name) = {
        let mut graph = world.resource_mut::<SceneGraph>();
        let node = graph.add_collection(name, parent);
        let parent = graph.get(node).and_then(|n| n.parent).unwrap_or(graph.root());
        let name = graph.get(node).map(|n| n.name.clone()).unwrap_or_default();
        (node, parent, name)
    };
    world
        .resource_mut::<GraphHistory>()
        .record(GraphOp::AddCollection { node, parent, name });
    node
}

pub fn do_remove(world: &mut World, node: NodeId) {
    let snapshot;
    let parent;
    {
        let graph = world.resource::<SceneGraph>();
        if node == graph.root() || !graph.contains(node) {
            return;
        }
        snapshot = serialize::snapshot_subtree(world, graph, node);
        parent = graph.get(node).and_then(|n| n.parent).unwrap_or(graph.root());
    }
    let entities = world.resource_mut::<SceneGraph>().remove(node);
    for e in entities {
        if world.get_entity(e).is_ok() {
            world.despawn(e);
        }
    }
    world
        .resource_mut::<GraphHistory>()
        .record(GraphOp::Remove { snapshot, parent });
}

pub fn undo(world: &mut World) {
    let Some(op) = world.resource_mut::<GraphHistory>().undo.pop() else {
        return;
    };
    let inverse = apply_inverse(world, op);
    world.resource_mut::<GraphHistory>().redo.push(inverse);
}

pub fn redo(world: &mut World) {
    let Some(op) = world.resource_mut::<GraphHistory>().redo.pop() else {
        return;
    };
    let inverse = apply_inverse(world, op);
    world.resource_mut::<GraphHistory>().undo.push(inverse);
}

fn apply_inverse(world: &mut World, op: GraphOp) -> GraphOp {
    match op {
        GraphOp::Rename { node, from, to } => {
            world.resource_mut::<SceneGraph>().rename(node, &from);
            world.resource_scope::<SceneGraph, _>(|world, graph| {
                sync_instance_name(world, &graph, node);
            });
            GraphOp::Rename {
                node,
                from: to,
                to: from,
            }
        }
        GraphOp::Reparent { node, from, to } => {
            let _ = world.resource_mut::<SceneGraph>().reparent(node, from);
            world.resource_scope::<SceneGraph, _>(|world, graph| {
                sync_instance_parent(world, &graph, node, from);
            });
            GraphOp::Reparent {
                node,
                from: to,
                to: from,
            }
        }
        GraphOp::AddCollection { node, parent, name } => {
            let snapshot = {
                let graph = world.resource::<SceneGraph>();
                serialize::snapshot_subtree(world, graph, node)
            };
            let entities = world.resource_mut::<SceneGraph>().remove(node);
            for e in entities {
                if world.get_entity(e).is_ok() {
                    world.despawn(e);
                }
            }
            let _ = name;
            GraphOp::Remove { snapshot, parent }
        }
        GraphOp::Remove { snapshot, parent } => {
            let restored = serialize::restore_subtree(world, &snapshot, parent, true);
            match restored {
                Some(node) => {
                    let name = world
                        .resource::<SceneGraph>()
                        .get(node)
                        .map(|n| n.name.clone())
                        .unwrap_or_default();
                    GraphOp::AddCollection { node, parent, name }
                }
                None => GraphOp::Remove { snapshot, parent },
            }
        }
    }
}
