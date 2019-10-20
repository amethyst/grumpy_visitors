use amethyst::ecs::{
    storage::{GenericReadStorage, MaskedStorage, Storage},
    Entities, Entity, Join,
};
use rand::{self, Rng};

use std::ops::Deref;

use gv_core::{
    actions::monster_spawn::Side,
    ecs::{
        components::{Dead, Monster, WorldPosition},
        resources::GameLevelState,
    },
    math::Vector2,
};

use crate::{
    ecs::systems::{AggregatedOutcomingUpdates, OutcomingNetUpdates},
    utils::entities::is_dead,
};

pub fn closest_monster<
    DW: Deref<Target = MaskedStorage<WorldPosition>>,
    DM: Deref<Target = MaskedStorage<Monster>>,
    G: GenericReadStorage<Component = Dead>,
>(
    missile_position: Vector2,
    world_positions: &Storage<'_, WorldPosition, DW>,
    entities: &Entities<'_>,
    monsters: &Storage<'_, Monster, DM>,
    dead: &G,
    frame_number: u64,
) -> Option<(Entity, Vector2)> {
    (world_positions, entities, monsters)
        .join()
        .filter(|(_, entity, _)| !is_dead(*entity, dead, frame_number))
        .fold(None, |res, (monster_position, monster, _)| {
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
        })
}

pub fn find_first_hit_monster<
    DT: Deref<Target = MaskedStorage<Monster>>,
    DP: Deref<Target = MaskedStorage<WorldPosition>>,
    G: GenericReadStorage<Component = Dead>,
>(
    object_position: Vector2,
    object_radius: f32,
    targets: &Storage<'_, Monster, DT>,
    target_positions: &Storage<'_, WorldPosition, DP>,
    entities: &Entities<'_>,
    dead: &G,
    frame_number: u64,
) -> Option<Entity> {
    (target_positions, entities, targets)
        .join()
        .filter(|(_, entity, _)| !is_dead(*entity, dead, frame_number))
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

pub fn random_spawn_position(game_level_state: &GameLevelState) -> Vector2 {
    let mut rng = rand::thread_rng();

    let (side_start, side_end, _) = spawning_side(rand::random(), &game_level_state);
    let d = side_end - side_start;
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

    side_start + random_displacement
}

pub fn spawning_side(side: Side, game_level_state: &GameLevelState) -> (Vector2, Vector2, Vector2) {
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

#[cfg(feature = "client")]
pub fn outcoming_net_updates_mut(
    aggregated_updates: &mut AggregatedOutcomingUpdates,
    _frame_number: u64,
    _current_frame_number: u64,
) -> &mut OutcomingNetUpdates {
    aggregated_updates
}

#[cfg(not(feature = "client"))]
pub fn outcoming_net_updates_mut(
    aggregated_updates: &mut AggregatedOutcomingUpdates,
    frame_number: u64,
    current_frame_number: u64,
) -> &mut OutcomingNetUpdates {
    aggregated_updates.get_update(frame_number, current_frame_number)
}
