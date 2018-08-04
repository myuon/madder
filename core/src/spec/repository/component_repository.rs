extern crate serde;
use spec::*;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;

pub trait ComponentRepository: MutRepository<<Self as ComponentRepository>::COMPONENT> {
    type COMPONENT : HaveComponent + Serialize + DeserializeOwned;
}

pub trait HaveComponentRepository {
    type ComponentRepository : ComponentRepository;

    fn component_repo(&self) -> &Self::ComponentRepository;
    fn component_repo_mut(&mut self) -> &mut Self::ComponentRepository;
}
