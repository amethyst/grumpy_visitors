use amethyst::{
    ecs::{storage::GenericReadStorage, Entities, Entity, Join, World, WriteStorage},
    shred::{ResourceId, SystemData},
    ui::{UiText, UiTransform},
};

#[derive(SystemData)]
pub struct UiFinderMut<'s> {
    entities: Entities<'s>,
    storage: WriteStorage<'s, UiTransform>,
}

impl<'s> UiFinderMut<'s> {
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

    pub fn get_ui_text<'a>(
        &mut self,
        ui_texts: &'a impl GenericReadStorage<Component = UiText>,
        id: &str,
    ) -> Option<&'a String> {
        self.find(id)
            .and_then(move |entity| ui_texts.get(entity))
            .map(|ui_text| &ui_text.text)
    }

    pub fn get_ui_text_mut<'a>(
        &mut self,
        ui_texts: &'a mut WriteStorage<UiText>,
        id: &str,
    ) -> Option<&'a mut String> {
        self.find(id)
            .and_then(move |entity| ui_texts.get_mut(entity))
            .map(|ui_text| &mut ui_text.text)
    }
}
