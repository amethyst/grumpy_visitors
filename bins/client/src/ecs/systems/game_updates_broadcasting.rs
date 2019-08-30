use amethyst::ecs::{Join, ReadExpect, System, WriteExpect, WriteStorage};

use std::iter::FromIterator;

use ha_core::{
    ecs::{
        resources::{
            net::MultiplayerGameState,
            world::{ClientWorldUpdates, ImmediatePlayerActionsUpdates, PlayerLookActionUpdates},
            GameEngineState,
        },
        system_data::time::GameTimeService,
    },
    net::{client_message::ClientMessagePayload, NetConnection},
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
        WriteExpect<'s, ClientWorldUpdates>,
        WriteStorage<'s, NetConnection>,
    );

    fn run(
        &mut self,
        (
            game_time_service,
            game_engine_state,
            multiplayer_game_state,
            mut client_world_updates,
            mut net_connections,
        ): Self::SystemData,
    ) {
        if !(*game_engine_state == GameEngineState::Playing && multiplayer_game_state.is_playing) {
            return;
        }

        let net_connection = (&mut net_connections)
            .join()
            .next()
            .expect("Expected a server connection");

        if !client_world_updates.walk_action_updates.is_empty() {
            send_message_reliable(
                net_connection,
                &ClientMessagePayload::WalkActions(ImmediatePlayerActionsUpdates {
                    frame_number: game_time_service.game_frame_number(),
                    updates: client_world_updates.walk_action_updates.clone(),
                }),
            );
            client_world_updates.walk_action_updates.clear();
        }

        if !client_world_updates.cast_action_updates.is_empty() {
            send_message_reliable(
                net_connection,
                &ClientMessagePayload::CastActions(ImmediatePlayerActionsUpdates {
                    frame_number: game_time_service.game_frame_number(),
                    updates: client_world_updates.cast_action_updates.clone(),
                }),
            );
            client_world_updates.cast_action_updates.clear();
        }

        let is_time_to_broadcast = game_time_service
            .game_frame_number()
            .wrapping_sub(self.last_broadcasted_frame)
            > BROADCAST_FRAME_INTERVAL;
        if !is_time_to_broadcast {
            return;
        }
        self.last_broadcasted_frame = game_time_service.game_frame_number();

        send_message_reliable(
            net_connection,
            &ClientMessagePayload::LookActions(PlayerLookActionUpdates {
                updates: Vec::from_iter(client_world_updates.look_actions_updates.drain(..)),
            }),
        );
    }
}
