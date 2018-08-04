use spec::*;
use feat::entity::*;
use feat::repository::hashmap_impl::*;

pub struct ComponentRepositoryImpl {
    repository: RepositoryHashMapImpl<ComponentExt>,
}

impl ComponentRepositoryImpl {
    pub fn new() -> ComponentRepositoryImpl {
        ComponentRepositoryImpl {
            repository: RepositoryHashMapImpl::new(),
        }
    }
}

// I really do not like to write the following impl-s by hand,
// but defining trait like RepositoryHashMapImplWrapper in hashmap_impl does not work
// since it conflicts the other impl for Repository...
impl Repository<ComponentExt> for ComponentRepositoryImpl {
    fn create(&mut self, entity: ComponentExt) -> String {
        self.repository.create(entity)
    }

    fn get(&self, key: &str) -> &ComponentExt {
        self.repository.get(key)
    }

    fn list(&self) -> Vec<Entity<&ComponentExt, &str>> {
        self.repository.list()
    }

    fn update(&mut self, key: String, entity: ComponentExt) {
        self.repository.update(key, entity)
    }

    fn delete(&mut self, key: &str) {
        self.repository.delete(key)
    }
}

impl MutRepository<ComponentExt> for ComponentRepositoryImpl {
    fn get_mut(&mut self, key: &str) -> &mut ComponentExt {
        self.repository.get_mut(key)
    }
}

impl ComponentRepository for ComponentRepositoryImpl {
    type COMPONENT = ComponentExt;
}

