//! Host-neutrale Engine-Crate fuer den FS25 AutoDrive Editor.

/// Application-Layer: Controller, State, Events und Use-Cases.
pub mod app;
/// Core-Domaenentypen: Nodes, Connections, RoadMap, Kamera, Spatial-Index.
pub mod core;
/// Geteilte, host-neutrale Vertraege und Optionen.
pub mod shared;
/// XML Import/Export fuer AutoDrive-Konfigurationen.
pub mod xml;

pub use app::{AppCommand, AppController, AppIntent, AppState};
pub use xml::{parse_autodrive_config, write_autodrive_config};
