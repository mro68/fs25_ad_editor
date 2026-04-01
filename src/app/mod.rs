//! Application-Layer: Controller, State, Events und Use-Cases.

/// Minimales Command-Log fuer Debug-Zwecke.
pub mod command_log;
/// Application Controller fuer zentrales Intent-Command-Dispatch.
pub mod controller;
/// AppIntent- und AppCommand-Events des Application-Layers.
pub mod events;
/// In-Session-Registry aller erstellten Segmente (fuer nachtraegliche Bearbeitung).
pub mod group_registry;
/// Feature-Handler fuer AppCommand-Verarbeitung.
pub mod handlers;
/// Undo/Redo-History mit Arc-basierten Snapshots (Copy-on-Write).
pub mod history;
mod intent_mapping;
/// Builder fuer Render-Szenen aus dem AppState.
pub mod render_scene;
/// Application State — zentrale Datenhaltung (View, Editor, Selektion, Dialoge).
pub mod state;
/// App-weiter Vertrag fuer Route-Tool-Identitaeten und Ankerdaten.
pub mod tool_contract;
/// App-weite Read-Vertraege fuer UI-nahe Route-Tool-Daten.
pub mod ui_contract;
/// Trait-basiertes Route-Tool-System fuer erweiterbare Strecken-Werkzeuge.
pub mod tools;
/// Mutierende Use-Case-Funktionen fuer alle Editing-Operationen.
pub mod use_cases;

pub use crate::core::Camera2D;
pub use crate::core::ZipImageEntry;
pub use crate::core::{
    BoundaryNode, Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode,
    NodeFlag, RoadMap,
};
pub use crate::shared::RenderQuality;
pub use command_log::CommandLog;
pub use controller::AppController;
pub use events::{AppCommand, AppIntent};
pub use group_registry::{
    BoundaryDirection, BoundaryInfo, GroupBase, GroupKind, GroupRecord, GroupRegistry,
};
pub use render_scene::build as build_render_scene;
pub use state::{
    AppState, Clipboard, EditorTool, EditorToolState, FloatingMenuKind, FloatingMenuState,
    GroupEditState, GroupSettingsPopupState, PostLoadDialogState, SelectionState, UiState,
    ViewState,
};
pub use tool_contract::{RouteToolId, TangentSource, ToolAnchor};
pub use ui_contract::{TangentMenuData, TangentOptionData};
pub use tools::field_boundary::compute_ring;
pub use tools::field_boundary::RingNodeKind;
