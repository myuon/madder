
struct ProjectUsecase<P>;

impl<PROJECT: Project> ProjectUsecase<PROJECT> {
    pub fn add_component(project: PROJECT, component: PROJECT::COMPONENT) {
        project.component_repo_mut().create(component);
    }
}

