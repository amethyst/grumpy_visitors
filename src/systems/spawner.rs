use amethyst::{
    assets::Handle,
    core::{Float, Time, Transform},
    ecs::{Entities, ReadExpect, System, WriteExpect, WriteStorage},
    renderer::{Material, Mesh},
};
use num;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::models::{MonsterAction, MonsterActionType};
use crate::{
    components::{Monster, WorldPosition},
    data_resources::{GameScene, MonsterDefinitions},
    factories::create_monster,
    models::{MonsterDefinition, SpawnActions, SpawnType},
    Vector2,
};

pub struct SpawnerSystem;

impl<'s> System<'s> for SpawnerSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, Time>,
        ReadExpect<'s, MonsterDefinitions>,
        ReadExpect<'s, GameScene>,
        WriteExpect<'s, SpawnActions>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Mesh>>,
        WriteStorage<'s, Handle<Material>>,
        WriteStorage<'s, Monster>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (
            entities,
            time,
            monster_definitions,
            game_scene,
            mut spawn_actions,
            mut transforms,
            mut meshes,
            mut materials,
            mut monsters,
            mut world_positions,
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
                 spawn_action: MonsterAction,
                 monster_definition: &MonsterDefinition| {
                    create_monster(
                        position,
                        spawn_action,
                        monster_definition,
                        entities.build_entity(),
                        &mut transforms,
                        &mut meshes,
                        &mut materials,
                        &mut world_positions,
                        &mut monsters,
                    )
                };

            match spawn_action.spawn_type {
                SpawnType::Random => {
                    for _ in 0..spawn_action.monsters.num {
                        let (side_start, side_end, _) = spawning_side(rand::random(), &game_scene);
                        let d = side_start - side_end;
                        let random_displacement = Vector2::new(
                            if d.x == 0.0.into() {
                                0.0.into()
                            } else {
                                (rng.gen_range(0.0, d.x.as_f32().abs()) * d.x.as_f32().signum())
                                    .into()
                            },
                            if d.y == 0.0.into() {
                                0.0.into()
                            } else {
                                (rng.gen_range(0.0, d.y.as_f32().abs()) * d.y.as_f32().signum())
                                    .into()
                            },
                        );
                        let position = side_start + random_displacement;
                        spawn_monster(position, MonsterAction::idle(time.absolute_time()), ghoul);
                    }
                }
                SpawnType::Borderline => {
                    let spawn_margin = Float::from(50.0);
                    let (side_start, side_end, destination) =
                        spawning_side(rand::random(), &game_scene);
                    let d = (side_start - side_end) / spawn_margin;
                    let monsters_to_spawn =
                        num::Float::max(d.x.as_f32().abs(), d.y.as_f32().abs()).round() as u8;
                    let spawn_distance =
                        (side_end - side_start) / Float::from(f32::from(monsters_to_spawn));

                    let mut position = side_start;
                    for _ in 0..monsters_to_spawn {
                        let action = MonsterAction {
                            started_at: time.absolute_time(),
                            action_type: MonsterActionType::Move(position + destination),
                        };
                        spawn_monster(position, action, ghoul);
                        position += spawn_distance;
                    }
                }
            }
        }
    }
}

fn spawning_side(side: Side, game_scene: &GameScene) -> (Vector2, Vector2, Vector2) {
    let scene_halfsize = game_scene.dimensions / Float::from(2.0);
    let border_distance = Float::from(100.0);
    let padding = Float::from(25.0);
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
            Vector2::new(0.0.into(), -game_scene.dimensions.y + border_distance),
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
            Vector2::new(-game_scene.dimensions.x + border_distance, 0.0.into()),
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
            Vector2::new(0.0.into(), game_scene.dimensions.y - border_distance),
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
            Vector2::new(game_scene.dimensions.x - border_distance, 0.0.into()),
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
