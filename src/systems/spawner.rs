use amethyst::{
    core::{Time, Transform},
    ecs::{Entities, ReadExpect, System, WriteExpect, WriteStorage},
    renderer::{Material, MeshHandle},
};

use crate::{
    components::{Monster, WorldPosition},
    data_resources::{GameScene, MonsterDefinitions},
    factories::create_monster,
    models::SpawnActions,
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
        WriteStorage<'s, MeshHandle>,
        WriteStorage<'s, Material>,
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
        let SpawnActions(ref mut spawn_actions) = *spawn_actions;
        for _spawn_action in spawn_actions.drain(..) {
            let ghoul = monster_definitions
                .0
                .get("Ghoul")
                .expect("Failed to get Ghoul monster definition");

            create_monster(
                -game_scene.half_size() - Vector2::new(100.0, 100.0),
                Vector2::new(0.0, 0.0),
                time.absolute_time(),
                ghoul,
                entities.build_entity(),
                &mut transforms,
                &mut meshes,
                &mut materials,
                &mut world_positions,
                &mut monsters,
            )
        }
    }
}
