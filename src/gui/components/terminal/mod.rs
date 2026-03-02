use anyhow::{bail, Result};
use iced::window::Id;
use std::{collections::HashMap, path::PathBuf};
use tracing::error;

use crate::gui::{show_error, AppMessage};

pub struct TermInstance {
    _title: String,
    id: Id,

    _command: String,
    _env: HashMap<String, String>,
    _workdir: Option<String>,
    term: iced_term::Terminal,
    completion_tx: Option<futures::channel::oneshot::Sender<()>>,
}

impl TermInstance {
    pub fn new(
        id: Id,
        command: String,
        env: HashMap<String, String>,
        workdir: Option<String>,
        completion_tx: Option<futures::channel::oneshot::Sender<()>>,
    ) -> Result<Self> {
        let shell = match std::env::var("SHELL") {
            Ok(sh) => sh,
            Err(e) => {
                error!("TermInstance::new() Failed to get $SHELL var! | Err: {e}");
                show_error("Failed to open terminal! Could not detect OS shell.");
                bail!("Failed to create terminal!")
            }
        };

        let workdir_path: Option<PathBuf> = if let Some(wd) = &workdir {
            Some(PathBuf::from(wd))
        } else {
            None
        };

        let term_settings = iced_term::settings::Settings {
            backend: iced_term::settings::BackendSettings {
                program: shell.to_string(),
                args: vec!["-c".to_string(), command.clone()],
                env: env.clone(),
                working_directory: workdir_path,
                ..Default::default()
            },
            ..Default::default()
        };

        match iced_term::Terminal::new(id.to_string().parse::<u64>().unwrap(), term_settings) {
            Ok(term) => Ok(Self {
                _title: "Monarch Terminal".to_string(),
                id,
                _command: command,
                _env: env,
                _workdir: workdir,
                term,
                completion_tx,
            }),
            Err(e) => {
                error!("TermInstance::new() iced_term::Terminal::new() failed to create new Terminal instance! | Err: {e}");
                show_error("Failed to open terminal! Could not detect OS shell.");
                bail!("Failed to create terminal!");
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, iced_term::Event> {
        iced_term::TerminalView::show(&self.term)
    }

    pub fn subscription(&self) -> iced::Subscription<iced_term::Event> {
        self.term.subscription()
    }

    pub fn update(&mut self, event: iced_term::Event) -> iced::Task<AppMessage> {
        match event {
            iced_term::Event::BackendCall(id, cmd) => {
                if id == self.term.id {
                    let action = self.term.handle(iced_term::Command::ProxyToBackend(cmd));
                    match action {
                        iced_term::actions::Action::Shutdown => {
                            if let Some(tx) = self.completion_tx.take() {
                                let _ = tx.send(());
                            }
                            let id = self.id;
                            return iced::Task::perform(
                                async move { id },
                                AppMessage::CloseTerminal,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        iced::Task::none()
    }
}
