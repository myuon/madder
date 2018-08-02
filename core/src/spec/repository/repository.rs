extern crate serde_json;

#[derive(Serialize, Deserialize)]
pub struct Entity<ENTITY, ID> {
    pub id: ID,

    #[serde(flatten)]
    pub entity: ENTITY,
}

pub trait Repository<ENTITY, ID> {
    fn create(&mut self, ENTITY);
    fn get(&self, usize) -> Entity<ENTITY, ID>;
    fn list(&self) -> Vec<Entity<ENTITY, ID>>;
    fn update(&mut self, ID, ENTITY);
    fn delete(&mut self, ID);
}

pub trait HaveRepository<ENTITY, ID> {
    type REPO : Repository<ENTITY, ID>;

    fn repository(&self) -> &Self::REPO;
    fn repository_mut(&mut self) -> &Self::REPO;
}
