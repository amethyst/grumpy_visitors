use amethyst::{
    ecs::{Entities, Join, ReadExpect, System, Write, WriteExpect, WriteStorage},
    network::simulation::TransportResource,
};

use gv_core::{
    actions::{
        player::{PlayerCastAction, PlayerWalkAction},
        ClientActionUpdate, IdentifiableAction,
    },
    ecs::{
        components::NetConnectionModel,
        resources::{
            net::{ActionUpdateIdProvider, MultiplayerGameState, MultiplayerRoomPlayer},
            world::{
                FramedUpdates, ImmediatePlayerActionsUpdates, PlayerLookActionUpdates,
                ReceivedClientActionUpdates, ServerWorldUpdates, LAG_COMPENSATION_FRAMES_LIMIT,
                PAUSE_FRAME_THRESHOLD,
            },
            GameEngineState, NewGameEngineState,
        },
        system_data::time::GameTimeService,
    },
    net::{
        client_message::ClientMessagePayload,
        server_message::{DisconnectReason, ServerMessagePayload},
        NetEvent, NetIdentifier, NetUpdate, INTERPOLATION_FRAME_DELAY,
    },
};
use gv_game::{
    ecs::resources::ConnectionEvents,
    utils::net::{broadcast_message_reliable, send_message_reliable},
    PLAYER_COLORS,
};

use crate::ecs::resources::{HostClientAddress, LastBroadcastedFrame};

const HEARTBEAT_FRAME_INTERVAL: u64 = 2;

pub struct ServerNetworkSystem {
    host_connection_id: NetIdentifier,
    last_heartbeat_frame: u64,
}

impl ServerNetworkSystem {
    pub fn new() -> Self {
        Self {
            host_connection_id: 0,
            last_heartbeat_frame: 0,
        }
    }
}

