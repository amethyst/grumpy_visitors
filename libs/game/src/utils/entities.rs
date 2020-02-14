use amethyst::ecs::{storage::GenericReadStorage, Entity};
#[cfg(feature = "client")]
use amethyst::{
    animation::{AnimationCommand, AnimationControlSet, AnimationSet, EndControl},
    core::{Named, ParentHierarchy},
    ecs::{ReadExpect, ReadStorage, WriteStorage},
    renderer::sprite::SpriteRender,
};

#[cfg(feature = "client")]
use gv_animation_prefabs::AnimationId;
use gv_core::ecs::{
    components::{missile::Missile, Dead},
    system_data::time::GameTimeService,
};

use crate::ecs::systems::missile::{MISSILE_LIFESPAN_SECS, MISSILE_TIME_TO_FADE};

pub fn is_dead(
    entity: Entity,
    dead: &impl GenericReadStorage<Component = Dead>,
    frame_number: u64,
) -> bool {
    dead.get(entity)
        .map_or(false, |dead| dead.is_dead(frame_number))
}

#[cfg(feature = "client")]
pub fn body_part_entity(
    parent_hierarchy: &ReadExpect<ParentHierarchy>,
    named: &ReadStorage<Named>,
    entity: Entity,
    body_part_name: &str,
) -> Option<Entity> {
    parent_hierarchy
        .children(entity)
        .iter()
        .find(|child_entity| {
            if let Some(entity_name) = named.get(**child_entity) {
                if entity_name.name == body_part_name {
                    return true;
                }
            }
            false
        })
        .cloned()
}

#[cfg(feature = "client")]
pub fn play_animation(
    parent_hierarchy: &ReadExpect<ParentHierarchy>,
    named: &ReadStorage<Named>,
    animation_sets: &ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
    animation_control_sets: &mut WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>,
    entity: Entity,
    body_part_name: &str,
    animation_id: AnimationId,
) {
    let body_part_entity = body_part_entity(parent_hierarchy, named, entity, body_part_name);
    if body_part_entity.is_none() {
        log::warn!(
            "Couldn't find the body part and play an animation: {}",
            body_part_name
        );
        return;
    }
    let body_part_entity = body_part_entity.unwrap();

    let animation_control_set = animation_control_sets.get_mut(body_part_entity);
    if let Some(animation_control_set) = animation_control_set {
        let animation_set = animation_sets
            .get(body_part_entity)
            .expect("Expected AnimationSet for an entity with AnimationControlSet");
        animation_control_set.add_animation(
            animation_id,
            &animation_set.get(&animation_id).unwrap(),
            EndControl::Stay,
            1.0,
            AnimationCommand::Start,
        );
    }
}

#[cfg(feature = "client")]
pub fn remove_animation(
    parent_hierarchy: &ReadExpect<ParentHierarchy>,
    named: &ReadStorage<Named>,
    animation_control_sets: &mut WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>,
    entity: Entity,
    body_part_name: &str,
    animation_id: AnimationId,
) {
    let body_part_entity = body_part_entity(parent_hierarchy, named, entity, body_part_name);
    if body_part_entity.is_none() {
        log::warn!(
            "Couldn't find the body part and remove an animation: {}",
            body_part_name
        );
        return;
    }
    let body_part_entity = body_part_entity.unwrap();

    let animation_control_set = animation_control_sets.get_mut(body_part_entity);
    if let Some(animation_control_set) = animation_control_set {
        animation_control_set.abort(animation_id);
    }
}

/// Returns values within the range [0.1; 1.0].
/// Energy start dropping below 1.0 on (MISSILE_LIFESPAN_SECS - MISSILE_TIME_TO_FADE).
pub fn missile_energy(
    missile: &Missile,
    is_dead: bool,
    game_time_service: &GameTimeService,
    frame_number: u64,
) -> f32 {
    let energy = ((MISSILE_LIFESPAN_SECS
        - game_time_service.seconds_between_frames(frame_number, missile.frame_spawned))
        / MISSILE_TIME_TO_FADE)
        .clamp(0.0, 1.0);
    if energy == 0.0 {
        return 0.0;
    }
    if is_dead {
        return 1.0;
    }
    energy
}
