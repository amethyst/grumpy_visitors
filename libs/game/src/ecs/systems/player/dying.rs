use amethyst::ecs::{
    prelude::ComponentEvent, BitSet, Entities, Join, ReadExpect, ReadStorage, ReaderId, System,
    WriteExpect, WriteStorage,
};

use ha_core::{
    ecs::{
        components::{damage_history::DamageHistory, Dead, Player},
        resources::{net::MultiplayerGameState, GameLevelState},
    },
    math::{Vector2, ZeroVector},
};

pub struct PlayerDyingSystem {
    damage_history_reader: ReaderId<ComponentEvent>,
    players_hit: BitSet,
}

impl PlayerDyingSystem {
    pub fn new(damage_history_reader: ReaderId<ComponentEvent>) -> Self {
        Self {
            damage_history_reader,
            players_hit: BitSet::new(),
        }
    }
}

impl<'s> System<'s> for PlayerDyingSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, DamageHistory>,
        ReadExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, GameLevelState>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Dead>,
    );

    fn run(
        &mut self,
        (
            entities,
            damage_histories,
            multiplayer_game_state,
            mut game_level_state,
            mut players,
            mut dead,
        ): Self::SystemData,
    ) {
        self.players_hit.clear();
        let events = damage_histories
            .channel()
            .read(&mut self.damage_history_reader);

        for event in events {
            if let ComponentEvent::Modified(index) = event {
                let entity = entities.entity(*index);
                let damage_history = damage_histories
                    .get(entity)
                    .expect("Expected a DamageHistory");
                let player = players.get_mut(entity);
                if let Some(player) = player {
                    for entry in &damage_history.last_entries().entries {
                        player.health -= entry.damage;
                    }
                    self.players_hit.add(*index);
                }
            }
        }

        for (player_entity, player, _) in (&entities, &mut players, &self.players_hit).join() {
            if player.health <= 0.001 {
                if !multiplayer_game_state.is_playing {
                    game_level_state.is_over = true;
                }
                player.velocity = Vector2::zero();
                dead.insert(player_entity, Dead)
                    .expect("Expected to insert Dead component");
            }
        }
    }
}
