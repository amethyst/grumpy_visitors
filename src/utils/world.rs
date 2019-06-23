use amethyst::ecs::{Entities, Entity, Join, ReadStorage, WriteStorage};
use rand::{self, Rng};

use crate::{
    components::{Monster, WorldPosition},
    data_resources::GameScene,
    Vector2,
};

pub fn closest_monster(
    missile_position: Vector2,
    world_positions: &WriteStorage<'_, WorldPosition>,
    entities: &Entities<'_>,
    monsters: &ReadStorage<'_, Monster>,
) -> Option<(Entity, Vector2)> {
    (world_positions, entities, monsters).join().fold(
        None,
        |res, (monster_position, monster, _)| {
            if let Some((closest_monster, closest_monster_position)) = res {
                if (closest_monster_position - missile_position).norm_squared()
                    > (**monster_position - missile_position).norm_squared()
                {
                    Some((monster, **monster_position))
                } else {
                    Some((closest_monster, closest_monster_position))
                }
            } else {
                Some((monster, **monster_position))
            }
        },
    )
}

pub fn random_scene_position(game_scene: &GameScene) -> Vector2 {
    let mut rng = rand::thread_rng();
    Vector2::new(
        rng.gen_range(
            -game_scene.half_size().x.as_f32(),
            game_scene.half_size().x.as_f32(),
        )
        .into(),
        rng.gen_range(
            -game_scene.half_size().y.as_f32(),
            game_scene.half_size().y.as_f32(),
        )
        .into(),
    )
}
