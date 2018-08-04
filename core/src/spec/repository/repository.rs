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

// Currently, we need to fix ID type as String
// since Rust does not support type-family or something like that feature

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

// for deserializer
pub trait RepositoryLoader<ENTITY> : Repository<ENTITY> {
    fn load_table(&mut self, Vec<Entity<ENTITY, String>>);
}

