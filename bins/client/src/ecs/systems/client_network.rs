use amethyst::{
    ecs::{Entities, Join, ReadExpect, System, World, Write, WriteExpect, WriteStorage},
    network::simulation::{laminar::LaminarSocketResource, TransportResource},
    shred::{ResourceId, SystemData},
};

use std::{
    cmp::Ordering,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use gv_client_shared::ecs::resources::{ConnectionStatus, MultiplayerRoomState};
use gv_core::{
    actions::monster_spawn::SpawnActions,
    ecs::{
        components::NetConnectionModel,
        resources::{
            net::{MultiplayerGameState, MultiplayerRoomPlayer, PlayersNetStatus},
            world::{
                FramedUpdates, PlayerActionUpdates, ReceivedPlayerUpdate,
                ReceivedServerWorldUpdate, ServerWorldUpdate, PAUSE_FRAME_THRESHOLD,
            },
            GameEngineState, NewGameEngineState,
        },
        system_data::time::GameTimeService,
    },
    net::{
        client_message::ClientMessagePayload,
        server_message::{DisconnectReason, ServerMessage, ServerMessagePayload},
        NetEvent, NetIdentifier, INTERPOLATION_FRAME_DELAY,
    },
};
use gv_game::{
    ecs::resources::ConnectionEvents,
    utils::net::{send_message_reliable, send_message_unreliable},
};

use crate::ecs::resources::{
    LastAcknowledgedUpdate, ServerCommand, UiNetworkCommand, UiNetworkCommandResource,
};

const HEARTBEAT_FRAME_INTERVAL: u64 = 10;

#[derive(SystemData)]
pub struct ClientNetworkSystemData<'s> {
    game_time_service: GameTimeService<'s>,
    game_engine_state: ReadExpect<'s, GameEngineState>,
    entities: Entities<'s>,
    connection_events: WriteExpect<'s, ConnectionEvents>,
    multiplayer_room_state: WriteExpect<'s, MultiplayerRoomState>,
    multiplayer_game_state: WriteExpect<'s, MultiplayerGameState>,
    new_game_engine_sate: WriteExpect<'s, NewGameEngineState>,
    last_acknowledged_update: WriteExpect<'s, LastAcknowledgedUpdate>,
    framed_updates: WriteExpect<'s, FramedUpdates<ReceivedServerWorldUpdate>>,
    player_actions_updates: WriteExpect<'s, FramedUpdates<PlayerActionUpdates>>,
    spawn_actions: WriteExpect<'s, FramedUpdates<SpawnActions>>,
    server_command: WriteExpect<'s, ServerCommand>,
    ui_network_command: WriteExpect<'s, UiNetworkCommandResource>,
    players_net_status: WriteExpect<'s, PlayersNetStatus>,
    net_connection_models: WriteStorage<'s, NetConnectionModel>,
    transport: Write<'s, TransportResource>,
    laminar_socket: WriteExpect<'s, LaminarSocketResource>,
}

#[derive(Default)]
pub struct ClientNetworkSystem {
    session_id_autoinc: NetIdentifier,
    last_heartbeat_frame: u64,
    has_sent_join_message: bool,
    nickname: String,
}

impl ClientNetworkSystem {
    fn next_session_id(&mut self) -> NetIdentifier {
        let id = self.session_id_autoinc;
        self.session_id_autoinc = self.session_id_autoinc.wrapping_add(1);
        id
    }

