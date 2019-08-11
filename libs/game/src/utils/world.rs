use amethyst::ecs::{
    storage::{MaskedStorage, Storage},
    Entities, Entity, Join,
};
use rand::{self, Rng};

use std::ops::Deref;

use ha_core::{
    ecs::{
        components::{Monster, WorldPosition},
        resources::GameLevelState,
    },
    math::Vector2,
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

pub fn find_first_hit_monster<
    DT: Deref<Target = MaskedStorage<Monster>>,
    DP: Deref<Target = MaskedStorage<WorldPosition>>,
>(
    object_position: Vector2,
    object_radius: f32,
    targets: &Storage<'_, Monster, DT>,
    target_positions: &Storage<'_, WorldPosition, DP>,
    entities: &Entities<'_>,
) -> Option<Entity> {
    (target_positions, entities, targets)
        .join()
        .find(|(target_position, _, target)| {
            let distance_squared = (object_position - ***target_position).norm_squared();
            let impact_distance = object_radius + target.radius;
            let impact_distance_squared = impact_distance * impact_distance;
            distance_squared <= impact_distance_squared
        })
        .map(|result| result.1)
}

pub fn random_scene_position(game_scene: &GameLevelState) -> Vector2 {
    let mut rng = rand::thread_rng();
    Vector2::new(
        rng.gen_range(
            -game_scene.dimensions_half_size().x,
            game_scene.dimensions_half_size().x,
        ),
        rng.gen_range(
            -game_scene.dimensions_half_size().y,
            game_scene.dimensions_half_size().y,
        ),
    )
}
