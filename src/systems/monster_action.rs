use amethyst::ecs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};
use rand::{self, Rng};

use crate::{
    components::{Monster, Player, WorldPosition},
    data_resources::MonsterDefinitions,
    models::{AttackAction, MonsterAction, MonsterActionType},
    Vector2,
};
use std::time::Instant;

pub struct MonsterActionSystem;

impl<'s> System<'s> for MonsterActionSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, MonsterDefinitions>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, WorldPosition>,
        WriteStorage<'s, Monster>,
    );

    fn run(
        &mut self,
        (entities, _monster_definitions, players, world_positions, mut monsters): Self::SystemData,
    ) {
        let now = Instant::now();
        let mut rng = rand::thread_rng();
        for (mut monster, monster_position) in (&mut monsters, &world_positions).join() {
            let new_action_type = match monster.action.action_type {
                MonsterActionType::Idle => {
                    if let Some((entity, _player_position)) = find_player_in_radius(
                        (&entities, &players, &world_positions).join(),
                        monster_position.position,
                        200.0,
                    ) {
                        Some(MonsterActionType::Chase(entity))
                    } else {
                        let pos =
                            Vector2::new(rng.gen_range(0.0, 1024.0), rng.gen_range(0.0, 768.0));
                        Some(MonsterActionType::Move(pos))
                    }
                }
                _ => None,
            };

            let new_destination = if let Some(ref new_action_type) = new_action_type {
                match new_action_type {
                    MonsterActionType::Move(position) => Some(*position),
                    MonsterActionType::Chase(entity) => {
                        Some(world_positions.get(*entity).unwrap().position)
                    }
                    MonsterActionType::Attack(AttackAction { target, .. }) => {
                        Some(world_positions.get(*target).unwrap().position)
                    }
                    _ => None,
                }
            } else {
                None
            };

            if let Some(action_type) = new_action_type {
                monster.action = MonsterAction {
                    started_at: now,
                    action_type,
                }
            }

            if let Some(new_destination) = new_destination {
                monster.velocity =
                    (new_destination - monster_position.position).normalize() * 500.0;
            }
        }
    }
}

fn find_player_in_radius<'a>(
    mut players: impl Iterator<Item = (Entity, &'a Player, &'a WorldPosition)>,
    position: Vector2,
    radius: f32,
) -> Option<(Entity, &'a WorldPosition)> {
    let radius_squared = radius * radius;
    players
        .find(|(_, _, player_position)| {
            (position - player_position.position).norm_squared() < radius_squared
        })
        .map(|(entity, _, player_position)| (entity, player_position))
}
