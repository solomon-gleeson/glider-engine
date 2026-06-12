pub mod registry;
pub mod sandbox;
pub mod script_runtime;

use bevy::prelude::*;

pub use registry::ScriptSource;

use crate::core::ecs::EngineState;

pub struct InstancePlugin;

impl Plugin for InstancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextInstanceId>()
            .init_resource::<DataModelRoot>()
            .init_resource::<ReplicatedStorageRoot>()
            .init_resource::<WorkspaceRoot>()
            .init_resource::<ScriptConsole>()
            .init_resource::<registry::ClassRegistry>()
            .add_systems(Startup, (setup_datamodel, register_core_classes))
            .add_systems(
                Startup,
                load_game_scripts
                    .after(setup_datamodel)
                    .after(register_core_classes),
            )
            .add_systems(OnEnter(EngineState::Running), sandbox::enter_play_mode)
            .add_systems(
                Update,
                sandbox::runtime_script_update.run_if(in_state(EngineState::Running)),
            )
            .add_systems(OnExit(EngineState::Running), sandbox::exit_play_mode);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Info,
    Error,
}

pub struct ConsoleLine {
    pub level: ConsoleLevel,
    pub text: String,
}

#[derive(Resource, Default)]
pub struct ScriptConsole {
    pub lines: Vec<ConsoleLine>,
}

impl ScriptConsole {
    const MAX_LINES: usize = 1000;

