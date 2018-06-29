use spec::*;

pub trait Project {
    type COMPONENT : Component;
    type COMPONENT_REPO : Repository<Self::COMPONENT>;

    fn component_repo(&self) -> &Self::COMPONENT_REPO;
    fn component_repo_mut(&mut self) -> &mut Self::COMPONENT_REPO;
}

pub trait HaveProject {
    type PROJECT : Project;

    fn project(&self) -> &Self::PROJECT;
    fn project_mut(&mut self) -> &mut Self::PROJECT;
}

impl<PROJECT: Project> HaveRepository<PROJECT::COMPONENT> for PROJECT {
    type REPO = PROJECT::COMPONENT_REPO;

    fn repository(&self) -> &Self::REPO {
        self.component_repo()
    }

    fn repository_mut(&mut self) -> &Self::REPO {
        self.component_repo_mut()
    }
}

