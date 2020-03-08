use amethyst::ecs::{Entities, Join, WriteStorage};

use gv_core::{
    ecs::{
        components::{
            damage_history::{DamageHistory, DamageHistoryEntries},
            Dead, EntityNetMetadata, Monster, Player,
        },
        resources::net::EntityNetMetadataStorage,
        system_data::time::GameTimeService,
    },
    net::NetUpdate,
    profile_scope,
};

use crate::{
    ecs::{
        system_data::GameStateHelper,
        systems::{OutcomingNetUpdates, WriteExpectCell, WriteStorageCell},
    },
    utils::entities::is_dead,
};

pub struct DamageSubsystem<'s> {
    pub game_state_helper: &'s GameStateHelper<'s>,
    pub game_time_service: &'s GameTimeService<'s>,
    pub entities: &'s Entities<'s>,
    pub entity_net_metadata_storage: WriteExpectCell<'s, EntityNetMetadataStorage>,
    pub entity_net_metadata: WriteStorageCell<'s, EntityNetMetadata>,
    pub players: WriteStorageCell<'s, Player>,
    pub monsters: WriteStorageCell<'s, Monster>,
    pub damage_histories: WriteStorageCell<'s, DamageHistory>,
    pub dead: WriteStorageCell<'s, Dead>,
}

impl<'s> DamageSubsystem<'s> {
    /// We need to reset damage entries when replaying world state in multiplayer.
    pub fn reset_damage_entries(&self, frame_number: u64) {
        profile_scope!("DamageSubsystem::reset_damage_entries");
        let entity_net_metadata = self.entity_net_metadata.borrow();
        let mut damage_histories = self.damage_histories.borrow_mut();
        for (damage_history, entity) in (&mut *damage_histories, self.entities).join() {
            let is_spawned = entity_net_metadata
                .get(entity)
                .map_or(true, |entity_net_metadata| {
                    entity_net_metadata.spawned_frame_number <= frame_number
                });
            // We won't have damage history for an entity that is not spawned,
            // without this check the code will panic.
            if is_spawned {
                damage_history.reset_entries(frame_number);
            }
        }
    }

    pub fn process_damage_history(
        &self,
        frame_number: u64,
        damage_histories_updates: Option<&Vec<NetUpdate<DamageHistoryEntries>>>,
        outcoming_net_updates: &mut OutcomingNetUpdates,
    ) {
        profile_scope!("DamageSubsystem::process_damage_history");
        let mut damage_histories = self.damage_histories.borrow_mut();

        self.fetch_incoming_net_updates(
            frame_number,
            &mut damage_histories,
            damage_histories_updates,
        );

        let entity_net_metadata = self.entity_net_metadata.borrow();
        let mut players = self.players.borrow_mut();
        let mut monsters = self.monsters.borrow_mut();
        let mut dead = self.dead.borrow_mut();

        for (entity, damage_history) in (self.entities, &*damage_histories).join() {
            if is_dead(entity, &*dead, frame_number) {
                continue;
            }

            let entity_net_metadata = entity_net_metadata.get(entity);

            if self.game_state_helper.is_multiplayer() {
                let is_not_spawned = entity_net_metadata
                    .expect("Expected EntityNetMetadata in multiplayer")
                    .spawned_frame_number
                    > frame_number;
                if is_not_spawned {
                    continue;
                }
            }

            if self.game_state_helper.is_multiplayer() && self.game_state_helper.is_authoritative()
            {
                put_outcoming_net_updates(
                    *entity_net_metadata.expect("Expected EntityNetMetadata in multiplayer"),
                    outcoming_net_updates,
                    damage_history.get_entries(frame_number).clone(),
                );
            }

            for damage_history_entry in &damage_history.get_entries(frame_number).entries {
                if let Some(player) = players.get_mut(entity) {
                    player.health -= damage_history_entry.damage;
                } else if let Some(monster) = monsters.get_mut(entity) {
                    monster.health -= damage_history_entry.damage;
                };
            }
        }

        for entity in (self.entities).join() {
            let health = {
                if let Some(player) = players.get_mut(entity) {
                    &mut player.health
                } else if let Some(monster) = monsters.get_mut(entity) {
                    &mut monster.health
                } else {
                    continue;
                }
            };
            if *health < 0.001 {
                let is_already_dead = dead
                    .get(entity)
                    .map_or(false, |dead| frame_number >= dead.dead_since_frame);
                if !is_already_dead {
                    *health = 0.0;
                    let dead_since_frame = frame_number + 1;
                    let frame_acknowledged =
                        dead_since_frame.max(self.game_time_service.game_frame_number());
                    dead.insert(entity, Dead::new(dead_since_frame, frame_acknowledged))
                        .expect("Expected to insert Dead component");
                }
            } else {
                // If an entity has Dead component for whatever reason, but it has positive health,
                // remove it's Dead component.
                // TODO: do we really need this code?
                let will_be_killed = dead
                    .get(entity)
                    .map_or(false, |dead| frame_number + 1 == dead.dead_since_frame);
                if will_be_killed {
                    dead.remove(entity)
                        .expect("Expected to remove Dead component");
                }
            }
        }
    }

    #[cfg(feature = "client")]
    fn fetch_incoming_net_updates(
        &self,
        frame_number: u64,
        damage_histories: &mut WriteStorage<DamageHistory>,
        incoming_net_updates: Option<&Vec<NetUpdate<DamageHistoryEntries>>>,
    ) {
        let entity_net_metadata_storage = self.entity_net_metadata_storage.borrow();
        let incoming_net_updates =
            incoming_net_updates.expect("Expected net updates on client side");
        for net_update in incoming_net_updates {
            assert_eq!(net_update.data.frame_number, frame_number);
            let entity = entity_net_metadata_storage.get_entity(net_update.entity_net_id);
            if entity.is_none() {
                log::error!(
                    "Couldn't find an entity (net id: {}) to apply damage entries",
                    net_update.entity_net_id
                );
                return;
            }
            let entity = entity.unwrap();
            let damage_history = damage_histories
                .get_mut(entity)
                .expect("Expected DamageHistory component");
            for damage_history_entry in net_update.data.entries.clone() {
                damage_history.add_entry(frame_number, damage_history_entry);
            }
        }
    }

    #[cfg(not(feature = "client"))]
    fn fetch_incoming_net_updates(
        &self,
        _frame_number: u64,
        _damage_histories: &mut WriteStorage<DamageHistory>,
        _incoming_net_updates: Option<&Vec<NetUpdate<DamageHistoryEntries>>>,
    ) {
    }
}

#[cfg(feature = "client")]
fn put_outcoming_net_updates(
    _entity_net_metadata: EntityNetMetadata,
    _outcoming_net_updates: &mut OutcomingNetUpdates,
    _damage_history_entries: DamageHistoryEntries,
) {
}

#[cfg(not(feature = "client"))]
fn put_outcoming_net_updates(
    entity_net_metadata: EntityNetMetadata,
    outcoming_net_updates: &mut OutcomingNetUpdates,
    damage_history_entries: DamageHistoryEntries,
) {
    assert_eq!(
        outcoming_net_updates.frame_number,
        damage_history_entries.frame_number
    );
    if !damage_history_entries.entries.is_empty() {
        log::trace!("Added net update {:?}", damage_history_entries);
        outcoming_net_updates
            .damage_histories_updates
            .push(NetUpdate {
                entity_net_id: entity_net_metadata.id,
                data: damage_history_entries,
            })
    }
}