impl<'s> System<'s> for ServerNetworkSystem {
    type SystemData = (
        GameTimeService<'s>,
        Entities<'s>,
        ReadExpect<'s, GameEngineState>,
        ReadExpect<'s, LastBroadcastedFrame>,
        WriteExpect<'s, ConnectionEvents>,
        WriteExpect<'s, HostClientAddress>,
        WriteExpect<'s, MultiplayerGameState>,
        WriteExpect<'s, NewGameEngineState>,
        WriteExpect<'s, FramedUpdates<ReceivedClientActionUpdates>>,
        WriteExpect<'s, ServerWorldUpdates>,
        WriteExpect<'s, ActionUpdateIdProvider>,
        WriteStorage<'s, NetConnectionModel>,
        Write<'s, TransportResource>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(
        &mut self,
        (
            game_time_service,
            entities,
            game_engine_state,
            last_broadcasted_frame,
            mut connection_events,
            mut host_client_address,
            mut multiplayer_game_state,
            mut new_game_engine_state,
            mut framed_updates,
            mut server_world_updates,
            mut action_update_id_provider,
            mut net_connection_models,
            mut transport,
        ): Self::SystemData,
    ) {
        if let Some(host_client_address) = host_client_address.0.take() {
            let net_connection_model = NetConnectionModel::new(0, host_client_address);
            self.host_connection_id = 0;
            log::info!("Sending a Handshake message to a hosting client");
            send_message_reliable(
                &mut transport,
                &net_connection_model,
                &ServerMessagePayload::Handshake {
                    net_id: 0,
                    is_host: true,
                },
            );
            entities
                .build_entity()
                .with(net_connection_model, &mut net_connection_models)
                .build();
        }

        for connection_event in connection_events.0.drain(..) {
            let connection_id = connection_event.connection_id;
            let mut connection_model = (&mut net_connection_models)
                .join()
                .find(|net_connection_model| net_connection_model.id == connection_id)
                .expect("Expected to find a NetConnection");

            // Handle ignored events if the game is already started.
            if multiplayer_game_state.is_playing {
                let is_ignored = match connection_event.event {
                    NetEvent::Message(ClientMessagePayload::JoinRoom { .. }) => {
                        let player_is_in_game = multiplayer_game_state
                            .players
                            .iter()
                            .any(|player| player.connection_id == connection_id);
                        if !player_is_in_game {
                            log::info!(
                                "A new client ({}) {} tried to connect while the game has already started",
                                connection_id,
                                connection_model.addr
                            );
                            send_message_reliable(
                                &mut transport,
                                connection_model,
                                &ServerMessagePayload::Disconnect(
                                    DisconnectReason::GameIsAlreadyStarted,
                                ),
                            );
                        }
                        true
                    }
                    NetEvent::Message(ClientMessagePayload::StartHostedGame) => {
                        log::info!(
                            "A client ({}) {} tried to start the game while it's already started",
                            connection_id,
                            connection_model.addr
                        );
                        true
                    }
                    _ => false,
                };

                if is_ignored {
                    continue;
                }
            }

            if !multiplayer_game_state.is_playing {
                let is_ignored = match connection_event.event {
                    NetEvent::Message(ClientMessagePayload::AcknowledgeWorldUpdate(_)) => true,
                    NetEvent::Message(ClientMessagePayload::WalkActions(_)) => true,
                    NetEvent::Message(ClientMessagePayload::CastActions(_)) => true,
                    NetEvent::Message(ClientMessagePayload::LookActions(_)) => true,
                    _ => false,
                };

                if is_ignored {
                    continue;
                }
            }

            match connection_event.event {
                NetEvent::Message(ClientMessagePayload::JoinRoom { nickname }) => {
                    // TODO: we'll need a more reliable way to determine the host in future.
                    let is_host = multiplayer_game_state.players.is_empty();
                    if is_host {
                        self.host_connection_id = connection_id;
                    }

                    log::info!("Sending a Handshake message: {}", connection_id);
                    send_message_reliable(
                        &mut transport,
                        connection_model,
                        &ServerMessagePayload::Handshake {
                            net_id: connection_id,
                            is_host,
                        },
                    );

                    log::info!(
                        "A client ({}) has joined the room: {}",
                        connection_id,
                        nickname
                    );
                    if let Some(player) = multiplayer_game_state
                        .update_players()
                        .iter_mut()
                        .find(|player| player.connection_id == connection_id)
                    {
                        log::info!("The player already existed, updating the nickname only");
                        player.nickname = nickname;
                    } else {
                        let color_index = multiplayer_game_state.players.len();
                        multiplayer_game_state
                            .update_players()
                            .push(MultiplayerRoomPlayer {
                                connection_id,
                                entity_net_id: 0,
                                nickname,
                                is_host: self.host_connection_id == connection_id,
                                color: PLAYER_COLORS[color_index],
                            });
                    }
                }

                NetEvent::Message(ClientMessagePayload::StartHostedGame)
                    if connection_id == self.host_connection_id
                        && !multiplayer_game_state.is_playing =>
                {
                    multiplayer_game_state.is_playing = true;
                    new_game_engine_state.0 = GameEngineState::Playing;
                }

                NetEvent::Message(ClientMessagePayload::WalkActions(actions)) => {
                    log::trace!(
                        "Received WalkAction updates (frame {}): {:?}",
                        game_time_service.game_frame_number(),
                        actions
                    );
                    let discarded_actions = add_walk_actions(
                        &mut *framed_updates,
                        actions,
                        game_time_service.game_frame_number(),
                    );

                    if !discarded_actions.is_empty() {
                        log::trace!(
                            "{} walk actions have been discarded",
                            discarded_actions.len()
                        );
                        send_message_reliable(
                            &mut transport,
                            connection_model,
                            &ServerMessagePayload::DiscardWalkActions(discarded_actions),
                        );
                    }
                }

                NetEvent::Message(ClientMessagePayload::CastActions(actions)) => {
                    add_cast_actions(
                        &mut *framed_updates,
                        actions,
                        &mut *action_update_id_provider,
                        game_time_service.game_frame_number(),
                    );
                }

                NetEvent::Message(ClientMessagePayload::LookActions(actions)) => {
                    add_look_actions(
                        &mut *framed_updates,
                        actions,
                        game_time_service.game_frame_number(),
                    );
                }

                NetEvent::Message(ClientMessagePayload::AcknowledgeWorldUpdate(frame_number)) => {
                    connection_model.last_acknowledged_update =
                        Some(frame_number).max(connection_model.last_acknowledged_update);
                }

                NetEvent::Disconnected => {
                    multiplayer_game_state
                        .update_players()
                        .retain(|player| player.connection_id == connection_id);
                }

                _ => {}
            }
        }

        if let Some(players) = multiplayer_game_state.read_updated_players() {
            broadcast_message_reliable(
                &mut transport,
                (&net_connection_models).join(),
                &ServerMessagePayload::UpdateRoomPlayers(players.to_owned()),
            );
        }

        if game_time_service.engine_time().frame_number() - self.last_heartbeat_frame
            > HEARTBEAT_FRAME_INTERVAL
        {
            self.last_heartbeat_frame = game_time_service.engine_time().frame_number();
            broadcast_message_reliable(
                &mut transport,
                (&net_connection_models).join(),
                &ServerMessagePayload::Heartbeat,
            );
        }

        // Pause server if one of clients is lagging behind.
        if *game_engine_state == GameEngineState::Playing && multiplayer_game_state.is_playing {
            let mut lagging_players = Vec::new();
            for net_connection_model in (&net_connection_models).join() {
                let frames_since_last_pong = game_time_service
                    .engine_time()
                    .frame_number()
                    .saturating_sub(net_connection_model.ping_pong_data.last_ponged_frame);
                let average_lagging_behind =
                    net_connection_model.ping_pong_data.average_lagging_behind();

                let expected_client_frame_number = last_broadcasted_frame
                    .0
                    .saturating_sub(INTERPOLATION_FRAME_DELAY);

                let was_lagging = multiplayer_game_state
                    .lagging_players
                    .iter()
                    .any(|connection_id| *connection_id == net_connection_model.id);

                // If a player was already lagging we expect them to fully catch up with others.
                let is_catching_up = net_connection_model.ping_pong_data.last_stored_game_frame()
                    < expected_client_frame_number;

                log::trace!(
                    "Frames since last pong (client {}): {}",
                    net_connection_model.id,
                    frames_since_last_pong
                );
                log::trace!(
                    "Last_stored_game_frame (client {}): {}. Expected_client_frame_number: {}",
                    net_connection_model.id,
                    net_connection_model.ping_pong_data.last_stored_game_frame(),
                    expected_client_frame_number,
                );
                log::trace!(
                    "Average lagging behind (client {}): {}",
                    net_connection_model.id,
                    average_lagging_behind
                );

                if frames_since_last_pong > PAUSE_FRAME_THRESHOLD
                    || was_lagging && is_catching_up
                    || average_lagging_behind > PAUSE_FRAME_THRESHOLD
                {
                    lagging_players.push(net_connection_model.id);
                }
            }

            multiplayer_game_state.lagging_players = lagging_players.clone();
            if !multiplayer_game_state.waiting_for_players && !lagging_players.is_empty() {
                multiplayer_game_state.waiting_for_players_pause_id += 1;
                broadcast_message_reliable(
                    &mut transport,
                    (&net_connection_models).join(),
                    &ServerMessagePayload::PauseWaitingForPlayers {
                        id: multiplayer_game_state.waiting_for_players_pause_id,
                        players: lagging_players,
                    },
                );
                multiplayer_game_state.waiting_for_players = true;
            } else if multiplayer_game_state.waiting_for_players && lagging_players.is_empty() {
                broadcast_message_reliable(
                    &mut transport,
                    (&net_connection_models).join(),
                    &ServerMessagePayload::UnpauseWaitingForPlayers(
                        multiplayer_game_state.waiting_for_players_pause_id,
                    ),
                );
                multiplayer_game_state.waiting_for_players = false;
            }
        }

        // We should reserve new updates only if we're not paused. If we do it regardless, we'll
        // get redundant updates reserved.
        if *game_engine_state == GameEngineState::Playing
            && !(multiplayer_game_state.waiting_network
                || multiplayer_game_state.waiting_for_players)
        {
            let current_frame_number = game_time_service.game_frame_number();
            server_world_updates.reserve_new_updates(
                framed_updates
                    .oldest_updated_frame
                    .min(current_frame_number),
                current_frame_number,
            );
        }
    }
}

