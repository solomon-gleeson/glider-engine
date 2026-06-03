// src/core/scripting.rs

use bevy::ecs::component::{ComponentDescriptor, ComponentId, StorageType};
use bevy::prelude::*;
use bevy::ptr::OwningPtr;
use lasso::{Rodeo, Spur};
use mlua::prelude::*;
use mlua::{RegistryKey, Value, Vector};
use smallvec::SmallVec;
use std::hash::{Hash, Hasher, DefaultHasher};
use std::ptr::NonNull;
use std::alloc::Layout;
use std::collections::{HashMap, hash_map::Entry};

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let string_pool = EngineStringPool {
            rodeo: Rodeo::new(),
            bridge: HashMap::new(),
        };

        let lua = Lua::new();

        // Bind custom engine logging to Luau environment
        let print_from_rust = lua.create_function(|_, msg: String| {
            info!("[Luau]: {}", msg);
            Ok(())
        }).unwrap();
        lua.globals().set("info", print_from_rust).unwrap();

        // Lua is !Send, so we must use insert_non_send_resource
        app.insert_non_send_resource(ScriptingRuntime { lua })
            .insert_resource(string_pool)
            .insert_resource(SchemaRegistry { schemas: HashMap::new() });
    }
}

pub struct ScriptingRuntime {
    pub lua: Lua,
}

#[derive(Resource)]
pub struct EngineStringPool {
    pub rodeo: Rodeo,
    // We store the strings as RegistryKeys to keep them interned in Lua memory across frames
    pub bridge: HashMap<Spur, RegistryKey>,
}

impl EngineStringPool {
    pub fn register_string(&mut self, lua: &Lua, text: &str) -> LuaResult<Spur> {
        let spur = self.rodeo.get_or_intern(text);
        match self.bridge.entry(spur) {
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                let lua_string = lua.create_string(text)?;
                let key = lua.create_registry_value(lua_string)?;
                entry.insert(key);
            }
        }
        Ok(spur)
    }

    pub fn register_lua_string(&mut self, lua: &Lua, s: &LuaString) -> Option<Spur> {
        let borrowed = s.to_str().ok()?;
        let spur = self.rodeo.get_or_intern(&*borrowed);
        if !self.bridge.contains_key(&spur) {
            if let Ok(key) = lua.create_registry_value(s.clone()) {
                self.bridge.insert(spur, key);
            }
        }
        Some(spur)
    }

    #[inline]
    pub fn get_lua_str<'lua>(&self, lua: &'lua Lua, spur: Spur) -> LuaString<'lua> {
        let key = self.bridge.get(&spur).expect("unregistered spur");
        lua.registry_value(key).expect("failed to retrieve registry string")
    }
}

pub enum LuauFrameIr {
    Bool(bool),
    Integer(i64),
    Number(f64),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    String(Spur),
    Buffer(Vec<u8>),
}

#[derive(Clone, Copy, Debug)]
pub enum LuauFieldType {
    Bool,
    Integer,
    Number,
    Vector3,
    Vector4,
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
            Self::Vector4 => Layout::new::<[f32; 4]>(),
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
    pub signature: u64,
}

impl DynamicComponentSchema {
    pub fn build(name: String, fields: &[(Spur, LuauFieldType)]) -> Self {
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
        let signature = Self::compute_signature(fields);

        Self { name, fields: field_offsets, layout, signature }
    }

    pub fn compute_signature(fields: &[(Spur, LuauFieldType)]) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut sorted = fields.to_vec();
        sorted.sort_by_key(|(spur, _)| *spur);
        for (spur, ty) in sorted {
            spur.hash(&mut hasher);
            std::mem::discriminant(&ty).hash(&mut hasher);
            if let LuauFieldType::Buffer(n) = ty {
                n.hash(&mut hasher)
            }
        }
        hasher.finish()
    }
}

#[derive(Resource)]
pub struct SchemaRegistry {
    pub schemas: HashMap<String, DynamicComponentSchema>,
}

impl SchemaRegistry {
    pub fn register(&mut self, world: &mut World, schema: DynamicComponentSchema) -> ComponentId {
        let descriptor = unsafe {
            ComponentDescriptor::new_with_layout(
                schema.name.clone(),
                StorageType::Table,
                schema.layout,
                None, // Dynamic components don't have a drop fn in this simplified model
            )
        };
        self.schemas.insert(schema.name.clone(), schema);
        world.init_component_with_descriptor(descriptor)
    }

    pub fn get(&self, schema_name: &str) -> Option<&DynamicComponentSchema> {
        self.schemas.get(schema_name)
    }
}

