use spec::*;
use feat::repository::hashmap_impl::*;

pub struct EffectRepositoryImpl {
    repository: RepositoryHashMapImpl<Effect>,
}

impl EffectRepositoryImpl {
    pub fn new() -> EffectRepositoryImpl {
        EffectRepositoryImpl {
            repository: RepositoryHashMapImpl::new(),
        }
    }
}

impl Repository<Effect> for EffectRepositoryImpl {
    fn create(&mut self, entity: Effect) -> String {
        self.repository.create(entity)
    }

    fn get(&self, key: &str) -> &Effect {
        self.repository.get(key)
    }

    fn list(&self) -> Vec<Entity<&Effect, &str>> {
        self.repository.list()
    }

    fn update(&mut self, key: String, entity: Effect) {
        self.repository.update(key, entity)
    }

    fn delete(&mut self, key: &str) {
        self.repository.delete(key)
    }
}

impl MutRepository<Effect> for EffectRepositoryImpl {
    fn get_mut(&mut self, key: &str) -> &mut Effect {
        self.repository.get_mut(key)
    }
}

impl EffectRepository for EffectRepositoryImpl {
    fn create_intermed(&mut self, effect_id: &str, point: EffectPoint) {
        let effect = self.repository.get_mut(effect_id);
        effect.intervals.push(point);
    }

    fn value(&self, effect_id: &str, time: f32) -> f32 {
        self.repository.get(effect_id).value(time)
    }
}

