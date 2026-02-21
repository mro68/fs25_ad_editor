//! FS25 AutoDrive Editor Library.
//! Core-Funktionalität als Library exportiert für Tests und Wiederverwendung.

pub mod app;
pub mod core;
pub mod render;
pub mod shared;
pub mod ui;
pub mod xml;

pub use app::{
    AppCommand, AppController, AppIntent, AppState, EditorTool, EditorToolState, UiState, ViewState,
};
pub use core::{
    AutoDriveMeta, Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode,
    NodeFlag, RoadMap,
};
pub use core::{BackgroundMap, Camera2D, SpatialIndex, SpatialMatch, WorldBounds};
pub use shared::{EditorOptions, RenderQuality, RenderScene};
pub use xml::{parse_autodrive_config, write_autodrive_config};
