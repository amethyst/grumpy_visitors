use amethyst::ecs::{Join, ReadStorage, System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::{
        components::NetConnectionModel,
        resources::world::{ServerWorldUpdate, ServerWorldUpdates},
        system_data::{game_state_helper::GameStateHelper, time::GameTimeService},
    },
    net::{server_message::ServerMessagePayload, NetConnection},
};
use ha_game::utils::net::send_message_reliable;

const BROADCAST_FRAME_INTERVAL: u64 = 5;

#[derive(Default)]
pub struct GameUpdatesBroadcastingSystem {
    last_broadcasted_frame: u64,
}

impl<'s> System<'s> for GameUpdatesBroadcastingSystem {
    type SystemData = (
        GameTimeService<'s>,
        GameStateHelper<'s>,
        WriteExpect<'s, ServerWorldUpdates>,
        ReadStorage<'s, NetConnectionModel>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_state_helper,
            mut server_world_updates,
            net_connection_models,
            mut net_connections,
        ): Self::SystemData,
    ) {
        if !game_state_helper.multiplayer_is_running() {
            return;
        }

        let is_time_to_broadcast = game_time_service
            .game_frame_number()
            .wrapping_sub(self.last_broadcasted_frame)
            > BROADCAST_FRAME_INTERVAL;
        if !is_time_to_broadcast {
            return;
        }
        self.last_broadcasted_frame = game_time_service.game_frame_number();

        let latest_update = server_world_updates
            .updates
            .back()
            .expect("Expected at least one ServerWorldUpdate")
            .0;

        // We'll use these to drop server updates that are no longer needed.
        let mut oldest_actual_update = latest_update + 1;
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

            let mut merged_server_updates: Vec<ServerWorldUpdate> = Vec::with_capacity(
                latest_update as usize
                    - net_connection_model.last_acknowledged_update.unwrap_or(0) as usize,
            );

            // Here we check which update a client acknowledged the last and gather all the
            // updates a client haven't acknowledged, merging all the new updates with the old ones,
            // as we may alter updates for a certain frame more than once.
            for (server_update_id, server_update) in server_world_updates.updates.iter().cloned() {
                let is_not_acknowledged = net_connection_model
                    .last_acknowledged_update
                    .map(|update_id| update_id < server_update_id)
                    .unwrap_or(true);
                if is_not_acknowledged {
                    if let Some(existing_update) = merged_server_updates
                        .iter_mut()
                        .find(|update| update.frame_number == server_update.frame_number)
                    {
                        existing_update.merge_another_update(server_update)
                    } else {
                        merged_server_updates.push(server_update)
                    }
                }
            }

            send_message_reliable(
                net_connection,
                &ServerMessagePayload::UpdateWorld {
                    id: latest_update,
                    updates: merged_server_updates,
                },
            );
        }

        // We don't need to store these updates anymore, as clients have already acknowledged them.
        if oldest_actual_update <= latest_update {
            server_world_updates
                .updates
                .drain(0..=oldest_actual_update_index);
        }
    }
}
