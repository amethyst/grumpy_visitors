use amethyst::ecs::{Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use ha_core::{
    ecs::{
        components::NetConnectionModel,
        resources::{
            net::MultiplayerGameState,
            world::{ServerWorldUpdate, ServerWorldUpdates},
            GameEngineState,
        },
        system_data::time::GameTimeService,
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
        ReadExpect<'s, GameEngineState>,
        WriteExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, ServerWorldUpdates>,
        ReadStorage<'s, NetConnectionModel>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_engine_state,
            multiplayer_game_state,
            mut server_world_updates,
            net_connection_models,
            mut net_connections,
        ): Self::SystemData,
    ) {
        if *game_engine_state != GameEngineState::Playing {
            return;
        }

        let is_time_to_broadcast = game_time_service
            .game_frame_number()
            .wrapping_sub(self.last_broadcasted_frame)
            > BROADCAST_FRAME_INTERVAL;
        if !(multiplayer_game_state.is_playing && is_time_to_broadcast) {
            return;
        }
        self.last_broadcasted_frame = game_time_service.game_frame_number();

        let latest_update = server_world_updates.updates[server_world_updates.updates.len() - 1].0;

        // We'll use these to drop server updates that are no more needed.
        let mut oldest_actual_update = latest_update + 1;
        let mut oldest_actual_update_index = 0;

        for (i, (net_connection_model, net_connection)) in
            (&net_connection_models, &mut net_connections)
                .join()
                .enumerate()
        {
            if oldest_actual_update > net_connection_model.last_acknowledged_update {
                oldest_actual_update = net_connection_model.last_acknowledged_update;
                oldest_actual_update_index = i;
            }

            let mut merged_server_updates: Vec<ServerWorldUpdate> = Vec::with_capacity(
                latest_update as usize - net_connection_model.last_acknowledged_update as usize,
            );

            for (server_update_id, server_update) in server_world_updates.updates.iter().cloned() {
                if net_connection_model.last_acknowledged_update < server_update_id {
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
