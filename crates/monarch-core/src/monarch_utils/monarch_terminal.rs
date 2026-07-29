use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tracing::{error, info};

pub type TerminalDone = Arc<Mutex<Option<futures::channel::oneshot::Sender<()>>>>;

type TerminalHandler = Box<
    dyn FnMut(String, HashMap<String, String>, Option<String>, TerminalDone) + Send,
>;

static TERMINAL_HANDLER: LazyLock<Mutex<Option<TerminalHandler>>> =
    LazyLock::new(|| Mutex::new(None));

/// Register a GUI (or other) handler that opens an interactive terminal.
///
/// Called once during app startup so `monarch_core` does not depend on the GUI crate.
pub fn register_terminal_handler(handler: TerminalHandler) {
    *TERMINAL_HANDLER.lock().unwrap() = Some(handler);
}

/// Spawn a new terminal window with the given command and environment variables.
pub fn spawn_terminal(
    command: String,
    env: HashMap<String, String>,
    workdir: Option<String>,
) -> futures::channel::oneshot::Receiver<()> {
    let (tx, rx) = futures::channel::oneshot::channel();
    let done = Arc::new(Mutex::new(Some(tx)));

    info!("Calling command: {}", command);
    if let Some(handler) = TERMINAL_HANDLER.lock().unwrap().as_mut() {
        handler(command, env, workdir, done);
    } else {
        error!(
            "monarch_terminal::spawn_terminal() no terminal handler registered! GUI might not be initialized."
        );
    }

    rx
}
