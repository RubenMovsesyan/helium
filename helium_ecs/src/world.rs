use crate::{component::ComponentVec, entity::Entity};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

pub struct World {
    entity_count: Entity,
    num_entities: Entity,
    component_maps: Vec<Box<dyn ComponentVec>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_count: 0,
            num_entities: 0,
            component_maps: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let entity_id = self.entity_count;

        self.entity_count += 1;
        self.num_entities += 1;
        entity_id
    }

    pub fn get_num_entities(&self) -> Entity {
        self.num_entities
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        for component_map in self.component_maps.iter_mut() {
            component_map.remove(entity);
        }
        self.num_entities -= 1;
    }

    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: Entity,
        component: ComponentType,
    ) {
        // Find the corresponding component map in our world and insert the component into it
        for component_map in self.component_maps.iter_mut() {
            if let Some(component_map) = component_map
                .as_any_mut()
                .downcast_mut::<RefCell<HashMap<Entity, ComponentType>>>()
            {
                component_map.borrow_mut().insert(entity, component);
                return;
            }
        }

        // If the component doesn't exist then we create it and add it to our component maps
        let mut new_component_map: HashMap<Entity, ComponentType> = HashMap::new();

        // Give the entity the component
        new_component_map.insert(entity, component);
        self.component_maps
            .push(Box::new(RefCell::new(new_component_map)));
    }

    pub fn borrow_component_map<ComponentType: 'static>(
        &self,
    ) -> Option<Ref<HashMap<Entity, ComponentType>>> {
        for component_map in self.component_maps.iter() {
            if let Some(component_map) = component_map
                .as_any()
                .downcast_ref::<RefCell<HashMap<Entity, ComponentType>>>()
            {
                return Some(component_map.borrow());
            }
        }
        None
    }

    pub fn borrow_component_map_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<HashMap<Entity, ComponentType>>> {
        for component_map in self.component_maps.iter() {
            if let Some(component_map) = component_map
                .as_any()
                .downcast_ref::<RefCell<HashMap<Entity, ComponentType>>>()
            {
                return Some(component_map.borrow_mut());
            }
        }
        None
    }
}
