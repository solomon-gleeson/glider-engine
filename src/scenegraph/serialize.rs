#![allow(dead_code)]

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{NodeId, SceneGraph, SceneNode, SceneNodeKind, SceneSpawned};

pub const SCENE_PATH: &str = "assets/scenes/main.scene.ron";

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedTransform {
    pub pos: [f32; 3],
    pub rot_deg: f32,
    pub scale: [f32; 2],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedSprite {
    pub size: Option<[f32; 2]>,
    pub color: [f32; 4],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedNode {
    pub id: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub collection: bool,
    pub adopted: bool,
    pub transform: Option<SavedTransform>,
    pub sprite: Option<SavedSprite>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneFile {
    pub next_id: u64,
    pub nodes: Vec<SavedNode>,
}

pub type SubtreeSnapshot = SceneFile;

fn capture_node(world: &World, graph: &SceneGraph, node: &SceneNode) -> SavedNode {
    let mut transform = None;
    let mut sprite = None;
    let mut adopted = false;

    if let Some(entity) = node.entity()
        && let Ok(entity_ref) = world.get_entity(entity)
    {
        adopted = entity_ref.get::<crate::instance::Instance>().is_some();
        if let Some(t) = entity_ref.get::<Transform>() {
            transform = Some(SavedTransform {
                pos: [t.translation.x, t.translation.y, t.translation.z],
                rot_deg: t.rotation.to_euler(EulerRot::XYZ).2.to_degrees(),
                scale: [t.scale.x, t.scale.y],
            });
        }
        if let Some(s) = entity_ref.get::<Sprite>() {
            let c = s.color.to_srgba();
            sprite = Some(SavedSprite {
                size: s.custom_size.map(|v| [v.x, v.y]),
                color: [c.red, c.green, c.blue, c.alpha],
            });
        }
    }

    SavedNode {
        id: node.id.0,
        name: node.name.clone(),
        parent: node.parent.filter(|p| *p != graph.root()).map(|p| p.0),
        collection: node.is_collection(),
        adopted,
        transform,
        sprite,
    }
}

fn capture_subtree(world: &World, graph: &SceneGraph, start: NodeId, include_start: bool) -> Vec<SavedNode> {
    let mut out = Vec::new();
    let mut stack = if include_start {
        vec![start]
    } else {
        graph.children_of(start)
    };
    stack.reverse();
    while let Some(id) = stack.pop() {
        let Some(node) = graph.get(id) else { continue };
        let mut saved = capture_node(world, graph, node);
        if include_start && id == start {
            saved.parent = None;
        }
        out.push(saved);
        let mut children = graph.children_of(id);
        children.reverse();
        stack.extend(children);
    }
    out
}

pub fn save_scene(world: &World, graph: &SceneGraph) -> SceneFile {
    let nodes = capture_subtree(world, graph, graph.root(), false);
    let next_id = nodes.iter().map(|n| n.id + 1).max().unwrap_or(1);
    SceneFile { next_id, nodes }
}

pub fn snapshot_subtree(world: &World, graph: &SceneGraph, node: NodeId) -> SubtreeSnapshot {
    let nodes = capture_subtree(world, graph, node, true);
    let next_id = nodes.iter().map(|n| n.id + 1).max().unwrap_or(1);
    SubtreeSnapshot { next_id, nodes }
}

fn spawn_saved_entity(world: &mut World, saved: &SavedNode) -> Entity {
    let mut entity = world.spawn(SceneSpawned);
    if let Some(t) = &saved.transform {
        entity.insert(
            Transform::from_xyz(t.pos[0], t.pos[1], t.pos[2])
                .with_rotation(Quat::from_rotation_z(t.rot_deg.to_radians()))
                .with_scale(Vec3::new(t.scale[0], t.scale[1], 1.0)),
        );
    }
    if let Some(s) = &saved.sprite {
        entity.insert(Sprite {
            custom_size: s.size.map(|v| Vec2::new(v[0], v[1])),
            color: Color::srgba(s.color[0], s.color[1], s.color[2], s.color[3]),
            ..default()
        });
    }
    entity.id()
}

pub fn restore_subtree(
    world: &mut World,
    snapshot: &SubtreeSnapshot,
    parent: NodeId,
    preserve_ids: bool,
) -> Option<NodeId> {
    let mut first: Option<NodeId> = None;
    world.resource_scope::<SceneGraph, _>(|world, mut graph| {
        let mut id_map: HashMap<u64, NodeId> = HashMap::new();
        for saved in &snapshot.nodes {
            let id = if preserve_ids {
                graph.ensure_next_id_above(saved.id);
                NodeId(saved.id)
            } else {
                graph.allocate_id()
            };
            let mapped_parent = saved
                .parent
                .and_then(|p| id_map.get(&p).copied())
                .unwrap_or(parent);
            let kind = if saved.collection {
                SceneNodeKind::Collection
            } else {
                SceneNodeKind::Entity(spawn_saved_entity(world, saved))
            };
            graph.insert_node(SceneNode {
                id,
                name: saved.name.clone(),
                parent: Some(mapped_parent),
                children: Vec::new(),
                kind,
            });
            id_map.insert(saved.id, id);
            if first.is_none() {
                first = Some(id);
            }
        }
    });
    first
}

pub fn write_scene(world: &World, graph: &SceneGraph, path: &str) -> Result<(), String> {
    let file = save_scene(world, graph);
    let text = ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())?;
    if let Some(dir) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn load_scene(world: &mut World, path: &str) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: SceneFile = ron::from_str(&text).map_err(|e| e.to_string())?;

    let spawned: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, With<SceneSpawned>>();
        q.iter(world).collect()
    };
    for e in spawned {
        world.despawn(e);
    }

    let mut count = 0;
    world.resource_scope::<SceneGraph, _>(|world, mut graph| {
        *graph = SceneGraph::new();
        graph.ensure_next_id_above(file.next_id);
        let mut id_map: HashMap<u64, NodeId> = HashMap::new();
        for saved in &file.nodes {
            if saved.adopted {
                continue;
            }
            let mapped_parent = saved
                .parent
                .and_then(|p| id_map.get(&p).copied())
                .unwrap_or(graph.root());
            let kind = if saved.collection {
                SceneNodeKind::Collection
            } else {
                SceneNodeKind::Entity(spawn_saved_entity(world, saved))
            };
            graph.ensure_next_id_above(saved.id);
            graph.insert_node(SceneNode {
                id: NodeId(saved.id),
                name: saved.name.clone(),
                parent: Some(mapped_parent),
                children: Vec::new(),
                kind,
            });
            id_map.insert(saved.id, NodeId(saved.id));
            count += 1;
        }
    });
    Ok(count)
}

pub fn save_prefab(world: &World, graph: &SceneGraph, node: NodeId, path: &str) -> Result<(), String> {
    let snapshot = snapshot_subtree(world, graph, node);
    let text = ron::ser::to_string_pretty(&snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())?;
    if let Some(dir) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn instantiate_prefab(world: &mut World, path: &str, parent: NodeId) -> Result<NodeId, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let snapshot: SubtreeSnapshot = ron::from_str(&text).map_err(|e| e.to_string())?;
    restore_subtree(world, &snapshot, parent, false).ok_or_else(|| "empty prefab".to_string())
}
