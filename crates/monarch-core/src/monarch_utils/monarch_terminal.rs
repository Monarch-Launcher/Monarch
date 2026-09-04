use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tracing::{error, info, warn};

pub type TerminalDone = Arc<Mutex<Option<futures::channel::oneshot::Sender<()>>>>;

type TerminalHandler =
    Box<dyn FnMut(String, HashMap<String, String>, Option<String>, TerminalDone) + Send>;

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

/// Returns the call command to the system shell
pub fn get_system_shell() -> String {
    let mut shell: String = match std::env::var("SHELL") {
        Ok(sh) => sh,
        Err(e) => {
            warn!("TermInstance::new() Failed to get $SHELL var! | Err: {e}");
            "".to_string()
        }
    };

    // Just try your best if $SHELL variable isn't set
    if shell.is_empty() {
        #[cfg(target_os = "windows")]
        {
            info!("No $SHELL was set, defaulting to powershell.exe");
            shell = "powershell.exe".to_string()
        }

        #[cfg(target_os = "linux")]
        {
            info!("No $SHELL was set, defaulting to /bin/sh");
            shell = "/bin/sh".to_string()
        }
    }

    shell
}

/// Quote an argument for a shell / terminal command string.
pub fn quote_arg(arg: &str) -> String {
    if arg.chars().any(|c| c.is_whitespace() || c == '\'') {
        format!("'{}'", arg.replace('\'', "'\\''"))
    } else {
        arg.to_string()
    }
}
