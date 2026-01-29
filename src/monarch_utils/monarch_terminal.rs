use crate::gui::{AppMessage, GUI_SENDER};
use std::collections::HashMap;
use tracing::{error, info};

/// Spawn a new terminal window with the given command and environment variables.
pub fn spawn_terminal(command: String, env: HashMap<String, String>) {
    info!("Calling command: {}", command);
    if let Some(sender) = GUI_SENDER.lock().unwrap().as_mut() {
        if let Err(e) = sender.unbounded_send(AppMessage::OpenTerminalRaw(command, env)) {
            error!(
                "monarch_terminal::spawn_terminal() Failed to send message to GUI | Err: {}",
                e
            );
        }
    } else {
        error!(
            "monarch_terminal::spawn_terminal() GUI_SENDER is None! GUI might not be initialized."
        );
    }
}