    fn process_ui_network_command(
        &mut self,
        system_data: &mut ClientNetworkSystemData,
        ui_network_command: UiNetworkCommand,
    ) {
        match ui_network_command {
            UiNetworkCommand::Host {
                nickname,
                server_addr,
            } => {
                self.nickname = nickname;
                system_data.multiplayer_room_state.is_active = true;
                system_data.multiplayer_room_state.is_host = true;
                system_data.multiplayer_room_state.connection_status =
                    ConnectionStatus::Connecting(Instant::now());

                let mut host_client_addr = system_data
                    .laminar_socket
                    .get_mut()
                    .expect("Expected a LaminarSocket")
                    .local_addr()
                    .expect("Expected a local address for a Laminar socket");
                match &mut host_client_addr {
                    SocketAddr::V4(addr) => addr.set_ip(Ipv4Addr::new(127, 0, 0, 1)),
                    SocketAddr::V6(addr) => addr.set_ip(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                };
                if let Err(err) = system_data
                    .server_command
                    .start(server_addr, host_client_addr)
                {
                    log::error!("Couldn't start the server: {:?}", err);
                    system_data.multiplayer_room_state.connection_status =
                        ConnectionStatus::ServerStartFailed;
                }
            }

            UiNetworkCommand::Connect {
                nickname,
                server_addr,
            } => {
                self.nickname = nickname;
                system_data.multiplayer_room_state.is_active = true;
                system_data.multiplayer_room_state.is_host = false;
                system_data.multiplayer_room_state.connection_status =
                    ConnectionStatus::Connecting(Instant::now());

                let net_connection_model =
                    NetConnectionModel::new(0, self.next_session_id(), server_addr);

                log::info!("Sending a JoinRoom message");
                self.has_sent_join_message = true;
                send_message_reliable(
                    &mut system_data.transport,
                    &net_connection_model,
                    ClientMessagePayload::JoinRoom {
                        sent_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Expected a duration unix timestamp"),
                        nickname: self.nickname.clone(),
                    },
                );

                system_data
                    .entities
                    .build_entity()
                    .with(net_connection_model, &mut system_data.net_connection_models)
                    .build();
            }

            UiNetworkCommand::Kick { player_number } => send_message_reliable(
                &mut system_data.transport,
                server_connection(&mut system_data.net_connection_models),
                ClientMessagePayload::Kick {
                    kicked_connection_id: system_data.multiplayer_game_state.players[player_number]
                        .connection_id,
                },
            ),

            UiNetworkCommand::Start => {
                if system_data.multiplayer_room_state.is_host {
                    send_message_reliable(
                        &mut system_data.transport,
                        server_connection(&mut system_data.net_connection_models),
                        ClientMessagePayload::StartHostedGame,
                    );
                } else {
                    log::error!(
                        "Client check failed: only host can send a StartHostedGame message"
                    );
                }
            }

            UiNetworkCommand::Leave => {
                log::info!("Closing the connection with the server...");
                let net_connection_model =
                    server_connection(&mut system_data.net_connection_models);
                send_message_reliable(
                    &mut system_data.transport,
                    net_connection_model,
                    ClientMessagePayload::Disconnect,
                );
                net_connection_model.disconnected = true;
                system_data.multiplayer_room_state.connection_status =
                    if !system_data.multiplayer_room_state.is_host {
                        ConnectionStatus::Disconnected(DisconnectReason::Closed)
                    } else {
                        ConnectionStatus::Disconnecting
                    }
            }

            UiNetworkCommand::Reset => {
                self.has_sent_join_message = false;
                self.last_heartbeat_frame = 0;
                system_data.multiplayer_room_state.connection_status =
                    ConnectionStatus::NotConnected;
                system_data.multiplayer_game_state.reset();
                system_data.multiplayer_room_state.reset();
            }
        }
    }
}

fn update_room_players(
    multiplayer_game_state: &mut MultiplayerGameState,
    players: Vec<MultiplayerRoomPlayer>,
) {
    log::info!("Updated room players (player count: {})", players.len());
    *multiplayer_game_state.update_players() = players;
}

impl<'s> System<'s> for ClientNetworkSystem {
    type SystemData = ClientNetworkSystemData<'s>;

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, mut system_data: Self::SystemData) {
        if let Some(ui_network_command) = system_data.ui_network_command.command.take() {
            self.process_ui_network_command(&mut system_data, ui_network_command);
        }

        if !system_data.multiplayer_room_state.is_active {
            system_data.net_connection_models.clear();
            return;
        }

        if system_data.server_command.is_started() {
            if let Some(exit_status) = system_data.server_command.exit_status() {
                let code = exit_status.code().expect("Expected an exit status code");
                if code == 0 {
                    log::info!("The server has closed");
                    system_data.multiplayer_room_state.connection_status =
                        ConnectionStatus::Disconnected(DisconnectReason::Closed);
                } else {
                    log::error!("The server crashed with the exit code {}", code);
                    system_data.multiplayer_room_state.connection_status =
                        ConnectionStatus::Disconnected(DisconnectReason::ServerCrashed(code));
                }
                system_data.server_command.stop();
            }
        }

        if system_data.net_connection_models.count() == 0 {
            if system_data.multiplayer_game_state.is_playing
                && *system_data.game_engine_state == GameEngineState::Playing
            {
                system_data.multiplayer_game_state.is_disconnected = true;
            }
            return;
        }

        // TODO: implement rejecting incoming connections for client, cause this can fail badly.
        let net_connection_model = server_connection(&mut system_data.net_connection_models);
        for connection_event in system_data.connection_events.0.drain(..) {
            // Ignore all the messages for disconnected models, except for Disconnected or Handshake.
            if net_connection_model.disconnected {
                let ignore_event = !matches!(connection_event.event, NetEvent::Disconnected
                | NetEvent::Message(ServerMessage {
                    payload: ServerMessagePayload::Handshake { .. },
                    ..
                }));
                if ignore_event {
                    continue;
                }
            }

            if let NetEvent::Message(ServerMessage { session_id, .. }) = &connection_event.event {
                if *session_id != net_connection_model.session_id {
                    log::warn!("Ignoring a message with session id {} from a connection {} with session id {}", session_id, net_connection_model.id, net_connection_model.session_id);
                    continue;
                }
            }

            if system_data.multiplayer_game_state.is_playing {
                let ignore_event = match &connection_event.event {
                    NetEvent::Message(ServerMessage {
                        session_id: _,
                        payload,
                    }) => match payload {
                        ServerMessagePayload::Handshake { .. } => true,
                        ServerMessagePayload::UpdateRoomPlayers(_) => true,
                        ServerMessagePayload::StartGame(_) => true,
                        _ => false,
                    },
                    _ => false,
                };
                if ignore_event {
                    continue;
                }
            }

            match connection_event.event {
                NetEvent::Message(ServerMessage {
                    session_id: _,
                    payload,
                }) => {
                    match payload {
                        // Are covered by NetConnectionManager.
                        ServerMessagePayload::Heartbeat
                        | ServerMessagePayload::Ping(_)
                        | ServerMessagePayload::Pong { .. } => {}

                        ServerMessagePayload::Handshake {
                            net_id: connection_id,
                            is_host,
                        } => {
                            log::info!(
                                "Received Handshake from a server ({}), is_host: {}",
                                connection_id,
                                is_host
                            );
                            // A hosting client won't send a join packet first, as a server initiates
                            // a connection.
                            if !self.has_sent_join_message {
                                log::info!("Sending a JoinRoom message");
                                self.has_sent_join_message = true;
                                send_message_reliable(
                                    &mut system_data.transport,
                                    net_connection_model,
                                    ClientMessagePayload::JoinRoom {
                                        sent_at: SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .expect("Expected a duration unix timestamp"),
                                        nickname: self.nickname.clone(),
                                    },
                                );
                            }

                            system_data.multiplayer_room_state.connection_status =
                                ConnectionStatus::Connected(connection_id);
                            system_data.multiplayer_room_state.is_host = is_host;
                        }
                        ServerMessagePayload::UpdateRoomPlayers(players) => {
                            update_room_players(&mut system_data.multiplayer_game_state, players);
                        }
                        ServerMessagePayload::StartGame(net_ids_and_players) => {
                            system_data.last_acknowledged_update.frame_number = 0;
                            system_data.last_acknowledged_update.id = 0;

                            let (entity_net_ids, players): (
                                Vec<NetIdentifier>,
                                Vec<MultiplayerRoomPlayer>,
                            ) = net_ids_and_players.into_iter().unzip();

                            if let Some(_) =
                                system_data.multiplayer_game_state.read_updated_players()
                            {
                                update_room_players(
                                    &mut system_data.multiplayer_game_state,
                                    players,
                                );
                            }

                            let connection_id = system_data
                                .multiplayer_room_state
                                .connection_status
                                .connection_id()
                                .expect(
                                    "Expected to be connected when receiving StartGame message",
                                );

                            let mut found_ourselves = false;
                            // Looking for an entity_net_id of a client's player
                            // and storing it in the MultiplayerRoomState.
                            for (i, player) in system_data
                                .multiplayer_game_state
                                .update_players()
                                .iter_mut()
                                .enumerate()
                            {
                                player.entity_net_id = entity_net_ids[i];
                                if connection_id == player.connection_id {
                                    log::info!(
                                        "Starting a new game as a player with net id {}",
                                        player.entity_net_id
                                    );
                                    found_ourselves = true;
                                    system_data.multiplayer_room_state.player_net_id =
                                        player.entity_net_id;
                                }
                            }
                            if !found_ourselves {
                                panic!(
                                    "Couldn't found a player with connection id {}",
                                    connection_id
                                );
                            }
                            system_data.multiplayer_game_state.is_playing = true;
                            system_data.new_game_engine_sate.0 = GameEngineState::Playing;
                        }
                        ServerMessagePayload::UpdateWorld { id, mut updates } => {
                            send_message_unreliable(
                                &mut system_data.transport,
                                net_connection_model,
                                ClientMessagePayload::AcknowledgeWorldUpdate(id),
                            );

                            if system_data.last_acknowledged_update.id < id {
                                updates.sort_by(|a, b| a.frame_number.cmp(&b.frame_number));

                                system_data.last_acknowledged_update.id = id;
                                system_data.last_acknowledged_update.frame_number =
                                    system_data.last_acknowledged_update.frame_number.max(
                                        updates
                                            .last()
                                            .map(|update| update.frame_number)
                                            .unwrap_or(0),
                                    );

                                let frame_to_reserve = system_data
                                    .last_acknowledged_update
                                    .frame_number
                                    .max(system_data.game_time_service.game_frame_number());
                                system_data.framed_updates.reserve_updates(frame_to_reserve);
                                system_data.spawn_actions.reserve_updates(frame_to_reserve);

                                apply_world_updates(
                                    vec![system_data.multiplayer_room_state.player_net_id],
                                    &mut system_data.framed_updates,
                                    &mut system_data.spawn_actions,
                                    updates,
                                );
                            }
                        }
                        ServerMessagePayload::DiscardWalkActions(discarded_actions) => {
                            discard_walk_actions(
                                &mut system_data.player_actions_updates,
                                discarded_actions,
                            );
                        }
                        ServerMessagePayload::ReportPlayersNetStatus { id, players } => {
                            if system_data.multiplayer_game_state.players_status_id < id {
                                system_data.multiplayer_game_state.players_status_id = id;
                                system_data.players_net_status.frame_received =
                                    system_data.game_time_service.game_frame_number();
                                system_data.players_net_status.players = players;
                            }
                        }
                        ServerMessagePayload::PauseWaitingForPlayers { id, players } => {
                            if system_data
                                .multiplayer_game_state
                                .waiting_for_players_pause_id
                                < id
                            {
                                // We don't always want set `waiting_for_players` to true, as we may need
                                // to catch up with the server if we're lagging too. See below.
                                system_data
                                    .multiplayer_game_state
                                    .waiting_for_players_pause_id = id;
                                system_data.multiplayer_game_state.lagging_players = players;
                            }
                        }
                        ServerMessagePayload::UnpauseWaitingForPlayers(id) => {
                            if system_data
                                .multiplayer_game_state
                                .waiting_for_players_pause_id
                                <= id
                            {
                                system_data.multiplayer_game_state.waiting_for_players = false;
                                system_data
                                    .multiplayer_game_state
                                    .waiting_for_players_pause_id = id;
                                system_data.multiplayer_game_state.lagging_players.clear();
                            }
                        }
                        ServerMessagePayload::Disconnect(disconnect_reason) => {
                            if !system_data
                                .multiplayer_room_state
                                .connection_status
                                .is_not_connected()
                            {
                                log::info!(
                                    "Received a Disconnect message: {:?}",
                                    disconnect_reason
                                );
                                let is_shutting_down_by_host = matches!(
                                    system_data.multiplayer_room_state.connection_status,
                                    ConnectionStatus::Disconnecting
                                );

                                if !is_shutting_down_by_host {
                                    system_data.multiplayer_room_state.connection_status =
                                        ConnectionStatus::Disconnected(disconnect_reason);
                                }
                            }
                        }
                    }
                }

                NetEvent::Disconnected => {
                    let mut is_not_connected = system_data
                        .multiplayer_room_state
                        .connection_status
                        .is_not_connected();
                    if let ConnectionStatus::Connecting(started_at) =
                        system_data.multiplayer_room_state.connection_status
                    {
                        // A really ugly way to ignore Disconnected events for previous connections.
                        is_not_connected = Instant::now() - started_at < Duration::from_secs(1);
                    }
                    if !is_not_connected {
                        system_data.multiplayer_room_state.connection_status =
                            ConnectionStatus::ConnectionFailed(None);
                    }
                }
                _ => {}
            }
        }

        if system_data.game_time_service.engine_time().frame_number() - self.last_heartbeat_frame
            > HEARTBEAT_FRAME_INTERVAL
            && !net_connection_model.disconnected
        {
            self.last_heartbeat_frame = system_data.game_time_service.engine_time().frame_number();
            send_message_reliable(
                &mut system_data.transport,
                net_connection_model,
                ClientMessagePayload::Heartbeat,
            );
        }

        // Until the server authorizes to unpause we need to use a chance to catch up with it,
        // even if it's not us lagging.
        if !system_data
            .multiplayer_game_state
            .lagging_players
            .is_empty()
        {
            let server_frame = system_data
                .framed_updates
                .updates
                .back()
                .map_or(0, |update| update.frame_number);

            system_data.multiplayer_game_state.waiting_for_players =
                system_data.game_time_service.game_frame_number() + INTERPOLATION_FRAME_DELAY
                    >= server_frame;
        }

        if *system_data.game_engine_state == GameEngineState::Playing
            && system_data.multiplayer_game_state.is_playing
        {
            // We always skip first INTERPOLATION_FRAME_DELAY frames on game start.
            match system_data
                .game_time_service
                .game_frame_number_absolute()
                .cmp(&INTERPOLATION_FRAME_DELAY)
            {
                Ordering::Less => {
                    system_data.multiplayer_game_state.waiting_network = true;
                    return;
                }
                Ordering::Equal => {
                    system_data.multiplayer_game_state.waiting_network = false;
                }
                _ => {}
            }

            // Wait if we a server is lagging behind for PAUSE_FRAME_THRESHOLD frames.
            let frames_ahead = system_data
                .game_time_service
                .game_frame_number()
                .saturating_sub(
                    system_data
                        .last_acknowledged_update
                        .frame_number
                        .saturating_sub(INTERPOLATION_FRAME_DELAY),
                );
            log::trace!("Frames ahead: {}", frames_ahead);
            if system_data.multiplayer_game_state.waiting_network {
                system_data.multiplayer_game_state.waiting_network = frames_ahead != 0;
            } else if frames_ahead > PAUSE_FRAME_THRESHOLD {
                system_data.multiplayer_game_state.waiting_network = true;
            }

            if system_data.multiplayer_game_state.waiting_network
                || system_data.multiplayer_game_state.waiting_for_players
            {
                log::debug!(
                    "Waiting for server. Frames ahead: {}. Current frame: {}. Last ServerWorldUpdate frame: {}. Estimated server frame: {}",
                    frames_ahead,
                    system_data.game_time_service.game_frame_number(),
                    system_data.last_acknowledged_update.frame_number,
                    net_connection_model.ping_pong_data.last_stored_game_frame(),
                );
            }
        }
    }
}

