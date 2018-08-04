extern crate serde;
use spec::*;

pub trait EffectRepository : MutRepository<Effect> + RepositoryLoader<Effect> {
    fn create_intermed(&mut self, &str, EffectPoint);
    fn value(&self, &str, f32) -> f32;
}

pub trait HaveEffectRepository {
    type EffectRepository : EffectRepository;

    fn effect_repo(&self) -> &Self::EffectRepository;
    fn effect_repo_mut(&mut self) -> &mut Self::EffectRepository;
}
