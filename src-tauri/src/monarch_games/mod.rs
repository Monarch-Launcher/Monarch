pub mod monarchgame;
pub mod commands;

#[cfg(target_os="windows")]
pub mod windows;

#[cfg(not(target_os="windows"))]
pub mod unix;