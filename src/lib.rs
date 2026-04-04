//! FS25 AutoDrive Editor Library.
//! Core-Funktionalitaet als Library exportiert fuer Tests und Wiederverwendung.

pub mod render;
pub mod ui;

pub use fs25_auto_drive_engine::app;
pub use fs25_auto_drive_engine::core;
pub use fs25_auto_drive_engine::shared;
pub use fs25_auto_drive_engine::xml;

pub use app::{AppCommand, AppController, AppIntent, AppState};
pub use xml::{parse_autodrive_config, write_autodrive_config};