/// Returns discarded actions.
fn add_walk_actions(
    framed_updates: &mut FramedUpdates<ReceivedClientActionUpdates>,
    actions: ImmediatePlayerActionsUpdates<ClientActionUpdate<PlayerWalkAction>>,
    frame_number: u64,
) -> Vec<NetIdentifier> {
    let mut discarded_actions = Vec::new();

    let added_actions_frame_number = actions.frame_number;

    // Just ignore these updates, most probably these are lost packages from the previous game,
    // or the client is just bonkers.
    if added_actions_frame_number.saturating_sub(frame_number) > PAUSE_FRAME_THRESHOLD {
        return Vec::new();
    }

    let oldest_possible_frame = frame_number.saturating_sub(LAG_COMPENSATION_FRAMES_LIMIT as u64);
    let are_lag_compensated = added_actions_frame_number > oldest_possible_frame;
    let actual_frame = if are_lag_compensated {
        added_actions_frame_number
    } else {
        oldest_possible_frame
    };

    let is_badly_late = added_actions_frame_number
        < frame_number.saturating_sub(LAG_COMPENSATION_FRAMES_LIMIT as u64 * 2);
    for action in actions.updates {
        let is_added = {
            if is_badly_late {
                // If there was any accepted update after this one, we're going to skip it,
                // as it's impossible to postpone the other ones.
                !framed_updates
                    .updates
                    .iter()
                    .skip_while(|update| update.frame_number < added_actions_frame_number)
                    .any(|update| {
                        update
                            .walk_action_updates
                            .iter()
                            .any(|net_update| net_update.entity_net_id == action.entity_net_id)
                    })
            } else {
                true
            }
        };

        if is_added {
            let frames_to_move = oldest_possible_frame.saturating_sub(added_actions_frame_number);
            if !is_badly_late && frames_to_move > 0 {
                let mut moved_updates = Vec::with_capacity(LAG_COMPENSATION_FRAMES_LIMIT);
                for framed_update in framed_updates
                    .updates
                    .iter_mut()
                    .skip_while(|update| update.frame_number < actual_frame)
                {
                    if let Some(i) = framed_update
                        .walk_action_updates
                        .iter()
                        .position(|net_update| net_update.entity_net_id == action.entity_net_id)
                    {
                        let moved_update = framed_update.walk_action_updates.remove(i);
                        if framed_update.frame_number + frames_to_move > frame_number {
                            discarded_actions.push(moved_update.data.client_action_id);
                        } else {
                            moved_updates.push((framed_update.frame_number, moved_update));
                        }
                    }
                }

                let mut framed_updates_iter =
                    framed_updates.updates_iter_mut(actual_frame).peekable();
                for (moved_update_frame_number, moved_update) in moved_updates.into_iter() {
                    loop {
                        let framed_update = framed_updates_iter.peek().unwrap();
                        if framed_update.frame_number == moved_update_frame_number {
                            break;
                        }
                    }
                    framed_updates_iter
                        .next()
                        .expect("Expected a framed update to move a NetUpdate into")
                        .walk_action_updates
                        .push(moved_update);
                }
            }
            let updated_frame = framed_updates
                .update_frame(actual_frame)
                .unwrap_or_else(|| panic!("Expected a frame {}", actual_frame));

            log::trace!(
                "Added a walk action update for frame {} to frame {}",
                added_actions_frame_number,
                updated_frame.frame_number
            );

            updated_frame.walk_action_updates.push(action);
        } else {
            discarded_actions.push(action.data.client_action_id);
        }
    }

    discarded_actions
}

