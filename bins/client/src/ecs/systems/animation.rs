use amethyst::{
    animation::{get_animation_set, AnimationControlSet, AnimationSet},
    core::{Named, Parent, Transform},
    ecs::{Entities, Join, ReadStorage, System, WriteStorage},
    renderer::SpriteRender,
};

use gv_animation_prefabs::AnimationId;
use gv_core::{
    ecs::components::{Monster, Player},
    math::Vector3,
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
            monsters,
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
                if player.velocity.norm_squared() > 0.0 {
                    control_set.set_rate(AnimationId::Walk, 1.0);
                } else {
                    control_set.set_rate(AnimationId::Walk, 0.0);
                }
                log::info!("player {}", named.name);

                let direction = if named.name == "mage64_legs" {
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
                transform.face_towards(Vector3::new(0.0, 0.0, 1.0), direction);
            } else if let Some(monster) = monsters.get(parent.entity) {
                log::info!("monster {}", named.name);
                if monster.velocity.norm_squared() > 0.0 {
                    control_set.set_rate(AnimationId::Walk, 1.0);
                } else {
                    control_set.set_rate(AnimationId::Walk, 0.0);
                }
            }
        }
    }
}
