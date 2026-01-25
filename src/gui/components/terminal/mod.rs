use std::collections::HashMap;

use iced::{window::Settings, Task};
use iced_term;
use tracing::info;

use crate::gui::pages::game_details;

pub struct TermInstance {
    title: String,

    command: String,
    env: HashMap<String, String>,
    term: iced_term::Terminal,
}

impl TermInstance {
    pub fn new(command: String, env: HashMap<String, String>) -> Self {
        let shell = std::env::var("SHELL").unwrap();
        let term_settings = iced_term::settings::Settings {
            backend: iced_term::settings::BackendSettings {
                program: shell.to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let term = iced_term::Terminal::new(0, term_settings).unwrap();

        Self {
            title: "Monarch Terminal".to_string(),
            command,
            env,
            term,
        }
    }

    /// Open a new terminal instance.
    pub fn open_terminal(&self) -> Task<game_details::Message> {
        info!("Opening terminal");
        let terminal_window_settings: Settings = Settings {
            visible: true,
            decorations: false,

            ..Default::default()
        };
        let (id, task) = iced::window::open(terminal_window_settings);
        info!("Opened terminal with id: {id}");
        task.map(|id| game_details::Message::OpenTerminal(id))
    }

    pub fn close_terminal(&self) {}

    pub fn run_command(&self) {}
}
