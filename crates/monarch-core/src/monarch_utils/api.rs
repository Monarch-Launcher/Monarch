/*
 * File for abstracting working with the protobuf API wiht monarch-launcher.com
*/

use anyhow::Result;
use prost::Message;

use crate::{monarch_games::monarchgame::{MonarchGame, StoreInfo}, proto::generated::{GetStoresResponse, SearchGamesResponse, SearchSuggestionsResponse}};

pub struct SearchSuggestion {
    pub name: String,
    pub id: String,
}

/// Search for games on monarch-launcher.com
pub async fn search_games(query: &str, limit: u32) -> Result<Vec<MonarchGame>> {
    let url = format!("{}/api/games/search", std::env!("MONARCH_URL"));
    let client = reqwest::Client::new();

    let bytes = client.get(url)
        .query(&[("query", query), ("limit", &limit.to_string())])
        .send()
        .await?
        .bytes()
        .await?;

    let response: SearchGamesResponse = SearchGamesResponse::decode(bytes)?;

    let games: Vec<MonarchGame> = response.games
        .into_iter()
        .map(|g| g.into())
        .collect::<Vec<MonarchGame>>();

    Ok(games)
}

/// Query for search suggestions/autocomplete from monarch-launcher.com
pub async fn search_suggestion(query: &str) -> Result<Vec<SearchSuggestion>> {
    let url = format!("{}/api/games/suggestions", std::env!("MONARCH_URL"));
    let client = reqwest::Client::new();

    let bytes = client.get(url)
        .query(&[("query", query)])
        .send()
        .await?
        .bytes()
        .await?;

    let response: SearchSuggestionsResponse = SearchSuggestionsResponse::decode(bytes)?;

    let suggestions: Vec<SearchSuggestion> = response.suggestions
        .into_iter()
        .map(|s| SearchSuggestion { name: s.name, id: s.id })
        .collect::<Vec<SearchSuggestion>>();

    Ok(suggestions)
}

/// Get the available storefronts the game is available at from 
/// monarch-launcher.com
pub async fn get_stores(id: &str) -> Result<Vec<StoreInfo>> {
    let url = format!("{}/api/games/stores", std::env!("MONARCH_URL"));
    let client = reqwest::Client::new();

    let bytes = client.get(url)
        .query(&[("id", id)])
        .send()
        .await?
        .bytes()
        .await?;

    let response: GetStoresResponse = GetStoresResponse::decode(bytes)?;

    let stores: Vec<StoreInfo> = response.stores
        .into_iter()
        .map(|s| StoreInfo { name: s.name, store_id: s.store_id, store_url: s.store_url })
        .collect::<Vec<StoreInfo>>();

    Ok(stores)
}

/// Get the metadata for a game game from monarch-launcher.com
pub async fn get_metadata(id: &str) -> Result<()> {
    let url = format!("{}/api/games/metadata", std::env!("MONARCH_URL"));
    let client = reqwest::Client::new();

    let _bytes = client.get(url)
        .query(&[("id", id)])
        .send()
        .await?
        .bytes()
        .await?;

    // TODO: Implement this function when metadata API works.

    Ok(())
}