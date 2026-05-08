use iced::widget::{column, row};
use iced::{alignment, Element};

use super::gamecard::GameCard;
use crate::gui::components::gamecard::GameCardMessage;
use crate::monarch_games::monarchgame::MonarchGame;

#[derive(Default, Debug, Clone)]
pub struct GameCardContainer {
    pub games: Vec<GameCard>,
}

impl GameCardContainer {
    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }

    pub fn update(&mut self, msg: GameCardMessage) -> iced::Task<GameCardMessage> {
        match msg {
            GameCardMessage::UpdateGames(games) => {
                self.games = games.iter().map(|g| GameCard::new(g.clone())).collect();

                self.games.sort_by(|a, b| a.game.name.cmp(&b.game.name));

                iced::Task::none()
            }
            _ => {
                let mut tasks = Vec::new();
                for game in &mut self.games {
                    tasks.push(game.update(msg.clone()));
                }
                iced::Task::batch(tasks)
            }
        }
    }

    pub fn view(&self, interactive: bool) -> Element<'_, GameCardMessage> {
        iced::widget::responsive(move |size| {
            // Calculate how many games can fit in the available width
            // base_width (240) + spacing (30) + some margin
            let card_width = 240.0 + 20.0;
            let games_per_row = (size.width / card_width).floor().max(1.0) as usize;

            let mut games_column = column![].spacing(20).align_x(alignment::Horizontal::Center);

            for chunk in self.games.chunks(games_per_row) {
                let mut row = row![].spacing(20);
                for game in chunk {
                    row = row.push(game.view(interactive));
                }
                games_column = games_column.push(row);
            }

            games_column.into()
        })
        .into()
    }
}

impl FromIterator<MonarchGame> for GameCardContainer {
    fn from_iter<I: IntoIterator<Item = MonarchGame>>(iter: I) -> Self {
        let mut games: Vec<GameCard> = iter.into_iter().map(|g| GameCard::new(g.clone())).collect();
        games.sort_by(|a, b| a.game.name.cmp(&b.game.name));
        Self { games }
    }
}
