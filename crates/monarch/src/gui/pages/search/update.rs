use tracing::error;

use crate::{
    gui::{
        components::gamecard,
        pages::search::{Message, SearchPage},
    },
    monarch_games::{
        self, games::SearchResult, monarchgame::MonarchWebApiGame, stores::SearchFilter,
    },
};

impl SearchPage {
    pub fn perform_search(&mut self, search_filter: SearchFilter) -> iced::Task<Message> {
        self.is_searching = true;
        self.dot_count = 3;
        self.tick_counter = 0;
        let search_term = self.search_value.clone();
        iced::Task::perform(
            async move { monarch_games::commands::search_games(search_term, search_filter).await },
            Message::UpdateGames,
        )
    }

    pub fn update_games(&mut self, games: Vec<MonarchWebApiGame>) -> iced::Task<Message> {
        self.is_searching = false;

        let processed_games: Vec<MonarchWebApiGame> = games
            .iter()
            .cloned()
            .map(|mut game| {
                game.thumbnail_path = "".to_string();
                game
            })
            .collect();

        // Trigger download tasks
        let download_tasks = iced::Task::batch(games.iter().cloned().map(|game| {
            iced::Task::perform(
                async move {
                    if let Err(e) =
                        monarch_games::commands::download_thumbnail(&game.into_monarchgame()).await
                    {
                        error!(
                            "Failed to download thumbnail for game {} ({}): {}",
                            game.id, game.cover_url, e
                        );
                    }
                    game
                },
                Message::GameImgLoaded,
            )
        }));

        // Update browser games
        let _ = self.browser.update(gamecard::GameCardMessage::UpdateGames(
            processed_games
                .iter()
                .map(|g| g.into_monarchgame())
                .collect(),
        ));

        download_tasks
    }

    pub fn game_img_loaded(&mut self, game: MonarchWebApiGame) -> iced::Task<Message> {
        if let Some(card) = self
            .browser
            .games
            .games
            .iter_mut()
            .find(|c| c.game.id == game.id)
        {
            card.game.thumbnail_path = game.thumbnail_path.clone();
        }
        iced::Task::none()
    }

    pub fn tick(&mut self) -> iced::Task<Message> {
        if self.is_searching {
            self.tick_counter = self.tick_counter.wrapping_add(1);
            if self.tick_counter % 60 == 0 {
                self.dot_count = (self.dot_count % 3) + 1;
            }
        }
        self.browser
            .update(gamecard::GameCardMessage::Tick)
            .map(Message::GameCard)
    }
}
