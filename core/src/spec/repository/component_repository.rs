extern crate serde_json;
extern crate serde;
use spec::*;
use serde::ser::Serialize;

pub trait HaveComponentRepository {
    type COMPONENT : HaveComponent + Serialize + From<serde_json::Value>;
    type ComponentRepository
        : MutRepository<Self::COMPONENT>
        + RepositoryLoader<Self::COMPONENT>;

    fn component_repo(&self) -> &Self::ComponentRepository;
    fn component_repo_mut(&mut self) -> &mut Self::ComponentRepository;

    fn new_from_json(json: serde_json::Value) -> Self::COMPONENT {
        Self::COMPONENT::from(json)
    }
}
