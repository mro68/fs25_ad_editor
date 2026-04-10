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
/// Zustandsbasierte Projektionsfunktionen fuer host-neutrale Snapshots.
pub mod projections;
/// Builder fuer explizite Render-Assets aus dem AppState.
pub mod render_assets;
/// Builder fuer Render-Szenen aus dem AppState.
pub mod render_scene;
/// Application State — zentrale Datenhaltung (View, Editor, Selektion, Dialoge).
pub mod state;
/// App-weiter Vertrag fuer Route-Tool-Identitaeten und Ankerdaten.
pub mod tool_contract;
/// Separater Persistenz- und Session-Layer fuer tool-editierbare Gruppen.
pub(crate) mod tool_editing;
/// Trait-basiertes Route-Tool-System fuer erweiterbare Strecken-Werkzeuge.
pub mod tools;
/// App-weite Read-Vertraege fuer UI-nahe Route-Tool-Daten.
pub mod ui_contract;
/// Mutierende Use-Case-Funktionen fuer alle Editing-Operationen.
pub mod use_cases;
mod viewport_overlay;

use self::tools::field_boundary::geometry::RingNodeKind;

pub use crate::core::Camera2D;
pub use crate::core::ZipImageEntry;
pub use crate::core::{
    Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
};
pub use crate::shared::RenderQuality;
pub use command_log::CommandLog;
pub use controller::AppController;
pub use events::{AppCommand, AppIntent};
pub use group_registry::{BoundaryDirection, BoundaryInfo, GroupRecord, GroupRegistry};
pub use render_assets::build as build_render_assets;
pub use render_scene::build as build_render_scene;
pub use state::{
    AppState, Clipboard, DedupDialogState, EditorTool, EditorToolState, EngineUiState,
    FloatingMenuKind, FloatingMenuState, GroupEditState, GroupSettingsPopupState,
    MarkerDialogState, OverviewOptionsDialogState, OverviewSourceContext, PostLoadDialogState,
    SaveOverviewDialogState, SelectionState, TraceAllFieldsDialogState, ViewState, ZipBrowserState,
};
pub use tool_editing::ToolEditStore;
pub use tools::field_boundary::compute_ring;

/// Ordnet die interne Ring-Klassifikation des FieldBoundary-Tools auf persistierbare Node-Flags ab.
pub(crate) fn field_boundary_ring_node_flag(kind: RingNodeKind) -> NodeFlag {
    match kind {
        RingNodeKind::RoundedCorner => NodeFlag::RoundedCorner,
        RingNodeKind::Regular | RingNodeKind::Corner => NodeFlag::Regular,
    }
}
