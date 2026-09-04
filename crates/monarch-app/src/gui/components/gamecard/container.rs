use iced::widget::{column, row, text, Space};
use iced::{alignment, Element};

use super::gamecard::GameCard;
use crate::gui::components::gamecard::GameCardMessage;
use monarch_core::monarch_games::games::GameType;
use monarch_core::monarch_games::monarchgame::MonarchGame;

/// Controls which library games are shown. A store/status is only enforced
/// when its corresponding checkbox is enabled; if no option in a category is
/// enabled that category is not filtered (all values are shown).
#[derive(Debug, Clone, Default)]
pub struct LibraryFilter {
    pub steam: bool,
    pub epic: bool,
    pub installed: bool,
    pub uninstalled: bool,
}

impl LibraryFilter {
    /// Whether any filter option is currently enabled.
    pub fn is_active(&self) -> bool {
        self.steam || self.epic || self.installed || self.uninstalled
    }

    /// Whether `game` passes the current filter.
    pub fn matches(&self, game: &MonarchGame) -> bool {
        if self.steam || self.epic {
            let store = game.get_store_name();
            let store_ok =
                (self.steam && (store == "steam" || store == "steamcmd"))
                    || (self.epic && store == "epicgames");
            if !store_ok {
                return false;
            }
        }

        if self.installed || self.uninstalled {
            let status_ok =
                (self.installed && game.is_installed) || (self.uninstalled && !game.is_installed);
            if !status_ok {
                return false;
            }
        }

        true
    }
}

#[derive(Default, Debug, Clone)]
pub struct GameCardContainer {
    pub games: Vec<GameCard>,
    pub filter: LibraryFilter,
}

impl GameCardContainer {
    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }

    /// Insert `game` or replace an existing card with the same id, then keep
    /// the list sorted by name.
    pub fn upsert_game(&mut self, game: MonarchGame) {
        if let Some(card) = self.games.iter_mut().find(|c| c.game.id == game.id) {
            card.update_game(game);
        } else {
            self.games.push(GameCard::new(game));
        }
        self.games.sort_by(|a, b| a.game.name.cmp(&b.game.name));
    }

    /// Drop a card by game id, if present.
    pub fn remove_game(&mut self, game_id: &str) {
        self.games.retain(|card| card.game.id != game_id);
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

            let mut games_column = column![].spacing(10).align_x(alignment::Horizontal::Left);

            // Only keep games matching the active filter.
            let visible: Vec<&GameCard> = self
                .games
                .iter()
                .filter(|card| self.filter.matches(&card.game))
                .collect();

            if visible.is_empty() {
                return games_column
                    .push(
                        text(
                            if self.filter.is_active() {
                                "No games match the current filters"
                            } else {
                                "No games found"
                            },
                        )
                        .size(24)
                        .font(crate::gui::styles::fonts::REGULAR),
                    )
                    .into();
            }

            let (installed_games, uninstalled_games): (Vec<&GameCard>, Vec<&GameCard>) =
                visible.iter().copied().partition(|card| card.game.is_installed);

            if !installed_games.is_empty() {
                games_column = games_column.push(
                    text("Installed")
                        .size(24)
                        .font(crate::gui::styles::fonts::BOLD),
                );

                for chunk in installed_games.chunks(games_per_row) {
                    let mut row = row![].spacing(10);
                    for game in chunk {
                        row = row.push(game.view(interactive));
                    }
                    games_column = games_column.push(row);
                }
            }

            if !uninstalled_games.is_empty() {
                if !installed_games.is_empty() {
                    games_column =
                        games_column.push(Space::new().height(iced::Length::Fixed(40.0)));
                }

                games_column = games_column.push(
                    text("Ready to Install")
                        .size(24)
                        .font(crate::gui::styles::fonts::BOLD),
                );

                for chunk in uninstalled_games.chunks(games_per_row) {
                    let mut row = row![].spacing(10);
                    for game in chunk {
                        row = row.push(game.view(interactive));
                    }
                    games_column = games_column.push(row);
                }
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
        Self {
            games,
            filter: LibraryFilter::default(),
        }
    }
}
