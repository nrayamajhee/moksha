use crate::{log, RcRcell};
use std::collections::HashMap;

pub type Id = usize;

pub trait Component: std::fmt::Debug {}

#[derive(Debug, Clone)]
pub struct EntityRef {
    components: HashMap<String, Id>,
}

#[derive(Debug)]
pub struct Entity {
    pub id: Id,
    pub storage: RcRcell<Storage>,
}

impl Entity {
    pub fn new(storage: RcRcell<Storage>) -> Self {
        let entity_info = EntityRef {
            components: HashMap::new(),
        };
        let id = {
            let mut storage = storage.borrow_mut();
            let id = storage.entities.len();
            storage.entities.push(entity_info);
            id
        };
        Self {
            id,
            storage: storage,
        }
    }
    pub fn with<C: Component + 'static>(self, key: &str, component: C) -> Self {
        {
            let id = {
                let storage = self.storage.borrow();
                let entity = storage.entity(self.id);
                if entity.components.contains_key(key) {
                    Some(*entity.components.get(key).unwrap())
                } else {
                    None
                }
            };
            self.storage.borrow_mut().add_component(key, component, id);
        }
        self
    }
}

#[derive(Debug)]
pub struct Storage {
    entities: Vec<EntityRef>,
    components: HashMap<String, Vec<Box<dyn Component>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }
    pub fn entity(&self, id: Id) -> &EntityRef {
        self.entities
            .get(id)
            .expect("No entity found at the given index.")
    }
    pub fn add_component<C: Component + 'static>(&mut self, key: &str, component: C, id: Option<Id>) {
        if !self.components.contains_key(key.into()) {
            log!("No component found with the key:" key.to_string() " in storage. Creating one.");
            self.components.insert(key.into(), Vec::new());
        }
        if let Some(id) = id {
            self.components.get_mut(key.into()).unwrap().insert(id, Box::new(component));
        } else {
            self.components
                .get_mut(key.into())
                .unwrap()
                .push(Box::new(component));
        }
    }
}
