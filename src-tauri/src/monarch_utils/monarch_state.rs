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
    /// For setting known library games.
    /// Should probably only be run when refreshing library.
    pub fn set_library_games(&mut self, games: &[MonarchGame]) {
        self.library_games = games.to_vec()
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
