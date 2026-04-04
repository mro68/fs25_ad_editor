//! Root-Fassade des FS25 AutoDrive Editors.
//! Re-exportiert Engine- und egui-Frontend-Oberflaechen fuer Tests, Benches und bestehende Rust-Call-Sites.

pub use fs25_auto_drive_engine::app;
pub use fs25_auto_drive_engine::core;
pub use fs25_auto_drive_engine::shared;
pub use fs25_auto_drive_engine::xml;
pub use fs25_auto_drive_frontend_egui::render;
pub use fs25_auto_drive_frontend_egui::ui;

pub use app::{AppCommand, AppController, AppIntent, AppState};
pub use xml::{parse_autodrive_config, write_autodrive_config};
