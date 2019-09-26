use amethyst::{
    ecs::{prelude::World, Entity, ReadExpect, ReadStorage},
    shred::{ResourceId, SystemData},
};

use crate::{
    ecs::{components::EntityNetMetadata, resources::net::EntityNetMetadataStorage},
    net::NetIdentifier,
};

#[derive(SystemData)]
pub struct EntityNetMetadataService<'s> {
    storage: ReadExpect<'s, EntityNetMetadataStorage>,
    entity_net_metadata: ReadStorage<'s, EntityNetMetadata>,
}

impl<'s> EntityNetMetadataService<'s> {
    pub fn get_entity(&self, entity_net_id: NetIdentifier) -> Entity {
        self.storage.get_entity(entity_net_id)
    }

    pub fn get_entity_net_metadata(&self, entity: Entity) -> EntityNetMetadata {
        *self
            .entity_net_metadata
            .get(entity)
            .expect("Expected EntityNetMetadata")
    }
}
