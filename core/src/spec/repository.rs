
pub trait Repository<Entity> {
    fn create(&mut self, Entity);
    fn get(&self, usize) -> Entity;
    fn list(&self) -> Vec<Entity>;
    fn update(&mut self, usize, Entity);
    fn delete(&mut self, usize);
}

pub trait HaveRepository<Entity> {
    type REPO : Repository<Entity>;

    fn repository(&self) -> &Self::REPO;
    fn repository_mut(&mut self) -> &Self::REPO;
}

