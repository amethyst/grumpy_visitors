use amethyst::ecs::{
    prelude::ComponentEvent, BitSet, Entities, Join, ReadStorage, ReaderId, System, WriteStorage,
};

use ha_core::ecs::{
    components::{damage_history::DamageHistory, Monster},
    system_data::game_state_helper::GameStateHelper,
};

pub struct MonsterDyingSystem {
    damage_history_reader: ReaderId<ComponentEvent>,
    monsters_hit: BitSet,
}

impl MonsterDyingSystem {
    pub fn new(damage_history_reader: ReaderId<ComponentEvent>) -> Self {
        Self {
            damage_history_reader,
            monsters_hit: BitSet::new(),
        }
    }
}

impl<'s> System<'s> for MonsterDyingSystem {
    type SystemData = (
        GameStateHelper<'s>,
        Entities<'s>,
        ReadStorage<'s, DamageHistory>,
        WriteStorage<'s, Monster>,
    );

    fn run(
        &mut self,
        (game_state_helper, entities, damage_histories, mut monsters): Self::SystemData,
    ) {
        if !game_state_helper.is_running() {
            return;
        }

        self.monsters_hit.clear();
        let events = damage_histories
            .channel()
            .read(&mut self.damage_history_reader);

        for event in events {
            if let ComponentEvent::Modified(index) = event {
                let entity = entities.entity(*index);
                let damage_history = damage_histories
                    .get(entity)
                    .expect("Expected a DamageHistory");
                let monster = monsters.get_mut(entity);
                if let Some(monster) = monster {
                    for entry in &damage_history.last_entries().entries {
                        monster.health -= entry.damage;
                    }
                    self.monsters_hit.add(*index);
                }
            }
        }

        for (monster_entity, monster, _) in (&entities, &mut monsters, &self.monsters_hit).join() {
            if monster.health <= 0.001 {
                entities.delete(monster_entity).unwrap();
            }
        }
    }
}
