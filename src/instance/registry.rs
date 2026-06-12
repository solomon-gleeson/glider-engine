use std::collections::HashMap;

use bevy::prelude::*;

use super::{Instance, NextInstanceId};

pub type ClassFactory = Box<dyn Fn(&mut World, Entity) + Send + Sync>;

#[derive(Resource, Default)]
pub struct ClassRegistry {
    factories: HashMap<String, ClassFactory>,
}

impl ClassRegistry {
    pub fn register<F>(&mut self, class_name: &str, factory: F)
    where
        F: Fn(&mut World, Entity) + Send + Sync + 'static,
    {
        self.factories
            .insert(class_name.to_string(), Box::new(factory));
    }

    pub fn spawn(
        &self,
        world: &mut World,
        class_name: &str,
        parent: Option<Entity>,
    ) -> Option<Entity> {
        let factory = self.factories.get(class_name)?;
        let id = world.resource_mut::<NextInstanceId>().allocate();

        let entity = world.spawn(Instance {
            id,
            name: class_name.to_string(),
            class_name: class_name.to_string(),
            parent: None,
            children: Vec::new(),
        });

        let entity_id = entity.id();
        factory(world, entity_id);

        if let Some(parent_entity) = parent {
            set_parent_internal(world, entity_id, Some(parent_entity));
        }

        Some(entity_id)
    }

    pub fn contains(&self, class_name: &str) -> bool {
        self.factories.contains_key(class_name)
    }
}

pub fn set_parent_internal(world: &mut World, child: Entity, new_parent: Option<Entity>) {
    let old_parent = world
        .entity(child)
        .get::<Instance>()
        .and_then(|inst| inst.parent);

    if let Some(old) = old_parent
        && let Some(mut old_inst) = world.entity_mut(old).get_mut::<Instance>()
    {
        old_inst.children.retain(|c| *c != child);
    }

    if let Some(new) = new_parent
        && let Some(mut new_inst) = world.entity_mut(new).get_mut::<Instance>()
    {
        new_inst.children.push(child);
    }

    if let Some(mut child_inst) = world.entity_mut(child).get_mut::<Instance>() {
        child_inst.parent = new_parent;
    }
}

#[derive(Component, Clone)]
pub struct ScriptSource {
    pub source: String,
    pub enabled: bool,

    pub path: Option<String>,
}

impl Default for ScriptSource {
    fn default() -> Self {
        Self {
            source: String::new(),
            enabled: true,
            path: None,
        }
    }
}

pub fn register_core_classes(registry: &mut ClassRegistry) {
    registry.register("Model", |_world, _entity| {});

    registry.register("Script", |world, entity| {
        world.entity_mut(entity).insert(ScriptSource::default());
    });

    registry.register("Part", |world, entity| {
        world.entity_mut(entity).insert((
            Sprite {
                custom_size: Some(Vec2::splat(50.0)),
                color: Color::srgb(0.2, 0.6, 1.0),
                ..default()
            },
            Transform::from_xyz(200.0, 0.0, 0.0),
        ));
    });
}