fn server_connection<'a>(
    net_connection_models: &'a mut WriteStorage<NetConnectionModel>,
) -> &'a mut NetConnectionModel {
    net_connection_models
        .join()
        .next()
        .expect("Expected a server connection")
}

// Expects incoming_updates to be sorted (lowest frame first).
fn apply_world_updates(
    controlled_players: Vec<NetIdentifier>,
    framed_updates: &mut FramedUpdates<ReceivedServerWorldUpdate>,
    spawn_actions: &mut FramedUpdates<SpawnActions>,
    mut incoming_updates: Vec<ServerWorldUpdate>,
) {
    if incoming_updates.is_empty() {
        return;
    }

    let first_incoming_frame_number = incoming_updates
        .first()
        .unwrap()
        .frame_number
        .saturating_sub(INTERPOLATION_FRAME_DELAY);
    let first_available_frame_number = framed_updates.updates.front().unwrap().frame_number;
    assert!(
        first_incoming_frame_number >= first_available_frame_number,
        "Tried to apply a too old ServerUpdate (frame {}), when the first available frame is {}",
        first_incoming_frame_number,
        first_available_frame_number,
    );

    let controlled_player_updates =
        collect_controlled_player_updates(&controlled_players, &mut incoming_updates);

    let (controlled_start_frame_number, others_start_frame_number) = incoming_updates
        .first()
        .map(|update| {
            (
                update
                    .frame_number
                    .saturating_sub(INTERPOLATION_FRAME_DELAY),
                update.frame_number,
            )
        })
        .unwrap();

    spawn_actions.oldest_updated_frame = others_start_frame_number;
    for (spawn_actions, server_update) in spawn_actions
        .updates_iter_mut(others_start_frame_number)
        .zip(incoming_updates.iter())
    {
        spawn_actions.spawn_actions = server_update.spawn_actions.clone()
    }

    framed_updates.oldest_updated_frame = controlled_start_frame_number;
    let mut controlled_player_updates_iter = controlled_player_updates.into_iter();
    let mut incoming_updates_iter = incoming_updates.into_iter();

    for frame_updates in framed_updates.updates_iter_mut(controlled_start_frame_number) {
        if let Some(controlled_player_updates) = controlled_player_updates_iter.next() {
            frame_updates.controlled_player_updates = controlled_player_updates;
        }
        if frame_updates.frame_number >= others_start_frame_number {
            let server_update = incoming_updates_iter.next();
            if server_update.is_none() {
                return;
            }
            frame_updates.apply_server_update(server_update.unwrap());
        }
    }
}

