#![allow(dead_code)]

use bevy::{
    ecs::component::{ComponentCloneBehavior, ComponentDescriptor, ComponentId, StorageType},
    prelude::*,
    ptr::OwningPtr,
};
use lasso::{Rodeo, Spur};
use mluau::prelude::*;
use std::{alloc::Layout, collections::HashMap, ptr::NonNull};

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send(ScriptingRuntime { lua: Lua::new() })
            .init_resource::<EngineStringPool>()
            .init_resource::<SchemaRegistry>();
    }
}

pub struct ScriptingRuntime {
    pub lua: Lua,
}

#[derive(Resource, Default)]
pub struct EngineStringPool {
    pub rodeo: Rodeo,
    pub bridge: HashMap<Spur, LuaRegistryKey>,
}

impl EngineStringPool {
    pub fn get_lua_str(&self, lua: &Lua, spur: Spur) -> LuaString {
        let key = self.bridge.get(&spur).expect("unregistered spur");
        lua.registry_value(key).expect("failed to retrieve string")
    }

    pub fn register_lua_string(&mut self, lua: &Lua, s: &LuaString) -> Option<Spur> {
        let borrowed = s.to_str().ok()?;
        let spur = self.rodeo.get_or_intern(&*borrowed);
        self.bridge
            .entry(spur)
            .or_insert_with(|| lua.create_registry_value(s.clone()).unwrap());
        Some(spur)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuauFieldType {
    Bool,
    Integer,
    Number,
    Vector3,
    String,
    Buffer(usize),
}

impl LuauFieldType {
    pub fn layout(self) -> Layout {
        match self {
            Self::Bool => Layout::new::<bool>(),
            Self::Integer => Layout::new::<i64>(),
            Self::Number => Layout::new::<f64>(),
            Self::Vector3 => Layout::new::<[f32; 3]>(),
            Self::String => Layout::new::<Spur>(),
            Self::Buffer(n) => Layout::array::<u8>(n).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct DynamicComponentSchema {
    pub name: String,
    pub fields: HashMap<Spur, (usize, LuauFieldType)>,
    pub layout: Layout,
}

#[derive(Resource, Default)]
pub struct SchemaRegistry {
    pub name_to_id: HashMap<String, ComponentId>,
    pub id_to_schema: HashMap<ComponentId, DynamicComponentSchema>,
}

impl SchemaRegistry {
    pub fn register(
        &mut self,
        world: &mut World,
        name: String,
        fields: &[(Spur, LuauFieldType)],
    ) -> ComponentId {
        let mut struct_layout = Layout::from_size_align(0, 1).unwrap();
        let mut field_offsets = HashMap::new();
        let mut sorted = fields.to_vec();
        sorted.sort_by_key(|(spur, _)| *spur);

        for (spur, field_type) in &sorted {
            let (new_layout, offset) = struct_layout.extend(field_type.layout()).unwrap();
            struct_layout = new_layout;
            field_offsets.insert(*spur, (offset, *field_type));
        }
        let layout = struct_layout.pad_to_align();

        let schema = DynamicComponentSchema {
            name: name.clone(),
            fields: field_offsets,
            layout,
        };

        let descriptor = unsafe {
            ComponentDescriptor::new_with_layout(
                name.clone(),
                StorageType::Table,
                layout,
                None,
                true,
                ComponentCloneBehavior::Ignore,
                None,
            )
        };

        let id = world.register_component_with_descriptor(descriptor);
        self.name_to_id.insert(name, id);
        self.id_to_schema.insert(id, schema);
        id
    }
}

pub struct DynamicComponentBridge;

impl DynamicComponentBridge {
    pub unsafe fn insert_from_table(
        world: &mut World,
        entity: Entity,
        component_id: ComponentId,
        registry: &SchemaRegistry,
        pool: &mut EngineStringPool,
        table: &LuaTable,
        lua: &Lua,
    ) -> LuaResult<()> {
        let schema = registry
            .id_to_schema
            .get(&component_id)
            .expect("Schema not registered");

        let u64_count = schema.layout.size().div_ceil(8);
        let mut scratch = vec![0u64; u64_count];
        let scratch_ptr = scratch.as_mut_ptr() as *mut u8;

        for (&spur, &(offset, field_type)) in &schema.fields {
            let lua_key = pool.get_lua_str(lua, spur);
            let field_ptr = scratch_ptr.add(offset);

            match (table.raw_get::<LuaValue>(lua_key)?, field_type) {
                (LuaValue::Boolean(b), LuauFieldType::Bool) => {
                    std::ptr::write(field_ptr as *mut bool, b)
                }
                (LuaValue::Integer(i), LuauFieldType::Integer) => {
                    std::ptr::write(field_ptr as *mut i64, i)
                }
                (LuaValue::Number(n), LuauFieldType::Number) => {
                    std::ptr::write(field_ptr as *mut f64, n)
                }
                (LuaValue::Vector(v), LuauFieldType::Vector3) => {
                    std::ptr::write(field_ptr as *mut [f32; 3], [v.x(), v.y(), v.z()])
                }
                (LuaValue::String(s), LuauFieldType::String) => {
                    if let Some(str_spur) = pool.register_lua_string(lua, &s) {
                        std::ptr::write(field_ptr as *mut Spur, str_spur);
                    }
                }
                (LuaValue::Buffer(b), LuauFieldType::Buffer(len)) => {
                    std::ptr::copy_nonoverlapping(
                        b.to_pointer() as *mut u8,
                        field_ptr,
                        b.len().min(len),
                    );
                }
                _ => {}
            }
        }

        let owning_ptr = OwningPtr::new(NonNull::new(scratch_ptr).unwrap());
        world
            .entity_mut(entity)
            .insert_by_id(component_id, owning_ptr);

        Ok(())
    }

    pub unsafe fn extract_to_table(
        world: &World,
        entity: Entity,
        component_id: ComponentId,
        registry: &SchemaRegistry,
        pool: &EngineStringPool,
        lua: &Lua,
    ) -> LuaResult<Option<LuaTable>> {
        let Some(schema) = registry.id_to_schema.get(&component_id) else {
            return Ok(None);
        };
        let Ok(ptr) = world.entity(entity).get_by_id(component_id) else {
            return Ok(None);
        };

        let raw_ptr = ptr.as_ptr();
        let table = lua.create_table()?;

        for (&spur, &(offset, field_type)) in &schema.fields {
            let lua_key = pool.get_lua_str(lua, spur);
            let field_ptr = raw_ptr.add(offset);

            match field_type {
                LuauFieldType::Bool => {
                    table.raw_set(lua_key, std::ptr::read(field_ptr as *const bool))?
                }
                LuauFieldType::Integer => {
                    table.raw_set(lua_key, std::ptr::read(field_ptr as *const i64))?
                }
                LuauFieldType::Number => {
                    table.raw_set(lua_key, std::ptr::read(field_ptr as *const f64))?
                }
                LuauFieldType::Vector3 => {
                    let v = std::ptr::read(field_ptr as *const [f32; 3]);
                    table.raw_set(lua_key, mluau::Vector::new(v[0], v[1], v[2]))?;
                }
                LuauFieldType::String => {
                    let str_spur = std::ptr::read(field_ptr as *const Spur);
                    table.raw_set(lua_key, pool.get_lua_str(lua, str_spur))?;
                }
                LuauFieldType::Buffer(len) => {
                    let slice = std::slice::from_raw_parts(field_ptr, len);
                    table.raw_set(lua_key, lua.create_buffer(slice)?)?;
                }
            }
        }

        Ok(Some(table))
    }
}
