extern crate uuid;
use spec::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct RepositoryHashMapImpl<ENTITY> {
    entities: HashMap<String, ENTITY>,
}

impl<ENTITY> RepositoryHashMapImpl<ENTITY> {
    pub fn new() -> RepositoryHashMapImpl<ENTITY> {
        RepositoryHashMapImpl {
            entities: HashMap::new(),
        }
    }
}

impl<ENTITY> Repository<ENTITY> for RepositoryHashMapImpl<ENTITY> {
    fn create(&mut self, entity: ENTITY) -> String {
        let key = Uuid::new_v4().to_string();
        self.entities.insert(key.clone(), entity);
        key
    }

    fn get(&self, index: &str) -> &ENTITY {
        self.entities.get(index).unwrap()
    }

    fn list(&self) -> Vec<Entity<&ENTITY, &str>> {
        self.entities.iter().map(|(k,v)| Entity::new(k.as_str(),v)).collect()
    }

    fn update(&mut self, index: String, entity: ENTITY) {
        self.entities.insert(index, entity);
    }

    fn delete(&mut self, index: &str) {
        self.entities.remove(index);
    }
}

impl<ENTITY> MutRepository<ENTITY> for RepositoryHashMapImpl<ENTITY> {
    fn get_mut(&mut self, index: &str) -> &mut ENTITY {
        self.entities.get_mut(index).unwrap()
    }
}