    pub fn push(&mut self, level: ConsoleLevel, text: String) {
        self.lines.push(ConsoleLine { level, text });
        if self.lines.len() > Self::MAX_LINES {
            let overflow = self.lines.len() - Self::MAX_LINES;
            self.lines.drain(0..overflow);
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

fn register_core_classes(mut registry: ResMut<registry::ClassRegistry>) {
    registry::register_core_classes(&mut registry);
}

#[derive(Component, Clone)]
pub struct Instance {
    pub id: InstanceId,
    pub name: String,
    pub class_name: String,
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

impl Instance {
    pub fn new(next_id: &mut NextInstanceId, name: &str, class_name: &str) -> Self {
        Self {
            id: next_id.allocate(),
            name: name.into(),
            class_name: class_name.into(),
            parent: None,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(u64);

#[derive(Resource)]
pub struct NextInstanceId(u64);

impl Default for NextInstanceId {
    fn default() -> Self {
        Self(1)
    }
}

impl NextInstanceId {
    pub fn allocate(&mut self) -> InstanceId {
        let id = InstanceId(self.0);
        self.0 += 1;
        id
    }
}

#[derive(Resource, Default)]
pub struct DataModelRoot(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct ReplicatedStorageRoot(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct WorkspaceRoot(pub Option<Entity>);

fn setup_datamodel(mut commands: Commands, mut next_id: ResMut<NextInstanceId>) {
    let dm = Instance {
        id: next_id.allocate(),
        name: "DataModel".into(),
        class_name: "DataModel".into(),
        parent: None,
        children: Vec::new(),
    };

    let dm_entity = commands.spawn(dm.clone()).id();

    let ws = Instance {
        id: next_id.allocate(),
        name: "Workspace".into(),
        class_name: "Workspace".into(),
        parent: Some(dm_entity),
        children: Vec::new(),
    };
    let ws_entity = commands.spawn(ws.clone()).id();

    let rs = Instance {
        id: next_id.allocate(),
        name: "ReplicatedStorage".into(),
        class_name: "ReplicatedStorage".into(),
        parent: Some(dm_entity),
        children: Vec::new(),
    };
    let rs_entity = commands.spawn(rs).id();

    let part = Instance {
        id: next_id.allocate(),
        name: "Part".into(),
        class_name: "Part".into(),
        parent: Some(ws_entity),
        children: Vec::new(),
    };
    let part_entity = commands
        .spawn((
            part,
            Sprite {
                custom_size: Some(Vec2::splat(50.0)),
                color: Color::srgb(0.2, 0.6, 1.0),
                ..default()
            },
            Transform::from_xyz(200.0, 0.0, 0.0),
        ))
        .id();

    commands.entity(dm_entity).insert(Instance {
        children: vec![ws_entity, rs_entity],
        ..dm
    });

    commands.entity(ws_entity).insert(Instance {
        children: {
            let mut c = ws.children.clone();
            c.push(part_entity);
            c
        },
        ..ws
    });

    commands.insert_resource(DataModelRoot(Some(dm_entity)));
    commands.insert_resource(ReplicatedStorageRoot(Some(rs_entity)));
    commands.insert_resource(WorkspaceRoot(Some(ws_entity)));
}

pub fn set_parent(child: Entity, new_parent: Option<Entity>, instances: &mut Query<&mut Instance>) {
    let old_parent = instances.get(child).ok().and_then(|inst| inst.parent);

    if let Some(old) = old_parent
        && let Ok(mut old_inst) = instances.get_mut(old)
    {
        old_inst.children.retain(|c| *c != child);
    }

    if let Some(new) = new_parent
        && let Ok(mut new_inst) = instances.get_mut(new)
    {
        new_inst.children.push(child);
    }

    if let Ok(mut child_inst) = instances.get_mut(child) {
        child_inst.parent = new_parent;
    }
}

pub fn get_children(parent: Entity, instances: &Query<&Instance>) -> Vec<Entity> {
    instances
        .get(parent)
        .map(|inst| inst.children.clone())
        .unwrap_or_default()
}

pub fn find_first_child(
    parent: Entity,
    name: &str,
    instances: &Query<&Instance>,
) -> Option<Entity> {
    instances.get(parent).ok().and_then(|inst| {
        inst.children
            .iter()
            .find(|&&child| instances.get(child).is_ok_and(|c| c.name == name))
            .copied()
    })
}

pub fn get_full_name(entity: Entity, instances: &Query<&Instance>) -> String {
    instances
        .get(entity)
        .map(|inst| {
            let mut parts = vec![inst.name.clone()];
            let mut current = inst.parent;
            while let Some(parent) = current {
                if let Ok(p_inst) = instances.get(parent) {
                    if p_inst.class_name == "DataModel" {
                        break;
                    }
                    parts.push(p_inst.name.clone());
                    current = p_inst.parent;
                } else {
                    break;
                }
            }
            parts.reverse();
            parts.join(".")
        })
        .unwrap_or_default()
}

pub fn destroy_recursive(
    entity: Entity,
    instances: &mut Query<&mut Instance>,
    commands: &mut Commands,
) {
    let children = instances
        .get(entity)
        .map(|inst| inst.children.clone())
        .unwrap_or_default();

    for &child in &children {
        destroy_recursive(child, instances, commands);
    }

    if let Ok(inst) = instances.get(entity)
        && let Some(parent) = inst.parent
        && let Ok(mut parent_inst) = instances.get_mut(parent)
    {
        parent_inst.children.retain(|c| *c != entity);
    }

    commands.entity(entity).despawn();
}

pub fn collect_descendants(entity: Entity, instances: &Query<&Instance>) -> Vec<Entity> {
    let mut result = Vec::new();
    collect_descendants_inner(entity, instances, &mut result);
    result
}

fn collect_descendants_inner(entity: Entity, instances: &Query<&Instance>, out: &mut Vec<Entity>) {
    if let Ok(inst) = instances.get(entity) {
        for &child in &inst.children {
            out.push(child);
            collect_descendants_inner(child, instances, out);
        }
    }
}

pub fn count_descendants(entity: Entity, instances: &Query<&Instance>) -> usize {
    instances
        .get(entity)
        .map(|inst| {
            let mut count = inst.children.len();
            for &child in &inst.children {
                count += count_descendants(child, instances);
            }
            count
        })
        .unwrap_or(0)
}

fn load_game_scripts(world: &mut World) {
    let rs_entity = world.resource::<ReplicatedStorageRoot>().0;
    let Some(parent) = rs_entity else { return };

    let Ok(entries) = std::fs::read_dir("assets/scripts") else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "luau") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Script");

        if name == "main" {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };

        let id = world.resource_mut::<NextInstanceId>().allocate();
        let entity = world
            .spawn((
                Instance {
                    id,
                    name: name.into(),
                    class_name: "Script".into(),
                    parent: Some(parent),
                    children: Vec::new(),
                },
                ScriptSource {
                    source,
                    enabled: true,
                    path: Some(path.to_string_lossy().to_string()),
                },
            ))
            .id();

        if let Some(mut p) = world.entity_mut(parent).get_mut::<Instance>() {
            p.children.push(entity);
        }
    }
}
