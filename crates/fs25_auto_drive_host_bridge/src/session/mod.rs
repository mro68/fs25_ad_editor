use fs25_auto_drive_engine::app::state::DistanzenState;
use fs25_auto_drive_engine::app::{
    AppController, AppState, Camera2D, ConnectionDirection, ConnectionPriority, EditorTool,
    GroupEditState, GroupRegistry, RoadMap, ToolEditStore,
};
use fs25_auto_drive_engine::shared::{EditorOptions, RenderAssetsSnapshot, RenderScene};
use indexmap::IndexSet;

mod chrome_state;
mod context_menu;
mod lifecycle;
mod read_models;
/// Chrome-/Dialog-Gruppe: host-lokale Panel-, Dialog- und Floating-Menu-Seams.
mod session_chrome;
/// Dispatch-Gruppe: `apply_action`/`apply_intent` und Komfort-Actions (Undo/Redo, Dialog-Drain).
mod session_dispatch;
/// Read-Model-/JSON-Gruppe: getypte und JSON-serialisierte Node-/Marker-/Connection-Reads.
mod session_read_models;
/// Snapshot-Gruppe: alle `build_*`/`snapshot*`-Methoden fuer Render- und UI-Vertraege.
mod session_snapshots;
mod snapshots;
pub use chrome_state::HostLocalDialogState;

use crate::dispatch::HostViewportInputState;
use crate::dto::{
    HostConnectionPairSnapshot, HostDefaultConnectionDirection, HostDefaultConnectionPriority,
    HostDialogRequest, HostMarkerListSnapshot, HostNodeDetails, HostSessionSnapshot,
};
use snapshots::build_snapshot;

fn map_connection_direction(direction: ConnectionDirection) -> HostDefaultConnectionDirection {
    match direction {
        ConnectionDirection::Regular => HostDefaultConnectionDirection::Regular,
        ConnectionDirection::Dual => HostDefaultConnectionDirection::Dual,
        ConnectionDirection::Reverse => HostDefaultConnectionDirection::Reverse,
    }
}

fn map_connection_priority(priority: ConnectionPriority) -> HostDefaultConnectionPriority {
    match priority {
        ConnectionPriority::Regular => HostDefaultConnectionPriority::Regular,
        ConnectionPriority::SubPriority => HostDefaultConnectionPriority::SubPriority,
    }
}

/// Gekoppelter Render-Snapshot fuer Hosts ohne direkte State-Inspektion.
///
/// Hosts koennen damit den per-frame Render-Vertrag und die langlebigen
/// Render-Assets als konsistentes read-only Paar abrufen, ohne `AppState`
/// direkt auszulesen.
pub struct HostRenderFrameSnapshot {
    /// Per-frame Render-Vertrag mit Kamera-, Sichtbarkeits- und Geometriedaten.
    pub scene: RenderScene,
    /// Langlebige Render-Assets inklusive Revisionen.
    pub assets: RenderAssetsSnapshot,
}

/// Schmaler State-Zugriff fuer Properties- und Edit-Panel im Host.
///
/// Die Struktur liefert genau die aktuell benoetigten Read-/Write-Felder,
/// ohne den kompletten `AppState` als mutable Escape-Hatch freizugeben.
pub struct HostPanelPropertiesState<'a> {
    /// Aktuelle Karte (falls geladen).
    pub road_map: Option<&'a RoadMap>,
    /// Aktuelle Selektion der Node-IDs.
    pub selected_node_ids: &'a IndexSet<u64>,
    /// Standardrichtung fuer neue Verbindungen.
    pub default_direction: ConnectionDirection,
    /// Standardprioritaet fuer neue Verbindungen.
    pub default_priority: ConnectionPriority,
    /// Aktives Editor-Werkzeug.
    pub active_tool: EditorTool,
    /// Registry fuer Segment-/Gruppeninformationen.
    pub group_registry: &'a GroupRegistry,
    /// Store fuer tool-spezifische Edit-Payloads.
    pub tool_edit_store: &'a ToolEditStore,
    /// Aktiver Gruppen-Edit-State (falls vorhanden).
    pub group_editing: Option<&'a GroupEditState>,
    /// Lokaler Distanz-Dialogzustand fuer Properties/Edit-Panel.
    pub distanzen: &'a mut DistanzenState,
    /// Laufzeit-Optionen (werden im Edit-Panel lokal editiert).
    pub options: &'a mut EditorOptions,
}

