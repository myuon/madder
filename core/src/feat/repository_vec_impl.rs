use spec::*;

pub trait HaveRepositoryImpl<Entity> {
    fn elements(&self) -> &Vec<Entity>;
    fn elements_mut(&mut self) -> &mut Vec<Entity>;
}

impl<Entity: Clone, R: HaveRepositoryImpl<Entity>> Repository<Entity> for R {
    fn create(&mut self, entity: Entity) {
        self.elements_mut().push(entity);
    }

    fn get(&self, index: usize) -> Entity {
        self.elements()[index].clone()
    }

    fn list(&self) -> Vec<Entity> {
        self.elements().clone()
    }

    fn update(&mut self, index: usize, entity: Entity) {
        self.elements_mut()[index] = entity;
    }

    fn delete(&mut self, index: usize) {
        self.elements_mut().remove(index);
    }
}

