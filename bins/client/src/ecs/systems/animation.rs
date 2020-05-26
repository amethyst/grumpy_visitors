use amethyst::{
    animation::{AnimationCommand, AnimationControlSet, AnimationSet, EndControl},
    core::{Named, Parent, Transform},
    ecs::{Entities, Join, ReadStorage, System, WriteStorage},
    renderer::SpriteRender,
};

use gv_animation_prefabs::AnimationId;
use gv_core::{
    ecs::{
        components::{Dead, Monster, Player},
        system_data::time::GameTimeService,
    },
    math::Vector3,
};
use gv_game::utils::entities::is_dead;

pub struct AnimationSystem;

impl<'s> System<'s> for AnimationSystem {
    type SystemData = (
        GameTimeService<'s>,
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Dead>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Named>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            entities,
            players,
            monsters,
            dead,
            parents,
            named_entities,
            animation_sets,
            mut transforms,
            mut animation_control_sets,
        ): Self::SystemData,
    ) {
        for (entity, parent, named, animation_set, transform) in (
            &entities,
            &parents,
            &named_entities,
            &animation_sets,
            &mut transforms,
        )
            .join()
        {
            let entity_is_dead =
                is_dead(parent.entity, &dead, game_time_service.game_frame_number());
            let control_set = animation_control_sets
                .entry(entity)
                .ok()
                .map(|entry| {
                    entry.or_insert_with(|| {
                        let mut control_set = AnimationControlSet::default();
                        // On death we abort the Walk animation and start Death one with
                        // EndControl::Stay. As amethyst removes aborted and finished animations
                        // and also removes empty control sets, we end up with the control set being
                        // recreated here. Setting Init command lets the sampler keep the last frame
                        // of the last played animation (Death).
                        let command = if entity_is_dead {
                            AnimationCommand::Init
                        } else {
                            AnimationCommand::Start
                        };
                        if players.contains(parent.entity) || monsters.contains(parent.entity) {
                            control_set.add_animation(
                                AnimationId::Walk,
                                &animation_set.get(&AnimationId::Walk).unwrap(),
                                EndControl::Loop(None),
                                1.0,
                                command,
                            );
                        }
                        control_set
                    })
                })
                .expect("Expected an initialized AnimationControlSet");

            let player = players.get(parent.entity);
            let monster = monsters.get(parent.entity);

            // TODO: set rate depending on base speed.
            let entity_velocity = player
                .map(|player| player.velocity)
                .or_else(|| monster.map(|monster| monster.velocity));
            if let Some(entity_velocity) = entity_velocity {
                let rate = if entity_is_dead || entity_velocity.norm_squared() == 0.0 {
                    0.0
                } else {
                    1.0
                };
                control_set.set_rate(AnimationId::Walk, rate);
            }

            if let Some(player) = player {
                let direction = if named.name == "mage_legs" {
                    Vector3::new(
                        -player.walking_direction.x,
                        -player.walking_direction.y,
                        transform.translation().z,
                    )
                } else {
                    Vector3::new(
                        -player.looking_direction.x,
                        -player.looking_direction.y,
                        transform.translation().z,
                    )
                };
                // TODO: educate myself about quaternions and rewrite that?
                transform.face_towards(Vector3::new(0.0, 0.0, 1.0), direction);
            } else if let Some(monster) = monster {
                let direction = Vector3::new(
                    monster.facing_direction.x,
                    monster.facing_direction.y,
                    transform.translation().z,
                );
                transform.face_towards(Vector3::new(0.0, 0.0, 1.0), direction);
            }
        }
    }
}