fn add_look_actions(
    framed_updates: &mut FramedUpdates<ReceivedClientActionUpdates>,
    actions: PlayerLookActionUpdates,
    frame_number: u64,
) {
    let frame_to_reserve = actions
        .updates
        .iter()
        .filter(|(_, updates)| !updates.is_empty())
        .map(|(frame_number, _)| frame_number)
        .max_by(|prev_frame_number, next_frame_number| prev_frame_number.cmp(next_frame_number));

    // Just ignore these updates, most probably these are lost packages from the previous game,
    // or the client is just bonkers.
    if frame_to_reserve.map_or(true, |frame_to_reserve| {
        frame_to_reserve.saturating_sub(frame_number) > PAUSE_FRAME_THRESHOLD
    }) {
        return;
    }

    if let Some(frame_to_reserve) = frame_to_reserve {
        framed_updates.reserve_updates(*frame_to_reserve);
    }

    let mut oldest_updated_frame = framed_updates.oldest_updated_frame;
    let oldest_possible_frame = frame_number.saturating_sub(LAG_COMPENSATION_FRAMES_LIMIT as u64);
    let mut framed_updates_iter = framed_updates.updates_iter_mut(oldest_possible_frame);

    'action_updates: for (update_frame_number, updates) in actions.updates {
        let mut framed_update = framed_updates_iter
            .next()
            .expect("Expected at least one framed update");

        if update_frame_number >= oldest_possible_frame {
            loop {
                if update_frame_number == framed_update.frame_number {
                    break;
                }
                framed_update = if let Some(framed_update) = framed_updates_iter.next() {
                    framed_update
                } else {
                    log::warn!(
                        "Server couldn't apply a look action update for frame {}, while being at frame {}",
                        update_frame_number,
                        frame_number,
                    );
                    break 'action_updates;
                }
            }
        }

        if !updates.is_empty() {
            oldest_updated_frame = oldest_updated_frame.min(framed_update.frame_number);
        }

        for update in updates {
            if let Some(i) = framed_update
                .look_action_updates
                .iter()
                .position(|net_update| net_update.entity_net_id == update.entity_net_id)
            {
                framed_update.look_action_updates[i] = update;
            } else {
                framed_update.look_action_updates.push(update);
            }
            log::trace!(
                "Added a look action update for frame {} to frame {}",
                update_frame_number,
                framed_update.frame_number
            );
        }
    }

    drop(framed_updates_iter);
    framed_updates.oldest_updated_frame = oldest_updated_frame;
}

