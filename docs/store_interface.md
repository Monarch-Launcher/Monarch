# Store Interface (WIP)

## Why
To make it easier to integrate more stores quicker and keep the codebase clean, there's a common interface for Monarch to communicate what it wants
to do to the different stores. 

## How
The store interface is a trait that is implemented for each store. It contains the following methods:

```rust
pub trait StoreInterface {
    fn search_game(&self, name: &str) -> Result<Vec<dyn GameType>>;
    fn download_game(&self, name: &str, platform_id: &str) -> Result<Vec<dyn GameType>>;
    fn uninstall_game(&self, platform_id: &str) -> Result<()>;
    fn update_game(&self, game: &MonarchGame) -> Result<()>;
}
```

To accompany the `StoreInterface` there's also a `GameType` trait for which lowers the requirements for what information needs to be gathered/stored 
about games. Instead the following methods need to be implememted:

```rust
pub trait GameType {
    fn get_name(&self) -> String;
    fn get_platform(&self) -> String;
    fn get_platform_id(&self) -> String;
    fn get_description(&self) -> String;
    fn get_price(&self) -> String;
}
```
As well as the From/Into Rust trait between the stores `GameType` and `MonarchGame`.

## Current status
The store interface is currently in the planning phase and is being developed along side the 
integration of Legendary as well as the improvements to the Steam integration.

## Future plans
The store interface is planned to be implemented into Monarch for Steam, Epic Games/Legendary and GOG 
at the moment. It is also planned to add some sort of external store interface that allows Monarch to 
communicate with other stores via 3rd party implementations. This enables faster integration since 
anyone can contribute with their time and knowledge of other stores/platforms.