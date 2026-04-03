//! FS25 AutoDrive Editor Library.
//! Core-Funktionalitaet als Library exportiert fuer Tests und Wiederverwendung.

pub mod app;
pub mod core;
pub mod render;
pub mod shared;
pub mod ui;
pub mod xml;

pub use app::{AppCommand, AppController, AppIntent, AppState};
pub use xml::{parse_autodrive_config, write_autodrive_config};
