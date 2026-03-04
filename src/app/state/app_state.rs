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
    /// Verlauf ausgefuehrter Commands
    pub command_log: CommandLog,
    /// Undo/Redo-History (Snapshot-basiert)
    pub history: crate::app::history::EditHistory,
    /// Laufzeit-Optionen (Farben, Groessen, Breiten)
    pub options: EditorOptions,
    /// Geteilte Arc-Variante der Optionen fuer RenderScene-Build (O(1) Clone pro Frame)
    options_arc: Arc<EditorOptions>,
    /// Ob der Options-Dialog angezeigt wird
    pub show_options_dialog: bool,
    /// In-Session-Registry aller erstellten Segmente (fuer nachtraegliche Bearbeitung)
    pub segment_registry: SegmentRegistry,
    /// Signalisiert dem Host (eframe), die Anwendung kontrolliert zu beenden
    pub should_exit: bool,
}

impl AppState {
    /// Erstellt einen neuen, leeren App-State
    pub fn new() -> Self {
        let options = EditorOptions::default();
        let options_arc = Arc::new(options.clone());

        Self {
            road_map: None,
            view: ViewState::new(),
            ui: UiState::new(),
            selection: SelectionState::new(),
            editor: EditorToolState::new(),
            command_log: CommandLog::new(),
            history: crate::app::history::EditHistory::new_with_capacity(200),
            options,
            options_arc,
            show_options_dialog: false,
            segment_registry: SegmentRegistry::new(),
            should_exit: false,
        }
    }

    /// Gibt die Anzahl der Nodes zurueck (fuer UI-Anzeige)
    pub fn node_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.node_count())
    }

    /// Gibt die Anzahl der Connections zurueck (fuer UI-Anzeige)
    pub fn connection_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.connection_count())
    }

    /// Undo/Redo helpers
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Gibt zurueck, ob ein Redo-Schritt verfuegbar ist.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Erstellt einen Undo-Snapshot des aktuellen Zustands.
    /// Reduziert Boilerplate in mutierenden Use-Cases.
    pub fn record_undo_snapshot(&mut self) {
        let snap = Snapshot::from_state(self);
        self.history.record_snapshot(snap);
    }

    /// Liefert die Arc-Variante der Optionen (fuer RenderScene-Build, zero-copy pro Frame).
    pub fn options_arc(&self) -> Arc<EditorOptions> {
        self.options_arc.clone()
    }

    /// Setzt neue Optionen und aktualisiert den geteilten Arc.
    pub fn set_options(&mut self, options: EditorOptions) {
        self.options = options;
        self.options_arc = Arc::new(self.options.clone());
    }

    /// Aktualisiert den geteilten Arc nach in-place Mutationen der Optionen.
    pub fn refresh_options_arc(&mut self) {
        self.options_arc = Arc::new(self.options.clone());
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
