/*
 * File for abstracting working with the protobuf API wiht monarch-launcher.com
*/

use anyhow::Result;
use prost::Message;

use crate::{monarch_games::{monarchgame::{MonarchGame, StoreInfo}, stores::StoreEnum}, proto::generated::{GetStoresResponse, SearchGamesResponse, SearchSuggestionsResponse}};

pub struct SearchSuggestion {
    pub name: String,
    pub id: String,
}

pub struct SearchParams {
    pub name: Option<String>,
    pub id: Option<String>,
    pub store: Option<String>,
    pub store_id: Option<String>,
    pub limit: u32,
}

/// Search for games on monarch-launcher.com
pub async fn search_games(params: SearchParams) -> Result<Vec<MonarchGame>> {
    let url = format!("{}/api/games/search", std::env!("MONARCH_URL"));
    let client = reqwest::Client::new();

    let limit = params.limit.to_string();
    let mut web_params: Vec<(&str, &str)> = vec![("limit", &limit)];

    if params.name.is_some() {
        web_params.push(("query", &params.name.as_ref().unwrap().as_str()));
    }

    if params.id.is_some() {
        web_params.push(("id", &params.id.as_ref().unwrap().as_str()));
    }
    
    if params.store.is_some() {
        web_params.push(("store", &params.store.as_ref().unwrap().as_str()));
    }

    if params.store_id.is_some() {
        web_params.push(("storeId", &params.store_id.as_ref().unwrap().as_str()));
    }

    let bytes = client.get(url)
        .query(&web_params)
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
        .map(|s| StoreInfo { name: StoreEnum::from(s.name), store_id: s.store_id, store_url: s.store_url })
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