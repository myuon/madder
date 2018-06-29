use feat::*;

pub struct ProjectImpl {
    component_repo_impl: Vec<ComponentExt>,
}

impl HaveRepositoryImpl<ComponentExt> for ProjectImpl {
    fn elements(&self) -> &Vec<ComponentExt> {
        &self.component_repo_impl
    }

    fn elements_mut(&mut self) -> &mut Vec<ComponentExt> {
        &mut self.component_repo_impl
    }
}

impl ProjectImpl {
    pub fn new() -> ProjectImpl {
        ProjectImpl {
            component_repo_impl: vec![],
        }
    }
}

