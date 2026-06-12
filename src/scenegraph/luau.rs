#![allow(dead_code)]

use std::cell::RefCell;

use bevy::prelude::*;
use mluau::UserData;
use mluau::prelude::*;

use super::{NodeId, SceneGraph, SceneGraphEvent, serialize};
use crate::instance::Instance;
use crate::instance::script_runtime::world_from_lua;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
    ChildAdded,
    ChildRemoved,
    ParentChanged,
    PathChanged,
}

struct Connection {
    node: NodeId,
    kind: SignalKind,
    func: LuaFunction,
}

#[derive(Default)]
pub struct SignalStore {
    connections: RefCell<Vec<Connection>>,
}

#[derive(Clone, Copy)]
pub struct SceneNodeHandle {
    pub id: NodeId,
}

#[derive(Clone, Copy)]
pub struct SignalHandle {
    node: NodeId,
    kind: SignalKind,
}

fn graph_of(world: &mut World) -> Mut<'_, SceneGraph> {
    world.resource_mut::<SceneGraph>()
}

fn handle_or_nil(lua: &Lua, id: Option<NodeId>) -> LuaResult<LuaValue> {
    match id {
        Some(id) => Ok(LuaValue::UserData(
            lua.create_userdata(SceneNodeHandle { id })?,
        )),
        None => Ok(LuaValue::Nil),
    }
}

fn sync_instance_name(world: &mut World, node: NodeId) {
    let (entity, name) = {
        let graph = world.resource::<SceneGraph>();
        let Some(entity) = graph.entity_of(node) else {
            return;
        };
        let Some(name) = graph.get(node).map(|n| n.name.clone()) else {
            return;
        };
        (entity, name)
    };
    if let Ok(mut entity_mut) = world.get_entity_mut(entity)
        && let Some(mut inst) = entity_mut.get_mut::<Instance>()
        && inst.name != name
    {
        inst.name = name;
    }
}

impl UserData for SceneNodeHandle {
    fn add_fields<F: mluau::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |lua, h| {
            let world = world_from_lua(lua);
            Ok(graph_of(world)
                .get(h.id)
                .map(|n| n.name.clone())
                .unwrap_or_default())
        });
        fields.add_field_method_set("Name", |lua, h, name: String| {
            let world = world_from_lua(lua);
            graph_of(world).rename(h.id, &name);
            sync_instance_name(world, h.id);
            Ok(())
        });
        fields.add_field_method_get("Path", |lua, h| {
            let world = world_from_lua(lua);
            Ok(graph_of(world).path_of(h.id))
        });
        fields.add_field_method_get("Parent", |lua, h| {
            let world = world_from_lua(lua);
            let parent = graph_of(world).get(h.id).and_then(|n| n.parent);
            handle_or_nil(lua, parent)
        });
        fields.add_field_method_set("Parent", |lua, h, value: Option<LuaAnyUserData>| {
            let world = world_from_lua(lua);
            let target = match value {
                Some(ud) => ud.borrow::<SceneNodeHandle>().map(|p| p.id).map_err(|_| {
                    mluau::Error::RuntimeError("Parent must be a scene node".into())
                })?,
                None => graph_of(world).root(),
            };
            graph_of(world)
                .reparent(h.id, target)
                .map_err(|e| mluau::Error::RuntimeError(e.to_string()))?;
            Ok(())
        });
        fields.add_field_method_get("IsCollection", |lua, h| {
            let world = world_from_lua(lua);
            Ok(graph_of(world)
                .get(h.id)
                .map(|n| n.is_collection())
                .unwrap_or(false))
        });
        for (label, kind) in [
            ("ChildAdded", SignalKind::ChildAdded),
            ("ChildRemoved", SignalKind::ChildRemoved),
            ("ParentChanged", SignalKind::ParentChanged),
            ("PathChanged", SignalKind::PathChanged),
        ] {
            fields.add_field_method_get(label, move |lua, h| {
                lua.create_userdata(SignalHandle { node: h.id, kind })
            });
        }
    }

    fn add_methods<M: mluau::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("GetChildren", |lua, h, ()| {
            let world = world_from_lua(lua);
            let children = graph_of(world).children_of(h.id);
            let table = lua.create_table()?;
            for (i, child) in children.iter().enumerate() {
                table.set(i + 1, lua.create_userdata(SceneNodeHandle { id: *child })?)?;
            }
            Ok(table)
        });
        methods.add_method("FindFirstChild", |lua, h, name: String| {
            let world = world_from_lua(lua);
            let found = graph_of(world).find_child(h.id, &name);
            handle_or_nil(lua, found)
        });
        methods.add_method("Find", |lua, h, path: String| {
            let world = world_from_lua(lua);
            let found = graph_of(world).resolve_path(h.id, &path);
            handle_or_nil(lua, found)
        });
        methods.add_method("CreateCollection", |lua, h, name: String| {
            let world = world_from_lua(lua);
            let id = graph_of(world).add_collection(&name, Some(h.id));
            lua.create_userdata(SceneNodeHandle { id })
        });
        methods.add_method("IsAlive", |lua, h, ()| {
            let world = world_from_lua(lua);
            Ok(graph_of(world).contains(h.id))
        });
        methods.add_method("Destroy", |lua, h, ()| {
            let world = world_from_lua(lua);
            let entities = graph_of(world).remove(h.id);
            for e in entities {
                if world.get_entity(e).is_ok() {
                    world.despawn(e);
                }
            }
            Ok(())
        });
    }
}

