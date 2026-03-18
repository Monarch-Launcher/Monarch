use crate::gui::{AppMessage, GUI_SENDER};
use std::collections::HashMap;
use tracing::{error, info};

/// Spawn a new terminal window with the given command and environment variables.
pub fn spawn_terminal(
    command: String,
    env: HashMap<String, String>,
    workdir: Option<String>,
) -> futures::channel::oneshot::Receiver<()> {
    let (tx, rx) = futures::channel::oneshot::channel();

    info!("Calling command: {}", command);
    if let Some(sender) = GUI_SENDER.lock().unwrap().as_mut() {
        if let Err(e) = sender.unbounded_send(AppMessage::OpenTerminalRaw(
            command,
            env,
            workdir,
            std::sync::Arc::new(std::sync::Mutex::new(Some(tx))),
        )) {
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

    rx
}
