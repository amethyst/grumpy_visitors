use amethyst::{
    animation::{
        get_animation_set, AnimationCommand, AnimationControlSet, AnimationSet, EndControl,
    },
    ecs::{Entities, Join, ReadStorage, World, WriteStorage},
    renderer::SpriteRender,
};

use ha_animation_prefabs::AnimationId;

pub fn start_hero_animations(world: &mut World) {
    world.exec(
        |(entities, animation_sets, mut control_sets): (
            Entities,
            ReadStorage<AnimationSet<AnimationId, SpriteRender>>,
            WriteStorage<AnimationControlSet<AnimationId, SpriteRender>>,
        )| {
            for (entity, animation_set) in (&entities, &animation_sets).join() {
                let control_set = get_animation_set(&mut control_sets, entity).unwrap();
                control_set.add_animation(
                    AnimationId::Walk,
                    &animation_set.get(&AnimationId::Walk).unwrap(),
                    EndControl::Loop(None),
                    1.0,
                    AnimationCommand::Start,
                );
            }
        },
    );
}
