use iced::{
    application::Application,
    widget::{container, text},
    Element,
    Length::Fill,
};

use crate::gui::components::header::{self, Header};

pub mod components;
pub mod pages;

#[derive(Clone)]
enum AppMessage {
    HeaderMessage(header::Message),
}

#[derive(Default)]
pub struct App {
    header: Header,
}

impl App {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn run(&self) {
        iced::run(App::update, App::view).unwrap();
    }
}

impl App {
    fn update(&mut self, _: AppMessage) {}

    fn view(&self) -> Element<'_, AppMessage> {
        container(self.header.view()).into()
    }
}