/// Schmaler Dialogzugriff fuer host-lokale Modalfenster.
///
/// Hosts koennen damit Dialog-UI mutieren, ohne direkten Vollzugriff auf den
/// gesamten `AppState` zu erhalten. Das `ui`-Feld zeigt auf den
/// host-lokalen `HostLocalDialogState`, nicht mehr auf den Engine-`UiState`.
pub struct HostDialogUiState<'a> {
    /// Aktuelle Karte (falls geladen).
    pub road_map: Option<&'a RoadMap>,
    /// Host-lokaler Dialog- und Chrome-Sichtbarkeitszustand.
    pub ui: &'a mut HostLocalDialogState,
    /// Laufzeit-Optionen fuer dialogspezifische Einstellungen.
    pub options: &'a mut EditorOptions,
}

/// Schmaler Viewport-Input-Zugriff fuer Host-Event-Sammler.
///
/// Kombiniert die benoetigten read-only Viewport-Daten mit dem lokal
/// mutierbaren Distanzzustand.
pub struct HostViewportInputContext<'a> {
    /// Ob eine Paste-Vorschau aktiv ist.
    pub paste_preview_active: bool,
    /// Aktuelle Kamera des Viewports.
    pub camera: &'a Camera2D,
    /// Aktuelle Karte (falls geladen).
    pub road_map: Option<&'a RoadMap>,
    /// Aktuelle Selektion der Node-IDs.
    pub selected_node_ids: &'a IndexSet<u64>,
    /// Aktives Editor-Werkzeug.
    pub active_tool: EditorTool,
    /// Standardrichtung fuer neue Verbindungen.
    pub default_direction: ConnectionDirection,
    /// Standardprioritaet fuer neue Verbindungen.
    pub default_priority: ConnectionPriority,
    /// Laufzeitoptionen fuer Input-/Kontextmenue-Guards.
    pub options: &'a EditorOptions,
    /// Ob Clipboard-Daten zum Einfuegen vorhanden sind.
    pub clipboard_has_nodes: bool,
    /// Ob Farmland-Polygone fuer feldbezogene Aktionen verfuegbar sind.
    pub farmland_available: bool,
    /// Ob eine Gruppenbearbeitung aktiv ist.
    pub group_editing_active: bool,
    /// Registry fuer gruppenbezogene Kontextmenue-Optionen.
    pub group_registry: &'a GroupRegistry,
    /// Lokaler Distanzzustand (wird ueber Maus/Shortcuts mutiert).
    pub distanzen: &'a mut DistanzenState,
}

/// Kompatibilitaetsalias fuer bestehende direkte Flutter-/FFI-Session-Importe.
pub type FlutterBridgeSession = HostBridgeSession;

/// Kompatibilitaetsalias fuer bestehende direkte Flutter-/FFI-Render-Importe.
pub type EngineRenderFrameSnapshot = HostRenderFrameSnapshot;

/// Kleine Session-Fassade ueber der host-neutralen Engine.
///
/// Die Bridge kapselt `AppController` und `AppState`, bleibt aber absichtlich
/// toolkit-frei. Host-spezifische Runtime-, Dialog- oder FFI-Details bleiben in
/// den jeweiligen Host-Adaptern.
pub struct HostBridgeSession {
    controller: AppController,
    state: AppState,
    viewport_input_state: HostViewportInputState,
    snapshot_cache: HostSessionSnapshot,
    snapshot_dirty: bool,
    inspected_node_id: Option<u64>,
    /// Host-lokaler Chrome- und Dialog-Sichtbarkeitszustand.
    chrome_state: HostLocalDialogState,
    /// Puffer fuer PickPath-Requests, die aus dem Engine-Queue gefiltert wurden.
    pending_dialog_requests: Vec<HostDialogRequest>,
}

impl HostBridgeSession {
    /// Erstellt eine neue Host-Bridge-Session mit leerem Engine-State.
    pub fn new() -> Self {
        let state = AppState::new();
        let chrome = HostLocalDialogState::new();
        let snapshot_cache = build_snapshot(&state, &chrome, 0);

        Self {
            controller: AppController::new(),
            state,
            viewport_input_state: HostViewportInputState::default(),
            snapshot_cache,
            snapshot_dirty: false,
            inspected_node_id: None,
            chrome_state: chrome,
            pending_dialog_requests: Vec::new(),
        }
    }

    /// Prueft, ob die Applikation beendet werden soll.
    pub fn should_exit(&self) -> bool {
        self.app_state().should_exit
    }

    /// Gibt zurueck, ob die geladene Karte seit dem letzten Load/Save veraendert wurde.
    pub fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    /// Invalidiert den gecachten `HostSessionSnapshot` explizit.
    ///
    /// Rust-Hosts nutzen diese Hilfsmethode nach lokalen Mutationen ueber
    /// schmale UI-Seams, falls dabei ausnahmsweise Felder veraendert wurden,
    /// die in `HostSessionSnapshot` gespiegelt werden.
    pub fn mark_snapshot_dirty(&mut self) {
        self.snapshot_dirty = true;
    }
}

impl Default for HostBridgeSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