fn add_cast_actions(
    framed_updates: &mut FramedUpdates<ReceivedClientActionUpdates>,
    actions: ImmediatePlayerActionsUpdates<ClientActionUpdate<PlayerCastAction>>,
    action_update_id_provider: &mut ActionUpdateIdProvider,
    frame_number: u64,
) {
    let added_actions_frame_number = actions.frame_number;

    // Just ignore these updates, most probably these are lost packages from the previous game,
    // or the client is just bonkers.
    if added_actions_frame_number.saturating_sub(frame_number) > PAUSE_FRAME_THRESHOLD {
        return;
    }

    let oldest_possible_frame = frame_number.saturating_sub(LAG_COMPENSATION_FRAMES_LIMIT as u64);
    let are_lag_compensated = added_actions_frame_number > oldest_possible_frame;
    let actual_frame = if are_lag_compensated {
        added_actions_frame_number
    } else {
        oldest_possible_frame
    };

    for action_update in actions.updates {
        let is_added = !framed_updates
            .updates
            .iter()
            .skip_while(|update| update.frame_number < actual_frame)
            .any(|update| {
                update
                    .cast_action_updates
                    .iter()
                    .any(|net_update| net_update.entity_net_id == action_update.entity_net_id)
            });

        if is_added {
            let updated_frame = framed_updates
                .update_frame(actual_frame)
                .unwrap_or_else(|| panic!("Expected a frame {}", actual_frame));

            log::trace!(
                "Added a walk action update for frame {} to frame {}",
                added_actions_frame_number,
                updated_frame.frame_number
            );

            updated_frame.cast_action_updates.push(NetUpdate {
                entity_net_id: action_update.entity_net_id,
                data: IdentifiableAction {
                    action_id: action_update_id_provider.next_update_id(),
                    action: action_update.data,
                },
            });
        }
    }
}
