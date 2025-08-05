use crate::monarch_games::monarchgame::MonarchGame;
use anyhow::{bail, Result};
use once_cell::sync::Lazy;

/// Global state of monarch backend (excluding settings for now)
/// TODO: Change to some Atomic structure in the future to avoid using shared refrences to mut static
pub static mut MONARCH_STATE: Lazy<MonarchState> = Lazy::<MonarchState>::new(MonarchState::default);

/// A struct for storing some sort of global state that
/// the backend can access to recieve relevant info.
#[derive(Default, Debug)]
pub struct MonarchState {
    library_games: Vec<MonarchGame>,
}

impl MonarchState {
    /// Returns what the backend thinks is the users library.
    pub fn get_library_games(&self) -> Vec<MonarchGame> {
        self.library_games.clone()
    }

    /// For setting known library games.
    /// Should probably only be run when refreshing library.
    pub fn set_library_games(&mut self, games: &[MonarchGame]) {
        if self.library_games.is_empty() {
            self.library_games = games.to_vec();
            return;
        }

        // Remove games that are no longer in library
        let mut new_games = self
            .library_games
            .iter()
            .zip(games.iter())
            .filter(|&(self_g, g)| self_g.id == g.id)
            .map(|(self_g, _)| self_g)
            .cloned()
            .collect::<Vec<MonarchGame>>();

        // Append new games
        for game in games {
            let mut is_dupe: bool = false;
            for self_game in new_games.iter() {
                if game.id == self_game.id {
                    is_dupe = true;
                    break;
                }
            }
            if !is_dupe {
                new_games.push(game.clone());
            }
        }

        self.library_games = new_games;
    }

    /// Update a game.
    /// Useful when updating game properties and want to let
    /// the backend state know of it.
    pub fn update_game(&mut self, game: &MonarchGame) -> Result<()> {
        for (i, self_game) in self.library_games.iter_mut().enumerate() {
            if self_game.id == game.id {
                self.library_games[i] = game.clone();
                return Ok(());
            }
        }
        bail!("monarch_state::update_game() No matching game found!")
    }

    /// Returns a library game with matching id.
    /// Useful when you might need some properties of a game.
    pub fn get_game(&self, id: &str) -> Option<MonarchGame> {
        for game in self.library_games.iter() {
            if game.id == id {
                return Some(game.clone());
            }
        }
        None
    }
}
