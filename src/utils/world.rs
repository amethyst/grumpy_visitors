use amethyst::ecs::{
    storage::{MaskedStorage, Storage},
    Entities, Entity, Join,
};
use rand::{self, Rng};

use std::ops::Deref;

use crate::{
    components::{Monster, WorldPosition},
    data_resources::GameScene,
    Vector2,
};

pub fn closest_monster<DW, DM>(
    missile_position: Vector2,
    world_positions: &Storage<'_, WorldPosition, DW>,
    entities: &Entities<'_>,
    monsters: &Storage<'_, Monster, DM>,
) -> Option<(Entity, Vector2)>
where
    DW: Deref<Target = MaskedStorage<WorldPosition>>,
    DM: Deref<Target = MaskedStorage<Monster>>,
{
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
