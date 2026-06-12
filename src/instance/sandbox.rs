use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::script_runtime::ScriptRuntime;
use super::{DataModelRoot, Instance, ReplicatedStorageRoot, ScriptConsole, ScriptSource};

#[derive(Resource)]
struct PlaySnapshot {
    instances: HashMap<Entity, (Instance, Option<ScriptSource>)>,
    data_model_root: Option<Entity>,
    replicated_storage_root: Option<Entity>,
}

pub fn enter_play_mode(world: &mut World) {
    world.resource_mut::<ScriptConsole>().clear();

    let mut instances = HashMap::new();
    let mut query = world.query::<(Entity, &Instance, Option<&ScriptSource>)>();
    for (entity, inst, src) in query.iter(world) {
        instances.insert(entity, (inst.clone(), src.cloned()));
    }

    let snapshot = PlaySnapshot {
        instances,
        data_model_root: world.resource::<DataModelRoot>().0,
        replicated_storage_root: world.resource::<ReplicatedStorageRoot>().0,
    };
    world.insert_resource(snapshot);

    let runtime = ScriptRuntime::new(world);
    runtime.init_scripts(world);
    world.insert_non_send(runtime);
}

pub fn exit_play_mode(world: &mut World) {
    world.remove_non_send::<ScriptRuntime>();

    let Some(snapshot) = world.remove_resource::<PlaySnapshot>() else {
        return;
    };

    let current: Vec<Entity> = {
        let mut query = world.query_filtered::<Entity, With<Instance>>();
        query.iter(world).collect()
    };
    let alive: HashSet<Entity> = current.iter().copied().collect();
    for &entity in &current {
        if !snapshot.instances.contains_key(&entity) {
            world.despawn(entity);
        }
    }

    let mut remap: HashMap<Entity, Entity> = HashMap::with_capacity(snapshot.instances.len());
    for &old in snapshot.instances.keys() {
        let resolved = if alive.contains(&old) {
            old
        } else {
            world.spawn_empty().id()
        };
        remap.insert(old, resolved);
    }

    for (&old, (inst, src)) in &snapshot.instances {
        let target = remap[&old];
        let restored = Instance {
            parent: inst.parent.and_then(|p| remap.get(&p).copied()),
            children: inst
                .children
                .iter()
                .filter_map(|c| remap.get(c).copied())
                .collect(),
            ..inst.clone()
        };

        let mut entity_mut = world.entity_mut(target);
        entity_mut.insert(restored);

        if !alive.contains(&old)
            && let Some(src) = src
        {
            entity_mut.insert(src.clone());
        }
    }

    if let Some(root) = snapshot.data_model_root
        && let Some(&resolved) = remap.get(&root)
    {
        world.resource_mut::<DataModelRoot>().0 = Some(resolved);
    }
    if let Some(root) = snapshot.replicated_storage_root
        && let Some(&resolved) = remap.get(&root)
    {
        world.resource_mut::<ReplicatedStorageRoot>().0 = Some(resolved);
    }
}

pub fn runtime_script_update(world: &mut World) {
    let dt = world.resource::<Time>().delta_secs();

    if let Some(runtime) = world.remove_non_send::<ScriptRuntime>() {
        runtime.update_scripts(world, dt);
        runtime.dispatch_scene_events(world);
        world.insert_non_send(runtime);
    }
}
