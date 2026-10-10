use std::sync::{Arc, RwLock};

use iced::widget::{button, column, row, scrollable, text, Space};
use iced::{alignment, Color, Element, Length, Task};
use monarch_core::monarch_games::monarchgame::MonarchGame;
use monarch_core::monarch_utils::monarch_game_downloader::MonarchDownloader;
use monarch_core::monarch_utils::monarch_state::MonarchState;

use crate::gui::components::gamecard::actions::{self, action_item, ActionsModal};
use crate::gui::components::gamecard::properties::{self, PropertiesModal};
use crate::gui::{resources, styles};

/// Combined edit modal shown from the details page: an action list plus a
/// launch properties editor (executable, compatibility layer, launch
/// arguments) that opens from the "Launch Properties" action.
#[derive(Debug, Clone)]
pub struct EditModal {
    properties: PropertiesModal,
    actions: ActionsModal,
    /// Whether the launch properties editor is currently shown instead of
    /// the action list.
    show_launch_properties: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    Properties(properties::Message),
    Actions(actions::Message),
    /// Open the launch properties editor from the action list.
    ShowLaunchProperties,
    /// Leave the launch properties editor and return to the action list.
    Back,
}

impl EditModal {
    pub fn new(
        state_handle: Arc<RwLock<MonarchState>>,
        game: Arc<RwLock<MonarchGame>>,
        downloader_handle: Arc<RwLock<MonarchDownloader>>,
    ) -> (Self, Task<Message>) {
        let (properties, properties_task) = PropertiesModal::new(state_handle, game.clone());
        let (actions, actions_task) = ActionsModal::new(game, downloader_handle);

        (
            Self {
                properties,
                actions,
                show_launch_properties: false,
            },
            Task::batch(vec![
                properties_task.map(Message::Properties),
                actions_task.map(Message::Actions),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Properties(p) => self.properties.update(p).map(Message::Properties),
            Message::Actions(a) => self.actions.update(a).map(Message::Actions),
            Message::ShowLaunchProperties => {
                self.show_launch_properties = true;
                Task::none()
            }
            Message::Back => {
                self.show_launch_properties = false;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // The content can get taller than the window, so make the inner area
        // scrollable in both views.
        let content = if self.show_launch_properties {
            scrollable(
                column![
                    self.properties.fields().map(Message::Properties),
                    Space::new().height(Length::Fixed(20.0)),
                    row![
                        button(text("Save"))
                            .on_press(Message::Properties(properties::Message::Save))
                            .padding(10)
                            .style(styles::button::primary),
                        Space::new().width(Length::Fixed(10.0)),
                        button(text("Back"))
                            .on_press(if self.properties.showing_compat_opts() {
                                // On the Proton/Wine options page, Back
                                // returns to the launch options fields.
                                Message::Properties(properties::Message::CompatOptsBack)
                            } else {
                                Message::Back
                            })
                            .padding(10)
                            .style(styles::button::secondary),
                    ]
                    .align_y(alignment::Vertical::Center),
                ]
                .spacing(10),
            )
        } else {
            scrollable(
                column![
                    section_header("Launch options"),
                    action_item(
                        resources::EDIT.clone(),
                        "Launch Options",
                        Some(Message::ShowLaunchProperties),
                    ),
                    Space::new().height(Length::Fixed(8.0)),
                    self.actions.sections().map(Message::Actions),
                ]
                .spacing(10),
            )
        };

        crate::gui::components::modal::Modal::new("Game Properties", content)
            .width(Length::Fixed(800.0))
            .on_close(Message::Properties(properties::Message::Cancel))
            .view()
    }
}

/// Section label matching the style used by the actions sections
/// (Maintenance / Files / Extras).
fn section_header(label: &str) -> Element<'_, Message> {
    text(label)
        .size(13)
        .color(Color::from_rgb8(140, 140, 140))
        .into()
}
