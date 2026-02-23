//! Application-Layer: Controller, State, Events und Use-Cases.

pub mod command_log;
pub mod controller;
pub mod events;
pub mod handlers;
pub mod history;
mod intent_mapping;
pub mod render_scene;
pub mod segment_registry;
/// Application State und Controller
///
/// Dieses Modul verwaltet den Zustand der Anwendung (geladene Daten, View, Tools).
pub mod state;
pub mod tools;
pub mod use_cases;

pub use crate::core::Camera2D;
pub use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
pub use crate::shared::RenderQuality;
pub use command_log::CommandLog;
pub use controller::AppController;
pub use events::{AppCommand, AppIntent};
pub use render_scene::build as build_render_scene;
pub use segment_registry::{SegmentKind, SegmentRecord, SegmentRegistry};
pub use state::{AppState, EditorTool, EditorToolState, SelectionState, UiState, ViewState};
