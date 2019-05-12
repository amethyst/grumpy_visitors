use amethyst::{
    animation::{get_animation_set, AnimationControlSet, AnimationSet},
    core::Parent,
    ecs::{Entities, Join, ReadStorage, System, WriteStorage},
    renderer::SpriteRender,
};

use crate::{
    components::{Monster, Player},
    AnimationId,
};

pub struct AnimationSystem;

impl<'s> System<'s> for AnimationSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
    );

    fn run(
        &mut self,
        (
            entities,
            players,
            _monsters,
            parents,
            animation_sets,
            mut animation_control_sets,
        ): Self::SystemData,
    ) {
        for (entity, parent, _animation_set) in (&entities, &parents, &animation_sets).join() {
            let control_set = get_animation_set(&mut animation_control_sets, entity).unwrap();

            // TODO: set rate depending on base speed.
            if let Some(player) = players.get(parent.entity) {
                if player.velocity.norm_squared() > 0.0 {
                    control_set.set_rate(AnimationId::Walk, 1.0)
                } else {
                    control_set.set_rate(AnimationId::Walk, 0.0)
                }
            }
        }
    }
}
