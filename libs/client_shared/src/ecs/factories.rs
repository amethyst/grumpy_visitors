#![allow(clippy::type_repetition_in_bounds)]

use amethyst::{
    core::{Parent, Transform},
    ecs::{Entities, Entity, ReadExpect, WriteStorage},
    renderer::Camera,
    window::ScreenDimensions,
};
use shred_derive::SystemData;

#[derive(SystemData)]
pub struct CameraFactory<'s> {
    entities: Entities<'s>,
    screen_dimensions: ReadExpect<'s, ScreenDimensions>,
    cameras: WriteStorage<'s, Camera>,
    transforms: WriteStorage<'s, Transform>,
    parents: WriteStorage<'s, Parent>,
}

impl<'s> CameraFactory<'s> {
    pub fn create(&mut self, player: Entity) {
        let (width, height) = (
            self.screen_dimensions.width(),
            self.screen_dimensions.height(),
        );
        let transform = {
            let mut transform = Transform::default();
            transform.set_translation_z(100.0);
            transform
        };

        self.entities
            .build_entity()
            .with(Camera::standard_2d(width, height), &mut self.cameras)
            .with(transform, &mut self.transforms)
            .with(Parent::new(player), &mut self.parents)
            .build();
    }
}
