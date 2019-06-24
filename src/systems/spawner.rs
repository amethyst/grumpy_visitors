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

use crate::{
    components::{DamageHistory, Monster, WorldPosition},
    data_resources::{EntityGraphics, GameScene, MonsterDefinitions},
    models::{
        common::MonsterDefinition,
        mob_actions::{MobAction, MobActionType},
        monster_spawn::{SpawnActions, SpawnType},
    },
    Vector2, ZeroVector,
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
        WriteStorage<'s, DamageHistory>,
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
            mut damage_histories,
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
                |position: Vector2, action: MobAction, monster_definition: &MonsterDefinition| {
                    let mut transform = Transform::default();
                    transform.set_translation_xyz(position.x, position.y, 11.0);
                    let destination = if let MobActionType::Move(destination) = action.action_type {
                        destination
                    } else {
                        Vector2::zero()
                    };

                    let MonsterDefinition {
                        name,
                        base_health,
                        base_speed: _base_speed,
                        base_attack: _base_attack,
                        graphics: EntityGraphics { mesh, material },
                        radius,
                    } = monster_definition.clone();
                    entities
                        .build_entity()
                        .with(mesh, &mut meshes)
                        .with(material, &mut materials)
                        .with(transform, &mut transforms)
                        .with(WorldPosition::new(position), &mut world_positions)
                        .with(
                            Monster {
                                health: base_health,
                                destination,
                                velocity: Vector2::zero(),
                                action,
                                name,
                                radius,
                            },
                            &mut monsters,
                        )
                        .with(DamageHistory::new(), &mut damage_histories)
                        .build();
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
                        spawn_monster(position, MobAction::idle(time.absolute_time()), ghoul);
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
                        let action = MobAction {
                            started_at: time.absolute_time(),
                            action_type: MobActionType::Move(position + destination),
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
