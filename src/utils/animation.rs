use amethyst::{
    animation::{
        get_animation_set, AnimationCommand, AnimationControlSet, AnimationSet, EndControl,
    },
    assets::{Handle, Prefab, PrefabLoader, ProgressCounter, RonFormat},
    ecs::{Entities, Join, ReadStorage, World, WriteStorage},
    renderer::SpriteRender,
};

use animation_prefabs::{AnimationId, GameSpritePrefab};

pub fn load_prefab(
    world: &mut World,
    progress_counter: &mut Option<ProgressCounter>,
) -> Handle<Prefab<GameSpritePrefab>> {
    world.exec(|loader: PrefabLoader<'_, GameSpritePrefab>| {
        loader.load(
            "resources/animation_metadata.ron",
            RonFormat,
            (),
            progress_counter.as_mut().unwrap(),
        )
    })
}

pub fn update_loading_prefab(world: &mut World, progress_counter: &mut Option<ProgressCounter>) {
    if let Some(ref counter) = progress_counter {
        if counter.is_complete() {
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
            *progress_counter = None;
        }
    }
}
