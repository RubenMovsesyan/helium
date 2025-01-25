use std::{
    cell::{Ref, RefMut},
    collections::HashMap,
};

pub use entity::Entity;
use log::*;
use world::World;

mod component;
mod entity;
mod world;

pub struct HeliumECS {
    world: World,
}

impl HeliumECS {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    /// Creates a new entity in the world
    ///
    /// # Returns
    ///
    /// The entity id
    pub fn new_entity(&mut self) -> Entity {
        self.world.new_entity()
    }

    /// Adds the specified component to the specified entity
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the component to be added
    /// * `entity` - The entity id to add the component to
    /// * `componenet` - The component to add
    pub fn add_component<ComponentType: 'static>(
        &mut self,
        entity: Entity,
        component: ComponentType,
    ) {
        self.world.add_component_to_entity(entity, component);
    }

    /// Removes the value from the specified component from the entity
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the component to be removed
    /// * `entity` - The entity id to remove the component from
    pub fn remove_component<ComponentType: 'static>(&mut self, entity: Entity) {
        self.world
            .borrow_component_map_mut::<ComponentType>()
            .unwrap()
            .remove(&entity);
    }

    /// Obtains an immutable reference to the component map specifed
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the component map to obtain
    ///
    /// # Returns
    ///
    /// an immutable reference to the specifed component map
    pub fn query<ComponentType: 'static>(&self) -> Ref<'_, HashMap<Entity, ComponentType>> {
        self.world.borrow_component_map::<ComponentType>().unwrap()
    }

    /// Obtains a mutable reference to the component map specifed
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the component map to obtain
    ///
    /// # Returns
    ///
    /// an mutable reference to the specifed component map
    pub fn query_mut<ComponentType: 'static>(&self) -> RefMut<'_, HashMap<Entity, ComponentType>> {
        self.world
            .borrow_component_map_mut::<ComponentType>()
            .unwrap()
    }

    /// Gives a list of entities that have a component with a specific comparator operator
    ///
    /// # Arguments
    ///
    /// * `ComponentType` - The type for the component map to seach
    /// * `comparator` - A fucntion pointer to compare the component value given
    ///
    /// # Returns
    ///
    /// A list of entities that contain the specified property
    pub fn entities_with<ComponentType: 'static>(
        &self,
        comparator: fn(&ComponentType) -> bool,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();
        for (entity, component) in self
            .world
            .borrow_component_map::<ComponentType>()
            .unwrap()
            .iter()
        {
            if comparator(component) {
                entities.push(*entity);
            }
        }

        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecs_struct() {
        struct Health(i32);
        struct Name(String);
        struct Player;

        let mut ecs = HeliumECS::new();

        let ralph = ecs.new_entity();
        let betty = ecs.new_entity();
        let player = ecs.new_entity();

        ecs.add_component(ralph, Health(100));
        ecs.add_component(ralph, Name(String::from("Ralph")));
        ecs.add_component(betty, Health(100));
        ecs.add_component(betty, Name(String::from("Betty")));
        ecs.add_component(player, Health(100));
        ecs.add_component(player, Player);

        let mut healths = ecs.query_mut::<Health>();
        let names = ecs.query::<Name>();
        for (name, health) in healths.iter_mut().filter_map(|(health_id, health)| {
            if let Some(name) = names.get(health_id) {
                return Some((name, health));
            }
            None
        }) {
            assert_eq!(health.0, 100);
            while health.0 > 0 {
                info!("{} has health {}", name.0, health.0);
                health.0 -= 1;
            }
            assert_eq!(health.0, 0);

            info!("{} has perished", name.0);
        }

        drop(names);

        let players = ecs.query::<Player>();
        for health in healths.iter().filter_map(|(health_id, health)| {
            if let Some(_player) = players.get(health_id) {
                return Some(health);
            }
            None
        }) {
            assert_eq!(health.0, 100);
        }
    }

    #[test]
    fn test_ecs_basics() {
        struct Health(i32);
        struct Name(String);
        struct Player;

        let mut world = World::new();

        let ralph = world.new_entity();
        world.add_component_to_entity(ralph, Health(100));
        world.add_component_to_entity(ralph, Name(String::from("Ralph")));

        let betty = world.new_entity();
        world.add_component_to_entity(betty, Health(100));
        world.add_component_to_entity(betty, Name(String::from("Betty")));

        let player = world.new_entity();
        world.add_component_to_entity(player, Health(100));
        world.add_component_to_entity(player, Player);

        let mut healths = world.borrow_component_map_mut::<Health>().unwrap();
        let names = world.borrow_component_map::<Name>().unwrap();
        let iter = healths.iter_mut().filter_map(|(health_id, health)| {
            if let Some(name) = names.get(health_id) {
                return Some((health, name));
            }
            None
        });

        for (health, name) in iter {
            assert_eq!(health.0, 100);
            while health.0 > 0 {
                info!("{} has health {}", name.0, health.0);
                health.0 -= 1;
            }
            assert_eq!(health.0, 0);

            info!("{} has perished", name.0);
        }

        drop(healths);
        drop(names);

        let healths = world.borrow_component_map::<Health>().unwrap();
        let players = world.borrow_component_map::<Player>().unwrap();
        let iter = healths.iter().filter_map(|(health_id, health)| {
            if let Some(_player) = players.get(health_id) {
                return Some(health);
            }
            None
        });

        for health in iter {
            assert_eq!(health.0, 100);
            info!("Player has health {}", health.0);
        }

        drop(healths);
        drop(players);

        world.remove_entity(ralph);

        assert_eq!(world.get_num_entities(), 2);
    }
}
