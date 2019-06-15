use amethyst::{
    animation::{get_animation_set, AnimationControlSet, AnimationSet},
    core::{Named, Parent, Transform},
    ecs::{Entities, Join, ReadStorage, System, WriteStorage},
    renderer::SpriteRender,
};

use crate::{
    components::{Monster, Player},
    AnimationId, Vector3,
};

pub struct AnimationSystem;

impl<'s> System<'s> for AnimationSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Named>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
    );

    fn run(
        &mut self,
        (
            entities,
            players,
            _monsters,
            parents,
            named_entities,
            animation_sets,
            mut transforms,
            mut animation_control_sets,
        ): Self::SystemData,
    ) {
        for (entity, parent, named, _animation_set, transform) in (
            &entities,
            &parents,
            &named_entities,
            &animation_sets,
            &mut transforms,
        )
            .join()
        {
            let control_set = get_animation_set(&mut animation_control_sets, entity).unwrap();

            // TODO: set rate depending on base speed.
            if let Some(player) = players.get(parent.entity) {
                if player.velocity.norm_squared() > 0.0.into() {
                    control_set.set_rate(AnimationId::Walk, 1.0);
                } else {
                    control_set.set_rate(AnimationId::Walk, 0.0);
                }

                let direction = if named.name == "hero_legs" {
                    Vector3::new(
                        player.walking_direction.x,
                        player.walking_direction.y,
                        transform.translation().z,
                    )
                } else {
                    Vector3::new(
                        player.looking_direction.x,
                        player.looking_direction.y,
                        transform.translation().z,
                    )
                };
                // TODO: educate myself about quaternions and rewrite that?
                transform.face_towards(Vector3::new(0.0.into(), 0.0.into(), 1.0.into()), direction);
            }
        }
    }
}
