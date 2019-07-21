use amethyst::{
    ecs::{Entities, Entity, Join, WriteStorage},
    ui::UiTransform,
    window::ScreenDimensions,
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

    pub fn find_with_mut_transform(&mut self, id: &str) -> Option<(Entity, &mut UiTransform)> {
        (&*self.entities, &mut self.storage)
            .join()
            .find(|(_, transform)| transform.id == id)
            .map(|(entity, transform)| (entity, transform))
    }
}

pub fn update_fullscreen_container(
    ui_finder: &mut UiFinderMut,
    id: &str,
    screen_dimensions: &ScreenDimensions,
) {
    if let Some((_, ui_background_transform)) = ui_finder.find_with_mut_transform(id) {
        ui_background_transform.width = screen_dimensions.width();
        ui_background_transform.height = screen_dimensions.height();
    }
}
