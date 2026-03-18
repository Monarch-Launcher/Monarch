pub mod game_details;
pub mod home;
pub mod library;
pub mod search;
pub mod settings;
pub mod store_details;

#[derive(Clone, Debug)]
pub enum Message {
    Home(home::Message),
    Library(library::Message),
    Search(search::Message),
    Settings(settings::Message),
    GameDetails(game_details::Message),
    StoreDetails(store_details::Message),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageTab {
    #[default]
    Home,
    Library,
    Search,
    Settings,
    GameDetails,
    StoreDetails,
}
