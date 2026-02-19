use iced::window::Id;
use std::collections::HashMap;

use crate::gui::AppMessage;

pub struct TermInstance {
    title: String,
    id: Id,

    command: String,
    env: HashMap<String, String>,
    term: iced_term::Terminal,
}

impl TermInstance {
    pub fn new(id: Id, command: String, env: HashMap<String, String>) -> Self {
        let shell = std::env::var("SHELL").unwrap();
        let term_settings = iced_term::settings::Settings {
            backend: iced_term::settings::BackendSettings {
                program: shell.to_string(),
                args: vec!["-c".to_string(), command.clone()],
                env: env.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        let term = iced_term::Terminal::new(id.to_string().parse::<u64>().unwrap(), term_settings)
            .unwrap();

        Self {
            title: "Monarch Terminal".to_string(),
            id,
            command,
            env,
            term,
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
