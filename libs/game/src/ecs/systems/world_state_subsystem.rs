use amethyst::ecs::Entities;

use ha_core::ecs::{
    components::{
        missile::Missile, Dead, Monster, Player, PlayerActions, PlayerLastCastedSpells,
        WorldPosition,
    },
    resources::world::SavedWorldState,
};

use crate::ecs::systems::WriteStorageCell;

pub struct WorldStateSubsystem<'s> {
    pub entities: &'s Entities<'s>,
    pub players: WriteStorageCell<'s, Player>,
    pub player_actions: WriteStorageCell<'s, PlayerActions>,
    pub player_last_casted_spells: WriteStorageCell<'s, PlayerLastCastedSpells>,
    pub monsters: WriteStorageCell<'s, Monster>,
    pub missiles: WriteStorageCell<'s, Missile>,
    pub world_positions: WriteStorageCell<'s, WorldPosition>,
    pub dead: WriteStorageCell<'s, Dead>,
}

impl<'s> WorldStateSubsystem<'s> {
    pub fn save_world_state(&self, saved_world_state: &mut SavedWorldState) {
        saved_world_state.players =
            SavedWorldState::copy_from_write_storage(&self.entities, &*self.players.borrow_mut());
        saved_world_state.player_actions = SavedWorldState::copy_from_write_storage(
            &self.entities,
            &*self.player_actions.borrow_mut(),
        );
        saved_world_state.player_last_casted_spells = SavedWorldState::copy_from_write_storage(
            &self.entities,
            &*self.player_last_casted_spells.borrow_mut(),
        );
        saved_world_state.monsters =
            SavedWorldState::copy_from_write_storage(&self.entities, &*self.monsters.borrow_mut());
        saved_world_state.missiles =
            SavedWorldState::copy_from_write_storage(&self.entities, &*self.missiles.borrow_mut());
        saved_world_state.world_positions = SavedWorldState::copy_from_write_storage(
            &self.entities,
            &*self.world_positions.borrow_mut(),
        );
        saved_world_state.dead =
            SavedWorldState::copy_from_write_storage(&self.entities, &*self.dead.borrow_mut());
    }

    pub fn load_from_world_state(&self, saved_world_state: &SavedWorldState) {
        SavedWorldState::load_storage_from(
            &mut self.players.borrow_mut(),
            &saved_world_state.players,
        );
        SavedWorldState::load_storage_from(
            &mut self.player_actions.borrow_mut(),
            &saved_world_state.player_actions,
        );
        SavedWorldState::load_storage_from(
            &mut self.player_last_casted_spells.borrow_mut(),
            &saved_world_state.player_last_casted_spells,
        );
        SavedWorldState::load_storage_from(
            &mut self.monsters.borrow_mut(),
            &saved_world_state.monsters,
        );
        SavedWorldState::load_storage_from(
            &mut self.missiles.borrow_mut(),
            &saved_world_state.missiles,
        );
        SavedWorldState::load_storage_from(
            &mut self.world_positions.borrow_mut(),
            &saved_world_state.world_positions,
        );
        SavedWorldState::load_storage_from(&mut self.dead.borrow_mut(), &saved_world_state.dead);
    }
}
