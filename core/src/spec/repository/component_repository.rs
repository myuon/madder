extern crate serde;
use spec::*;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;

pub trait ComponentRepository: Repository<<Self as ComponentRepository>::COMPONENT, usize> {
    type COMPONENT : HaveComponent + Serialize + DeserializeOwned;
}

pub trait HaveComponentRepository {
    type REPO : ComponentRepository;

    fn component_repo(&self) -> &Self::REPO;
    fn component_repo_mut(&mut self) -> &mut Self::REPO;
}