impl UserData for SignalHandle {
    fn add_methods<M: mluau::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Connect", |lua, h, func: LuaFunction| {
            if let Some(store) = lua.app_data_ref::<SignalStore>() {
                store.connections.borrow_mut().push(Connection {
                    node: h.node,
                    kind: h.kind,
                    func,
                });
            }
            Ok(())
        });
    }
}

pub fn install_scene_api(lua: &Lua) {
    lua.set_app_data(SignalStore::default());
    let globals = lua.globals();
    let scene = lua.create_table().unwrap();

    scene
        .set(
            "Root",
            lua.create_function(|lua, ()| {
                let world = world_from_lua(lua);
                let root = graph_of(world).root();
                lua.create_userdata(SceneNodeHandle { id: root })
            })
            .unwrap(),
        )
        .unwrap();

    scene
        .set(
            "Find",
            lua.create_function(|lua, path: String| {
                let world = world_from_lua(lua);
                let found = graph_of(world).find_path(&path);
                handle_or_nil(lua, found)
            })
            .unwrap(),
        )
        .unwrap();

    scene
        .set(
            "CreateCollection",
            lua.create_function(|lua, (name, parent): (String, Option<LuaAnyUserData>)| {
                let world = world_from_lua(lua);
                let parent = parent
                    .and_then(|ud| ud.borrow::<SceneNodeHandle>().ok().map(|h| h.id));
                let id = graph_of(world).add_collection(&name, parent);
                lua.create_userdata(SceneNodeHandle { id })
            })
            .unwrap(),
        )
        .unwrap();

    scene
        .set(
            "InstantiatePrefab",
            lua.create_function(|lua, (path, parent): (String, Option<LuaAnyUserData>)| {
                let world = world_from_lua(lua);
                let parent = parent
                    .and_then(|ud| ud.borrow::<SceneNodeHandle>().ok().map(|h| h.id))
                    .unwrap_or_else(|| world.resource::<SceneGraph>().root());
                let id = serialize::instantiate_prefab(world, &path, parent)
                    .map_err(mluau::Error::RuntimeError)?;
                lua.create_userdata(SceneNodeHandle { id })
            })
            .unwrap(),
        )
        .unwrap();

    scene
        .set(
            "SavePrefab",
            lua.create_function(|lua, (node, path): (LuaAnyUserData, String)| {
                let world = world_from_lua(lua);
                let id = node
                    .borrow::<SceneNodeHandle>()
                    .map(|h| h.id)
                    .map_err(|_| mluau::Error::RuntimeError("expected a scene node".into()))?;
                world.resource_scope::<SceneGraph, _>(|world, graph| {
                    serialize::save_prefab(world, &graph, id, &path)
                        .map_err(mluau::Error::RuntimeError)
                })?;
                Ok(true)
            })
            .unwrap(),
        )
        .unwrap();

    globals.set("Scene", scene).unwrap();
}

pub fn dispatch_events(lua: &Lua, events: &[SceneGraphEvent]) {
    let targets: Vec<(LuaFunction, LuaMultiValue)> = {
        let Some(store) = lua.app_data_ref::<SignalStore>() else {
            return;
        };
        let connections = store.connections.borrow();
        let mut out = Vec::new();
        for event in events {
            for conn in connections.iter() {
                let args = match (event, conn.kind) {
                    (SceneGraphEvent::ChildAdded { parent, child }, SignalKind::ChildAdded)
                        if *parent == conn.node =>
                    {
                        arg_handle(lua, Some(*child))
                    }
                    (SceneGraphEvent::ChildRemoved { parent, child }, SignalKind::ChildRemoved)
                        if *parent == conn.node =>
                    {
                        arg_handle(lua, Some(*child))
                    }
                    (SceneGraphEvent::ParentChanged { node, new, .. }, SignalKind::ParentChanged)
                        if *node == conn.node =>
                    {
                        arg_handle(lua, *new)
                    }
                    (SceneGraphEvent::PathChanged { node, path }, SignalKind::PathChanged)
                        if *node == conn.node =>
                    {
                        path.as_str()
                            .into_lua(lua)
                            .map(|v| LuaMultiValue::from_iter([v]))
                    }
                    _ => continue,
                };
                if let Ok(args) = args {
                    out.push((conn.func.clone(), args));
                }
            }
        }
        out
    };

    for (func, args) in targets {
        if let Err(e) = func.call::<()>(args) {
            warn!("Scene signal handler error: {e}");
        }
    }
}

fn arg_handle(lua: &Lua, id: Option<NodeId>) -> LuaResult<LuaMultiValue> {
    let value = handle_or_nil(lua, id)?;
    Ok(LuaMultiValue::from_iter([value]))
}
