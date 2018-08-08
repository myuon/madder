extern crate serde_json;
extern crate serde;
use spec::*;
use serde::ser::Serialize;

pub trait ComponentRepository
    : MutRepository<<Self as ComponentRepository>::COMPONENT>
    + RepositoryLoader<<Self as ComponentRepository>::COMPONENT>
{
    type COMPONENT : HaveComponent + Serialize + From<serde_json::Value>;
}

pub trait HaveComponentRepository {
    type ComponentRepository : ComponentRepository;

    fn component_repo(&self) -> &Self::ComponentRepository;
    fn component_repo_mut(&mut self) -> &mut Self::ComponentRepository;

    fn new_from_json(json: serde_json::Value) ->
        <<Self as HaveComponentRepository>::ComponentRepository as ComponentRepository>::COMPONENT {
        <<Self as HaveComponentRepository>::ComponentRepository as ComponentRepository>::COMPONENT::from(json)
    }
}
