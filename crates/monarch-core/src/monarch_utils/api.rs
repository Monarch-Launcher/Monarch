/*
 * File for abstracting working with the protobuf API wiht monarch-launcher.com
*/

use anyhow::{Context, Result, bail};
use prost::Message;
use bytes::Bytes;

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

    let bytes = perform_search_query("/api/games/search", &web_params)
        .await
        .with_context(|| "api::search_games() -> ")?;

    let search_games_response: SearchGamesResponse = SearchGamesResponse::decode(bytes)?;

    let games: Vec<MonarchGame> = search_games_response.games
        .into_iter()
        .map(|g| g.into())
        .collect::<Vec<MonarchGame>>();

    Ok(games)
}

/// Query for search suggestions/autocomplete from monarch-launcher.com
pub async fn search_suggestion(query: &str) -> Result<Vec<SearchSuggestion>> {
    let bytes = perform_search_query("/api/games/suggestions", &[("query", query)])
        .await
        .with_context(|| "api::search_games() -> ")?;
    
    let search_suggestions_response: SearchSuggestionsResponse = SearchSuggestionsResponse::decode(bytes)?;

    let suggestions: Vec<SearchSuggestion> = search_suggestions_response.suggestions
        .into_iter()
        .map(|s| SearchSuggestion { name: s.name, id: s.id })
        .collect::<Vec<SearchSuggestion>>();

    Ok(suggestions)
}

/// Get the available storefronts the game is available at from 
/// monarch-launcher.com
pub async fn get_stores(id: &str) -> Result<Vec<StoreInfo>> {
    let bytes = perform_search_query("/api/games/stores", &[("id", id)])
        .await
        .with_context(|| "api::get_stores() -> ")?;

    let get_stores_response: GetStoresResponse = GetStoresResponse::decode(bytes)?;

    let stores: Vec<StoreInfo> = get_stores_response.stores
        .into_iter()
        .map(|s| StoreInfo { name: StoreEnum::from(s.name), store_id: s.store_id, store_url: s.store_url })
        .collect::<Vec<StoreInfo>>();

    Ok(stores)
}

/// Get the metadata for a game game from monarch-launcher.com
pub async fn get_metadata(id: &str) -> Result<()> {
    let _bytes = perform_search_query("/api/games/metadata", &[("id", id)])
        .await
        .with_context(|| "api::get_metadata -> ")?;

    Ok(())
}

async fn perform_search_query(path: &str, query: &[(&str, &str)]) -> Result<Bytes> {
    let url = format!("{}{}", std::env!("MONARCH_URL"), path);
    let client = reqwest::Client::new();

    let response = client.get(&url)
        .query(query)
        .send()
        .await
        .with_context(|| "api::search_suggestion() -> ")?;

    if !response.status().is_success() {
        bail!("api::perform_search_query() {url} did not return 2XX! (Real response was HTTP {}: )", response.status().as_str());
    }

    response.bytes()
        .await
        .with_context(|| "api::search_suggestion() -> ")
}