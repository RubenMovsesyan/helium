use crate::entity::Entity;
use std::{any::Any, cell::RefCell, collections::HashMap};

pub trait ComponentVec {
    #[allow(unused)]
    fn remove(&mut self, entity: Entity);

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> ComponentVec for RefCell<HashMap<Entity, T>> {
    fn remove(&mut self, entity: Entity) {
        self.borrow_mut().remove(&entity);
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}
