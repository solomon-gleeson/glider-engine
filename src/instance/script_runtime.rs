use bevy::prelude::*;
use mluau::UserData;
use mluau::prelude::*;

use super::{ConsoleLevel, Instance, ScriptConsole, ScriptSource};
use crate::instance::registry::ClassRegistry;

pub struct WorldAccessor {
    pub world_ptr: *mut World,
}

unsafe impl Send for WorldAccessor {}
unsafe impl Sync for WorldAccessor {}

#[derive(Clone, Copy)]
pub struct InstanceHandle {
    entity: Entity,
}

impl UserData for InstanceHandle {
    fn add_fields<F: mluau::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |lua, handle| {
            let world = world_from_lua(lua);
            Ok(get_name(world, handle.entity))
        });
        fields.add_field_method_set("Name", |lua, handle, name: String| {
            let world = world_from_lua(lua);
            set_name(world, handle.entity, &name);
            Ok(())
        });
        fields.add_field_method_get("Parent", |lua, handle| {
            let world = world_from_lua(lua);
            let parent = get_parent(world, handle.entity);
            match parent {
                Some(p) => Ok(Some(lua.create_userdata(InstanceHandle { entity: p })?)),
                None => Ok(None),
            }
        });
        fields.add_field_method_set("Parent", |lua, handle, value: Option<LuaAnyUserData>| {
            let world = world_from_lua(lua);
            let parent = value.and_then(|ud| ud.borrow::<InstanceHandle>().ok().map(|h| h.entity));
            reparent(world, handle.entity, parent);
            Ok(())
        });
    }

    fn add_methods<M: mluau::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("GetChildren", |lua, handle, ()| {
            let world = world_from_lua(lua);
            let children = get_children(world, handle.entity);
            let table = lua.create_table()?;
            for (i, child) in children.iter().enumerate() {
                let ud = lua.create_userdata(InstanceHandle { entity: *child })?;
                table.set(i + 1, ud)?;
            }
            Ok(table)
        });
        methods.add_method("FindFirstChild", |lua, handle, name: String| {
            let world = world_from_lua(lua);
            let child = find_first_child(world, handle.entity, &name);
            match child {
                Some(c) => Ok(Some(lua.create_userdata(InstanceHandle { entity: c })?)),
                None => Ok(None),
            }
        });
        methods.add_method("Destroy", |lua, handle, ()| {
            let world = world_from_lua(lua);
            destroy_recursive(world, handle.entity);
            Ok(())
        });
    }
}

#[allow(clippy::mut_from_ref)]
fn world_from_lua(lua: &Lua) -> &mut World {
    let accessor = lua
        .app_data_ref::<WorldAccessor>()
        .expect("world accessed outside of a script call (no live WorldAccessor)");
    unsafe { &mut *accessor.world_ptr }
}

fn get_name(world: &World, entity: Entity) -> String {
    world
        .entity(entity)
        .get::<Instance>()
        .map(|i| i.name.clone())
        .unwrap_or_default()
}

fn set_name(world: &mut World, entity: Entity, name: &str) {
    if let Some(mut inst) = world.entity_mut(entity).get_mut::<Instance>() {
        inst.name = name.to_string();
    }
}

fn get_parent(world: &World, entity: Entity) -> Option<Entity> {
    world
        .entity(entity)
        .get::<Instance>()
        .and_then(|i| i.parent)
}

fn reparent(world: &mut World, child: Entity, new_parent: Option<Entity>) {
    let old = get_parent(world, child);
    if let Some(old) = old
        && let Some(mut p) = world.entity_mut(old).get_mut::<Instance>()
    {
        p.children.retain(|c| *c != child);
    }
    if let Some(new) = new_parent
        && let Some(mut p) = world.entity_mut(new).get_mut::<Instance>()
    {
        p.children.push(child);
    }
    if let Some(mut inst) = world.entity_mut(child).get_mut::<Instance>() {
        inst.parent = new_parent;
    }
}

fn get_children(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .entity(entity)
        .get::<Instance>()
        .map(|i| i.children.clone())
        .unwrap_or_default()
}

fn find_first_child(world: &World, entity: Entity, name: &str) -> Option<Entity> {
    get_children(world, entity)
        .iter()
        .find(|&&c| get_name(world, c) == name)
        .copied()
}

fn destroy_recursive(world: &mut World, entity: Entity) {
    for child in get_children(world, entity) {
        destroy_recursive(world, child);
    }
    if let Some(parent) = get_parent(world, entity)
        && let Some(mut p) = world.entity_mut(parent).get_mut::<Instance>()
    {
        p.children.retain(|c| *c != entity);
    }
    world.entity_mut(entity).despawn();
}

fn spawn_instance(world: &mut World, class_name: &str, parent: Option<Entity>) -> Option<Entity> {
    world.get_resource::<ClassRegistry>()?;

    world.resource_scope::<ClassRegistry, _>(|world, registry| {
        registry.spawn(world, class_name, parent)
    })
}

pub struct ScriptEntry {
    pub entity: Entity,
    pub init_fn: Option<LuaFunction>,
    pub update_fn: Option<LuaFunction>,
}

pub struct ScriptRuntime {
    pub lua: Lua,
    pub scripts: Vec<ScriptEntry>,
}

impl ScriptRuntime {
    pub fn new(world: &mut World) -> Self {
        let lua = Lua::new();

        install_api(&lua);
        let _ = lua.sandbox(true);

        let pending = collect_pending_scripts(world);
        let scripts = with_world(&lua, world, |lua| run_pending_scripts(lua, pending));

        ScriptRuntime { lua, scripts }
    }

