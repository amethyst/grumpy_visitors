use amethyst::{
    ecs::{Entities, Entity, Join, WriteStorage},
    ui::UiTransform,
};
use shred_derive::SystemData;

#[derive(SystemData)]
pub struct UiFinderMut<'a> {
    entities: Entities<'a>,
    storage: WriteStorage<'a, UiTransform>,
}

impl<'a> UiFinderMut<'a> {
    pub fn find(&self, id: &str) -> Option<Entity> {
        (&*self.entities, &self.storage)
            .join()
            .find(|(_, transform)| transform.id == id)
            .map(|(entity, _)| entity)
    }

    pub fn get_id_by_entity(&self, searched_entity: Entity) -> Option<String> {
        (&*self.entities, &self.storage)
            .join()
            .find(|(entity, _)| *entity == searched_entity)
            .map(|(_, transform)| transform.id.clone())
    }

    pub fn find_with_mut_transform(&mut self, id: &str) -> Option<(Entity, &mut UiTransform)> {
        (&*self.entities, &mut self.storage)
            .join()
            .find(|(_, transform)| transform.id == id)
            .map(|(entity, transform)| (entity, transform))
    }
}
