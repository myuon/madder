extern crate serde_json;

#[derive(Serialize, Deserialize)]
pub struct Entity<ENTITY, ID> {
    pub id: ID,

    #[serde(flatten)]
    pub entity: ENTITY,
}

impl<ENTITY, ID> Entity<ENTITY, ID> {
    pub fn new(id: ID, entity: ENTITY) -> Entity<ENTITY, ID> {
        Entity {
            id: id,
            entity: entity,
        }
    }
}

// memcache repository
pub trait Repository<ENTITY> {
    fn create(&mut self, ENTITY) -> String;
    fn get(&self, &str) -> &ENTITY;
    fn list(&self) -> Vec<Entity<&ENTITY, &str>>;
    fn update(&mut self, String, ENTITY);
    fn delete(&mut self, &str);
}

// for internal use
pub trait MutRepository<ENTITY> : Repository<ENTITY> {
    fn get_mut(&mut self, &str) -> &mut ENTITY;
}