    pub fn init_scripts(&self, world: &mut World) {
        with_world(&self.lua, world, |lua| {
            for entry in &self.scripts {
                if let Some(ref f) = entry.init_fn
                    && let Err(e) = f.call::<()>(())
                {
                    warn!("Script init error: {e}");
                    console_push(lua, ConsoleLevel::Error, format!("init error: {e}"));
                }
            }
        });
    }

    pub fn update_scripts(&self, world: &mut World, dt: f32) {
        with_world(&self.lua, world, |lua| {
            for entry in &self.scripts {
                if let Some(ref f) = entry.update_fn
                    && let Err(e) = f.call::<()>(dt)
                {
                    warn!("Script update error: {e}");
                    console_push(lua, ConsoleLevel::Error, format!("update error: {e}"));
                }
            }
        });
    }
}

fn with_world<R>(lua: &Lua, world: &mut World, f: impl FnOnce(&Lua) -> R) -> R {
    lua.set_app_data(WorldAccessor {
        world_ptr: world as *mut World,
    });
    let result = f(lua);
    lua.remove_app_data::<WorldAccessor>();
    result
}

fn install_api(lua: &Lua) {
    let globals = lua.globals();

    let instance = lua.create_table().unwrap();
    let new_fn = lua
        .create_function(
            |lua, (class_name, parent): (String, Option<LuaAnyUserData>)| {
                let world = world_from_lua(lua);
                let parent_entity =
                    parent.and_then(|ud| ud.borrow::<InstanceHandle>().ok().map(|h| h.entity));
                let entity =
                    spawn_instance(world, &class_name, parent_entity).ok_or_else(|| {
                        mluau::Error::RuntimeError(format!("unknown class '{class_name}'"))
                    })?;
                let handle = lua.create_userdata(InstanceHandle { entity })?;
                Ok(handle)
            },
        )
        .unwrap();
    instance.set("new", new_fn).unwrap();
    globals.set("Instance", instance).unwrap();

    globals
        .set(
            "print",
            lua.create_function(|lua, args: LuaVariadic<LuaValue>| {
                let text = lua_args_to_string(lua, &args);
                info!("[Luau] {text}");
                console_push(lua, ConsoleLevel::Info, text);
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();
}

fn lua_args_to_string(lua: &Lua, args: &LuaVariadic<LuaValue>) -> String {
    args.iter()
        .map(|v| match lua.coerce_string(v.clone()) {
            Ok(Some(s)) => s.to_string_lossy(),
            _ => v.type_name().to_string(),
        })
        .collect::<Vec<_>>()
        .join("\t")
}

fn console_push(lua: &Lua, level: ConsoleLevel, text: String) {
    let world = world_from_lua(lua);
    if let Some(mut console) = world.get_resource_mut::<ScriptConsole>() {
        console.push(level, text);
    }
}

struct PendingScript {
    entity: Entity,
    name: String,
    source: String,
}

fn collect_pending_scripts(world: &World) -> Vec<PendingScript> {
    let mut pending = Vec::new();

    for entity_ref in world.iter_entities() {
        let entity = entity_ref.id();

        let Some(instance) = entity_ref.get::<Instance>() else {
            continue;
        };
        let Some(source) = entity_ref.get::<ScriptSource>() else {
            continue;
        };
        if !source.enabled {
            continue;
        }

        pending.push(PendingScript {
            entity,
            name: instance.name.clone(),
            source: source.source.clone(),
        });
    }

    pending
}

fn run_pending_scripts(lua: &Lua, pending: Vec<PendingScript>) -> Vec<ScriptEntry> {
    let mut entries = Vec::new();

    for script in pending {
        let env = match make_script_env(lua, script.entity) {
            Ok(env) => env,
            Err(e) => {
                warn!("Script '{}' setup error: {e}", script.name);
                console_push(
                    lua,
                    ConsoleLevel::Error,
                    format!("{}: setup error: {e}", script.name),
                );
                continue;
            }
        };

        let func = match lua
            .load(script.source.as_bytes())
            .set_environment(env.clone())
            .into_function()
        {
            Ok(f) => f,
            Err(e) => {
                warn!("Script '{}' compile error: {e}", script.name);
                console_push(
                    lua,
                    ConsoleLevel::Error,
                    format!("{}: compile error: {e}", script.name),
                );
                continue;
            }
        };

        if let Err(e) = func.call::<()>(()) {
            warn!("Script '{}' runtime error: {e}", script.name);
            console_push(
                lua,
                ConsoleLevel::Error,
                format!("{}: runtime error: {e}", script.name),
            );
            continue;
        }

        let init_fn = env.raw_get::<Option<LuaFunction>>("init").ok().flatten();
        let update_fn = env.raw_get::<Option<LuaFunction>>("update").ok().flatten();

        entries.push(ScriptEntry {
            entity: script.entity,
            init_fn,
            update_fn,
        });
    }

    entries
}

fn make_script_env(lua: &Lua, entity: Entity) -> LuaResult<LuaTable> {
    let env = lua.create_table()?;
    let mt = lua.create_table()?;
    mt.set("__index", lua.globals())?;
    env.set_metatable(Some(mt))?;
    env.set("script", lua.create_userdata(InstanceHandle { entity })?)?;
    Ok(env)
}
