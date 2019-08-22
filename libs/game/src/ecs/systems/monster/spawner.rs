use amethyst::ecs::{ReadExpect, System, WriteExpect};
use num;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use ha_core::{
    actions::{
        mob::MobAction,
        monster_spawn::{SpawnActions, SpawnType},
        Action,
    },
    ecs::{resources::GameLevelState, system_data::time::GameTimeService},
    math::{Vector2, ZeroVector},
};

use crate::ecs::{
    factories::MonsterFactory,
    resources::{MonsterDefinition, MonsterDefinitions},
};

pub struct MonsterSpawnerSystem;

impl<'s> System<'s> for MonsterSpawnerSystem {
    type SystemData = (
        GameTimeService<'s>,
        ReadExpect<'s, MonsterDefinitions>,
        ReadExpect<'s, GameLevelState>,
        WriteExpect<'s, SpawnActions>,
        MonsterFactory<'s>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            monster_definitions,
            game_level_state,
            mut spawn_actions,
            mut monster_factory,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let SpawnActions(ref mut spawn_actions) = *spawn_actions;
        for spawn_action in spawn_actions.drain(..) {
            let ghoul = monster_definitions
                .0
                .get("Ghoul")
                .expect("Failed to get Ghoul monster definition");

            let mut spawn_monster =
                |position: Vector2,
                 action: Action<MobAction>,
                 monster_definition: &MonsterDefinition| {
                    let destination = if let Some(MobAction::Move(destination)) = action.action {
                        destination
                    } else {
                        Vector2::zero()
                    };
                    monster_factory.create(
                        monster_definition.clone(),
                        position,
                        destination,
                        action,
                    );
                };

            match spawn_action.spawn_type {
                SpawnType::Random => {
                    for _ in 0..spawn_action.monsters.num {
                        let (side_start, side_end, _) =
                            spawning_side(rand::random(), &game_level_state);
                        let d = side_start - side_end;
                        let random_displacement = Vector2::new(
                            if d.x == 0.0 {
                                0.0
                            } else {
                                rng.gen_range(0.0, d.x.abs()) * d.x.signum()
                            },
                            if d.y == 0.0 {
                                0.0
                            } else {
                                rng.gen_range(0.0, d.y.abs()) * d.y.signum()
                            },
                        );
                        let position = side_start + random_displacement;
                        spawn_monster(position, Action::default(), ghoul);
                    }
                }
                SpawnType::Borderline => {
                    let spawn_margin = 50.0;
                    let (side_start, side_end, destination) =
                        spawning_side(rand::random(), &game_level_state);
                    let d = (side_start - side_end) / spawn_margin;
                    let monsters_to_spawn = num::Float::max(d.x.abs(), d.y.abs()).round() as u8;
                    let spawn_distance = (side_end - side_start) / f32::from(monsters_to_spawn);

                    let mut position = side_start;
                    for _ in 0..monsters_to_spawn {
                        let action = Action {
                            frame_number: game_time_service.game_frame_number(),
                            action: Some(MobAction::Move(position + destination)),
                        };
                        spawn_monster(position, action, ghoul);
                        position += spawn_distance;
                    }
                }
            }
        }
    }
}

fn spawning_side(side: Side, game_level_state: &GameLevelState) -> (Vector2, Vector2, Vector2) {
    let scene_halfsize = game_level_state.dimensions / 2.0;
    let border_distance = 100.0;
    let padding = 25.0;
    match side {
        Side::Top => (
            Vector2::new(
                -scene_halfsize.x + padding,
                scene_halfsize.y + border_distance,
            ),
            Vector2::new(
                scene_halfsize.x - padding,
                scene_halfsize.y + border_distance,
            ),
            Vector2::new(0.0, -game_level_state.dimensions.y + border_distance),
        ),
        Side::Right => (
            Vector2::new(
                scene_halfsize.x + border_distance,
                scene_halfsize.y - padding,
            ),
            Vector2::new(
                scene_halfsize.x + border_distance,
                -scene_halfsize.y + padding,
            ),
            Vector2::new(-game_level_state.dimensions.x + border_distance, 0.0),
        ),
        Side::Bottom => (
            Vector2::new(
                scene_halfsize.x - padding,
                -scene_halfsize.y - border_distance,
            ),
            Vector2::new(
                -scene_halfsize.x + padding,
                -scene_halfsize.y - border_distance,
            ),
            Vector2::new(0.0, game_level_state.dimensions.y - border_distance),
        ),
        Side::Left => (
            Vector2::new(
                -scene_halfsize.x - border_distance,
                -scene_halfsize.y + padding,
            ),
            Vector2::new(
                -scene_halfsize.x - border_distance,
                scene_halfsize.y - padding,
            ),
            Vector2::new(game_level_state.dimensions.x - border_distance, 0.0),
        ),
    }
}

enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen_range(0, 4) {
            0 => Side::Top,
            1 => Side::Right,
            2 => Side::Bottom,
            _ => Side::Left,
        }
    }
}