fn collect_controlled_player_updates(
    controlled_players: &[NetIdentifier],
    incoming_updates: &mut Vec<ServerWorldUpdate>,
) -> Vec<ReceivedPlayerUpdate> {
    incoming_updates
        .iter_mut()
        .skip_while(|update| {
            // Skips the first 10 frames, as there shouldn't be any player updates on game start.
            update.frame_number < INTERPOLATION_FRAME_DELAY
        })
        .map(|update| {
            let mut controlled_player_update = ReceivedPlayerUpdate::default();

            let walk_action_pos = update
                .player_walk_actions_updates
                .iter()
                .position(|action| controlled_players.contains(&action.entity_net_id));
            if let Some(walk_action_pos) = walk_action_pos {
                let walk_action = update.player_walk_actions_updates.remove(walk_action_pos);
                controlled_player_update
                    .player_walk_actions_updates
                    .push(walk_action);
            }

            let cast_action_pos = update
                .player_cast_actions_updates
                .iter()
                .position(|action| controlled_players.contains(&action.entity_net_id));
            if let Some(cast_action_pos) = cast_action_pos {
                let cast_action = update.player_cast_actions_updates.remove(cast_action_pos);
                controlled_player_update
                    .player_cast_actions_updates
                    .push(cast_action);
            }

            let look_action_pos = update
                .player_look_actions_updates
                .iter()
                .position(|action| controlled_players.contains(&action.entity_net_id));
            if let Some(look_action_pos) = look_action_pos {
                // We just remove a look action here, as we are not interested in replaying it.
                update.player_look_actions_updates.remove(look_action_pos);
            }

            controlled_player_update
        })
        .collect()
}

fn discard_walk_actions(
    client_player_updates: &mut FramedUpdates<PlayerActionUpdates>,
    mut discarded_updates: Vec<NetIdentifier>,
) {
    let mut oldest_updated_frame = client_player_updates.oldest_updated_frame;
    for update in client_player_updates.updates.iter_mut().rev() {
        let update_frame_number = update.frame_number;
        update.walk_action_updates.retain(|net_update| {
            if let Some(i) = discarded_updates
                .iter()
                .position(|discarded_update| *discarded_update == net_update.data.client_action_id)
            {
                discarded_updates.remove(i);
                oldest_updated_frame = update_frame_number;
                false
            } else {
                true
            }
        });

        if discarded_updates.is_empty() {
            break;
        }
    }
    client_player_updates.oldest_updated_frame = oldest_updated_frame;
}
