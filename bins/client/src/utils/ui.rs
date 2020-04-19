use gv_core::net::server_message::DisconnectReason;

pub fn disconnect_reason_title(disconnect_reason: DisconnectReason) -> String {
    match disconnect_reason {
        DisconnectReason::Uninitialized => "The server is not initialized yet".to_owned(),
        DisconnectReason::GameIsStarted => "The server has already started the game".to_owned(),
        DisconnectReason::RoomIsFull => "The room is full".to_owned(),
        DisconnectReason::Kick => "You've been kicked".to_owned(),
        DisconnectReason::Closed => "The host has closed the server".to_owned(),
        DisconnectReason::ServerCrashed(exit_code) => {
            format!("The server unexpectedly closed: {}", exit_code)
        }
    }
}