pub unsafe fn insert_luau_data(
    world: &mut World,
    entity: Entity,
    component_id: ComponentId,
    registry: &SchemaRegistry,
    schema_name: &str,
    data: &LuauFrameIrLayout,
) {
    let schema = registry.get(schema_name).expect("Schema not registered");

    let scratch_ptr = std::alloc::alloc_zeroed(schema.layout);
    if scratch_ptr.is_null() {
        std::alloc::handle_alloc_error(schema.layout);
    }

    for (spur, val) in &data.fields {
        if let Some(&(offset, field_type)) = schema.fields.get(spur) {
            let field_ptr = scratch_ptr.add(offset);
            match val {
                LuauFrameIr::Bool(b) => if matches!(field_type, LuauFieldType::Bool) { std::ptr::write(field_ptr as *mut bool, *b); },
                LuauFrameIr::Integer(i) => if matches!(field_type, LuauFieldType::Integer) { std::ptr::write(field_ptr as *mut i64, *i); },
                LuauFrameIr::Number(n) => if matches!(field_type, LuauFieldType::Number) { std::ptr::write(field_ptr as *mut f64, *n); },
                LuauFrameIr::Vector3(v) => if matches!(field_type, LuauFieldType::Vector3) { std::ptr::write(field_ptr as *mut [f32; 3], *v); },
                LuauFrameIr::Vector4(v) => if matches!(field_type, LuauFieldType::Vector4) { std::ptr::write(field_ptr as *mut [f32; 4], *v); },
                LuauFrameIr::String(s) => if matches!(field_type, LuauFieldType::String) { std::ptr::write(field_ptr as *mut Spur, *s); },
                LuauFrameIr::Buffer(buf) => if let LuauFieldType::Buffer(len) = field_type {
                    let copy_len = buf.len().min(len);
                    std::ptr::copy_nonoverlapping(buf.as_ptr(), field_ptr, copy_len);
                }
            }
        }
    }

    let non_null = NonNull::new(scratch_ptr).unwrap();
    let owning_ptr = OwningPtr::new(non_null);

    world.entity_mut(entity).insert_by_id(component_id, owning_ptr);

    // FIX THE LEAK: Free transient allocation now that Bevy copied it to table column
    std::alloc::dealloc(scratch_ptr, schema.layout);
}

pub struct LuauFrameIrLayout {
    pub fields: SmallVec<[(Spur, LuauFrameIr); 8]>,
}

impl LuauFrameIrLayout {
    pub fn write_to_table(&self, lua: &Lua, table: &LuaTable, pool: &EngineStringPool) -> LuaResult<()> {
        for (key_spur, val) in &self.fields {
            let lua_key = pool.get_lua_str(lua, *key_spur);
            match val {
                LuauFrameIr::Bool(b) => table.raw_set(lua_key, *b)?,
                LuauFrameIr::Integer(i) => table.raw_set(lua_key, *i)?,
                LuauFrameIr::Number(n) => table.raw_set(lua_key, *n)?,
                LuauFrameIr::String(s) => table.raw_set(lua_key, pool.get_lua_str(lua, *s))?,
                LuauFrameIr::Vector3([x, y, z]) => table.raw_set(lua_key, Vector::new(*x, *y, *z))?,
                _ => {}
            }
        }
        Ok(())
    }

    pub fn read_from_table(lua: &Lua, table: &LuaTable, schema: &[Spur], pool: &mut EngineStringPool) -> LuaResult<Self> {
        let mut fields = SmallVec::new();
        for &key_spur in schema {
            let key = pool.get_lua_str(lua, key_spur);
            match table.raw_get::<LuaString, Value>(key)? {
                Value::Boolean(b) => fields.push((key_spur, LuauFrameIr::Bool(b))),
                Value::Integer(i) => fields.push((key_spur, LuauFrameIr::Integer(i as i64))),
                Value::Number(n) => fields.push((key_spur, LuauFrameIr::Number(n))),
                Value::String(s) => {
                    // FIX: Dynamically register unexpected strings created on runtime heap
                    if let Some(spur) = pool.register_lua_string(lua, &s) {
                        fields.push((key_spur, LuauFrameIr::String(spur)));
                    }
                }
                Value::Vector(vector) => {
                    fields.push((key_spur, LuauFrameIr::Vector3([vector.x(), vector.y(), vector.z()])));
                }
                // mlua 0.9 might not have a Buffer variant yet depending on features
                _ => {}
            }
        }
        Ok(Self { fields })
    }
}
