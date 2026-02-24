use crate::app::history::Snapshot;
use crate::app::segment_registry::SegmentRegistry;
use crate::app::CommandLog;
use crate::core::RoadMap;
use crate::shared::EditorOptions;
use std::sync::Arc;

use super::{EditorToolState, SelectionState, UiState, ViewState};

/// Hauptzustand der Anwendung
pub struct AppState {
    /// Aktuell geladene RoadMap (None = keine Datei geladen)
    pub road_map: Option<Arc<RoadMap>>,
    /// View-State
    pub view: ViewState,
    /// UI-State
    pub ui: UiState,
    /// Selection-State
    pub selection: SelectionState,
    /// Editor-Werkzeug-State
    pub editor: EditorToolState,
    /// Verlauf ausgeführter Commands
    pub command_log: CommandLog,
    /// Undo/Redo-History (Snapshot-basiert)
    pub history: crate::app::history::EditHistory,
    /// Laufzeit-Optionen (Farben, Größen, Breiten)
    pub options: EditorOptions,
    /// Ob der Options-Dialog angezeigt wird
    pub show_options_dialog: bool,
    /// In-Session-Registry aller erstellten Segmente (für nachträgliche Bearbeitung)
    pub segment_registry: SegmentRegistry,
    /// Signalisiert dem Host (eframe), die Anwendung kontrolliert zu beenden
    pub should_exit: bool,
}

impl AppState {
    /// Erstellt einen neuen, leeren App-State
    pub fn new() -> Self {
        Self {
            road_map: None,
            view: ViewState::new(),
            ui: UiState::new(),
            selection: SelectionState::new(),
            editor: EditorToolState::new(),
            command_log: CommandLog::new(),
            history: crate::app::history::EditHistory::new_with_capacity(200),
            options: EditorOptions::default(),
            show_options_dialog: false,
            segment_registry: SegmentRegistry::new(),
            should_exit: false,
        }
    }

    /// Gibt die Anzahl der Nodes zurück (für UI-Anzeige)
    pub fn node_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.node_count())
    }

    /// Gibt die Anzahl der Connections zurück (für UI-Anzeige)
    pub fn connection_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.connection_count())
    }

    /// Undo/Redo helpers
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Gibt zurück, ob ein Redo-Schritt verfügbar ist.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Erstellt einen Undo-Snapshot des aktuellen Zustands.
    /// Reduziert Boilerplate in mutierenden Use-Cases.
    pub fn record_undo_snapshot(&mut self) {
        let snap = Snapshot::from_state(self);
        self.history.record_snapshot(snap);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
