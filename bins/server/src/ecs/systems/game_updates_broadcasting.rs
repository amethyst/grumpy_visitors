use amethyst::ecs::{Join, ReadStorage, System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::{
        components::NetConnectionModel,
        resources::world::ServerWorldUpdates,
        system_data::{game_state_helper::GameStateHelper, time::GameTimeService},
    },
    net::{server_message::ServerMessagePayload, NetConnection},
};
use ha_game::utils::net::send_message_unreliable;

use crate::ecs::resources::LastBroadcastedFrame;

const BROADCAST_FRAME_INTERVAL: u64 = 5;

#[derive(Default)]
pub struct GameUpdatesBroadcastingSystem;

impl<'s> System<'s> for GameUpdatesBroadcastingSystem {
    type SystemData = (
        GameTimeService<'s>,
        GameStateHelper<'s>,
        WriteExpect<'s, ServerWorldUpdates>,
        WriteExpect<'s, LastBroadcastedFrame>,
        ReadStorage<'s, NetConnectionModel>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_state_helper,
            mut server_world_updates,
            mut last_broadcasted_frame,
            net_connection_models,
            mut net_connections,
        ): Self::SystemData,
    ) {
        if !game_state_helper.multiplayer_is_running() {
            return;
        }

        let last_broadcasted_frame = &mut last_broadcasted_frame.0;

        let is_time_to_broadcast = game_time_service
            .game_frame_number()
            .wrapping_sub(*last_broadcasted_frame)
            > BROADCAST_FRAME_INTERVAL;
        if !is_time_to_broadcast {
            return;
        }
        *last_broadcasted_frame = game_time_service.game_frame_number();

        let (latest_update_number, latest_update_frame_number) = {
            let latest_update = server_world_updates
                .updates
                .back()
                .expect("Expected at least one ServerWorldUpdate");
            (latest_update.0, latest_update.1.frame_number)
        };

        // We'll use these to drop server updates that are no longer needed.
        let mut oldest_actual_update = latest_update_number + 1;
        let mut oldest_actual_update_index = 0;

        for (i, (net_connection_model, net_connection)) in
            (&net_connection_models, &mut net_connections)
                .join()
                .enumerate()
        {
            let last_acknowledged_is_older = net_connection_model
                .last_acknowledged_update
                .map(|last_acknowledged| oldest_actual_update > last_acknowledged)
                .unwrap_or(true);
            if last_acknowledged_is_older {
                oldest_actual_update = net_connection_model.last_acknowledged_update.unwrap_or(0);
                oldest_actual_update_index = i;
            }

            // Gather the updates this client needs based on its last_acknowledged_update.
            let mut oldest_added_frame = latest_update_frame_number + 1;
            let updates = server_world_updates
                .updates
                .iter()
                .rev()
                .take_while(|update| Some(update.0) > net_connection_model.last_acknowledged_update)
                .filter_map(move |update| {
                    let update = &update.1;
                    // We may store some repetitive updates, so we need to filter them out.
                    if oldest_added_frame > update.frame_number {
                        oldest_added_frame = update.frame_number;
                        Some(update.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let updates = updates.into_iter().rev().collect();

            send_message_unreliable(
                net_connection,
                &ServerMessagePayload::UpdateWorld {
                    id: latest_update_number,
                    updates,
                },
            );
        }

        // We don't need to store these updates anymore, as clients have already acknowledged them.
        if oldest_actual_update <= latest_update_number {
            server_world_updates
                .updates
                .drain(0..=oldest_actual_update_index);
        }
    }
}
