use anyhow::Result;
use fs25_auto_drive_engine::app::projections as engine_projections;
use fs25_auto_drive_engine::app::state::DistanzenState;
use fs25_auto_drive_engine::app::ui_contract::{
    DialogRequest, HostUiSnapshot, PanelState, ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::{
    AppController, AppIntent, AppState, Camera2D, ConnectionDirection, ConnectionPriority,
    EditorTool, FloatingMenuKind, FloatingMenuState, GroupEditState, GroupRegistry, RoadMap,
    ToolEditStore,
};
use fs25_auto_drive_engine::shared::{EditorOptions, RenderAssetsSnapshot, RenderScene};
use glam::Vec2;
use indexmap::IndexSet;
use std::collections::BTreeSet;

mod chrome_state;
mod context_menu;
mod mappings;
mod snapshots;
pub use chrome_state::HostLocalDialogState;

use mappings::{
    map_connection_direction, map_connection_priority, map_host_field_detection_source_to_engine,
    map_host_overview_layers_to_engine,
};
use snapshots::{build_dialog_snapshot, build_editing_snapshot, build_snapshot};

use crate::dispatch::HostViewportInputState;
use crate::dto::{
    HostActiveTool, HostChromeSnapshot, HostConnectionPairEntry, HostConnectionPairSnapshot,
    HostContextMenuSnapshot, HostDialogRequest, HostDialogResult, HostDialogSnapshot,
    HostEditingSnapshot, HostMarkerInfo, HostMarkerListSnapshot, HostNodeDetails, HostNodeFlag,
    HostNodeMarkerInfo, HostNodeNeighbor, HostOverviewOptionsDialogSnapshot,
    HostRouteToolViewportSnapshot, HostSessionAction, HostSessionSnapshot,
    HostViewportGeometrySnapshot,
};

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
    snapshot_dirty: &'a mut bool,
    show_command_palette_before: bool,
    show_options_dialog_before: bool,
    chrome_dirty_before: bool,
}

impl HostDialogUiState<'_> {
    fn snapshot_relevant_changed(&self) -> bool {
        self.ui.show_command_palette != self.show_command_palette_before
            || self.ui.show_options_dialog != self.show_options_dialog_before
    }
}

impl Drop for HostDialogUiState<'_> {
    fn drop(&mut self) {
        let snapshot_relevant_changed = self.snapshot_relevant_changed();
        if snapshot_relevant_changed {
            self.ui.mark_dirty();
        }

        if snapshot_relevant_changed || (!self.chrome_dirty_before && self.ui.chrome_dirty) {
            *self.snapshot_dirty = true;
        }
    }
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

    fn reconcile_host_local_dialog_state_for_intent(&mut self, intent: &AppIntent) {
        if matches!(intent, AppIntent::OverviewOptionsConfirmed) {
            self.state.ui.overview_options_dialog =
                self.chrome_state.overview_options_dialog.clone();
        }
    }

    /// Wendet eine explizite Host-Aktion auf die Session an.
    ///
    /// Die Methode delegiert auf die gemeinsame Rust-Host-Dispatch-Seam in
    /// `crate::dispatch::apply_host_action(...)` und markiert den Snapshot-
    /// Cache nur nach erfolgreich verarbeiteten Aktionen als dirty.
    pub fn apply_action(&mut self, action: HostSessionAction) -> Result<()> {
        if let HostSessionAction::QueryNodeDetails { node_id } = action {
            self.set_inspected_node_id(Some(node_id));
            return Ok(());
        }

        let handled = crate::dispatch::apply_host_action_with_viewport_input_state(
            &mut self.controller,
            &mut self.state,
            &mut self.viewport_input_state,
            action,
        )?;
        if handled {
            self.snapshot_dirty = true;
            self.drain_engine_requests();
            self.sync_chrome_from_engine();
        }
        Ok(())
    }

    /// Verarbeitet einen `AppIntent` direkt ueber den App-Controller.
    ///
    /// Diese Methode bleibt als Uebergangs-Seam fuer Hosts, die bereits auf
    /// Session-Ownership umgestellt sind, aber noch nicht alle Schreibpfade auf
    /// `HostSessionAction` umgehangen haben. Vor der Verarbeitung werden fuer
    /// intentsensitive Host-Dialoge die lokalen Draft-Werte in den Engine-State
    /// zurueckgespiegelt.
    pub fn apply_intent(&mut self, intent: AppIntent) -> Result<()> {
        self.reconcile_host_local_dialog_state_for_intent(&intent);
        self.controller.handle_intent(&mut self.state, intent)?;
        self.snapshot_dirty = true;
        self.drain_engine_requests();
        self.sync_chrome_from_engine();
        Ok(())
    }

    /// Liefert eine read-only Referenz auf den aktuellen `AppState`.
    ///
    /// Diese API ist als temporaere Read-Seam fuer den Ownership-Flip gedacht,
    /// bis alle host-neutralen Snapshots konsumiert werden.
    pub fn app_state(&self) -> &AppState {
        &self.state
    }

    /// Liefert die Details eines Nodes als getypten Rust-Struct.
    ///
    /// Die Methode ist ein reiner Read ohne JSON-Serialisierung und ohne
    /// Seiteneffekt auf `inspected_node_id`.
    pub fn node_details(&self, node_id: u64) -> Option<HostNodeDetails> {
        self.build_node_details_for(node_id)
    }

    /// Liefert die komplette Markerliste als getypten Rust-Struct.
    pub fn marker_list(&self) -> HostMarkerListSnapshot {
        self.build_marker_list_snapshot()
    }

    /// Liefert die Verbindungsdetails zwischen zwei Nodes.
    pub fn connection_pair(&self, node_a: u64, node_b: u64) -> HostConnectionPairSnapshot {
        self.build_connection_pair_snapshot(node_a, node_b)
    }

    /// Prueft, ob die Applikation beendet werden soll.
    pub fn should_exit(&self) -> bool {
        self.app_state().should_exit
    }

    /// Gibt zurueck, ob die geladene Karte seit dem letzten Load/Save veraendert wurde.
    pub fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    /// Setzt die aktuell fuer das Properties-Panel inspizierte Node-ID.
    pub fn set_inspected_node_id(&mut self, id: Option<u64>) {
        self.inspected_node_id = id;
    }

    /// Liefert die aktuell fuer das Properties-Panel inspizierte Node-ID.
    pub fn inspected_node_id(&self) -> Option<u64> {
        self.inspected_node_id
    }

    /// Invalidiert den gecachten `HostSessionSnapshot` explizit.
    ///
    /// Rust-Hosts nutzen diese Hilfsmethode nach lokalen Mutationen ueber
    /// snapshot-transparente Seams wie `panel_properties_state_mut()` oder
    /// `viewport_input_context_mut()`, falls dabei ausnahmsweise Felder
    /// veraendert wurden, die in `HostSessionSnapshot` gespiegelt werden.
    /// `chrome_state_mut()` invalidiert bereits direkt und
    /// `dialog_ui_state_mut()` uebernimmt dies bei snapshot-relevanten
    /// Aenderungen ueber den Rueckgabe-Guard automatisch.
    pub fn mark_snapshot_dirty(&mut self) {
        self.snapshot_dirty = true;
    }

    /// Liefert eine read-only Referenz auf den host-lokalen Chrome-/Dialog-Zustand.
    pub fn chrome_state(&self) -> &HostLocalDialogState {
        &self.chrome_state
    }

    /// Liefert eine mutable Referenz auf den host-lokalen Chrome-/Dialog-Zustand.
    ///
    /// Dieser Accessor invalidiert den Session-Snapshot vorsorglich sofort,
    /// damit auch direkte lokale Mutationen ohne explizites Dirty-Marking keine
    /// stale Snapshot-Daten hinterlassen.
    pub fn chrome_state_mut(&mut self) -> &mut HostLocalDialogState {
        self.snapshot_dirty = true;
        &mut self.chrome_state
    }

    /// Liefert den schmalen Properties-/Edit-Panel-Zugriff.
    ///
    /// Diese Seams kapseln die verbleibenden host-lokalen UI-Mutationen
    /// (`distanzen`, `options`) bei gleichzeitig read-only Zugriff auf
    /// Selektions-/Gruppen-/Karteninformationen. Der Zugriff bleibt bewusst
    /// Snapshot-transparent, weil diese lokalen Felder nicht Teil des kleinen
    /// `HostSessionSnapshot` sind.
    pub fn panel_properties_state_mut(&mut self) -> HostPanelPropertiesState<'_> {
        let state = &mut self.state;

        HostPanelPropertiesState {
            road_map: state.road_map.as_deref(),
            selected_node_ids: &state.selection.selected_node_ids,
            default_direction: state.editor.default_direction,
            default_priority: state.editor.default_priority,
            active_tool: state.editor.active_tool,
            group_registry: &state.group_registry,
            tool_edit_store: &state.tool_edit_store,
            group_editing: state.group_editing.as_ref(),
            distanzen: &mut state.ui.distanzen,
            options: &mut state.options,
        }
    }

    /// Liefert den schmalen Dialogzugriff fuer host-lokale Modalfenster.
    ///
    /// Das `ui`-Feld zeigt nun auf `chrome_state` (statt `state.ui`), sodass
    /// Dialog-Mutationen durch das Frontend direkt im host-lokalen State landen.
    /// Beim Drop des Rueckgabe-Guards werden Snapshot-relevante Chrome-Felder
    /// geprueft; bei Aenderungen wird der Session-Snapshot automatisch als dirty
    /// markiert.
    pub fn dialog_ui_state_mut(&mut self) -> HostDialogUiState<'_> {
        let show_command_palette_before = self.chrome_state.show_command_palette;
        let show_options_dialog_before = self.chrome_state.show_options_dialog;
        let chrome_dirty_before = self.chrome_state.chrome_dirty;

        HostDialogUiState {
            road_map: self.state.road_map.as_deref(),
            ui: &mut self.chrome_state,
            options: &mut self.state.options,
            snapshot_dirty: &mut self.snapshot_dirty,
            show_command_palette_before,
            show_options_dialog_before,
            chrome_dirty_before,
        }
    }

    /// Aktualisiert den host-lokalen Draft des Overview-Options-Dialogs aus einem DTO-Snapshot.
    ///
    /// Serialisierbare Hosts spiegeln damit lokal editierte Dialogwerte in die
    /// Session zurueck, bevor sie `OverviewOptionsConfirmed` ausloesen. Der
    /// Engine-Dialogzustand bleibt bis zur Bestaetigung unveraendert.
    pub fn update_overview_options_dialog(&mut self, snapshot: HostOverviewOptionsDialogSnapshot) {
        self.chrome_state.overview_options_dialog.visible = snapshot.visible;
        self.chrome_state.overview_options_dialog.zip_path = snapshot.zip_path;
        self.chrome_state.overview_options_dialog.layers =
            map_host_overview_layers_to_engine(&snapshot.layers);
        self.chrome_state
            .overview_options_dialog
            .field_detection_source =
            map_host_field_detection_source_to_engine(snapshot.field_detection_source);
        self.chrome_state.overview_options_dialog.available_sources = snapshot
            .available_sources
            .into_iter()
            .map(map_host_field_detection_source_to_engine)
            .collect();
        self.chrome_state.mark_dirty();
        self.snapshot_dirty = true;
    }

    /// Liefert den schmalen Viewport-Input-Zugriff fuer Host-Event-Sammler.
    ///
    /// Der Zugriff bleibt bewusst Snapshot-transparent, weil der lokale
    /// Distanzzustand nicht im kleinen `HostSessionSnapshot` gespiegelt wird.
    pub fn viewport_input_context_mut(&mut self) -> HostViewportInputContext<'_> {
        let state = &mut self.state;
        let farmland_available = state
            .farmland_polygons_arc()
            .is_some_and(|polygons| !polygons.is_empty());

        HostViewportInputContext {
            paste_preview_active: state.paste_preview_pos.is_some(),
            camera: &state.view.camera,
            road_map: state.road_map.as_deref(),
            selected_node_ids: &state.selection.selected_node_ids,
            active_tool: state.editor.active_tool,
            default_direction: state.editor.default_direction,
            default_priority: state.editor.default_priority,
            options: &state.options,
            clipboard_has_nodes: !state.clipboard.nodes.is_empty(),
            farmland_available,
            group_editing_active: state.group_editing.is_some(),
            group_registry: &state.group_registry,
            distanzen: &mut state.ui.distanzen,
        }
    }

    /// Schliesst das host-lokale Floating-Menue explizit.
    pub fn clear_floating_menu(&mut self) {
        self.chrome_state.floating_menu = None;
        self.chrome_state.mark_dirty();
        self.snapshot_dirty = true;
    }

    /// Schaltet das host-lokale Floating-Menue fuer den angegebenen Menue-Typ um.
    ///
    /// `pointer_pos` beschreibt die aktuelle Pointer-Position in Host-Pixeln.
    /// Ist keine Position verfuegbar, wird bei Aktivierung kein Menue geoeffnet.
    pub fn toggle_floating_menu(&mut self, kind: FloatingMenuKind, pointer_pos: Option<Vec2>) {
        let next_menu = match self.chrome_state.floating_menu {
            Some(existing) if existing.kind == kind => None,
            Some(_) | None => pointer_pos.map(|pos| FloatingMenuState { kind, pos }),
        };

        self.chrome_state.floating_menu = next_menu;
        self.chrome_state.mark_dirty();
        self.snapshot_dirty = true;
    }

    /// Setzt die aktuelle Statusmeldung explizit.
    pub fn set_status_message(&mut self, message: Option<String>) {
        self.state.ui.status_message = message;
        self.snapshot_dirty = true;
    }

    /// Schaltet die Command-Palette um.
    pub fn toggle_command_palette(&mut self) -> Result<()> {
        self.apply_action(HostSessionAction::ToggleCommandPalette)
    }

    /// Setzt das aktive Editor-Tool.
    pub fn set_editor_tool(&mut self, tool: HostActiveTool) -> Result<()> {
        self.apply_action(HostSessionAction::SetEditorTool { tool })
    }

    /// Oeffnet oder schliesst den Optionen-Dialog explizit.
    pub fn set_options_dialog_visible(&mut self, visible: bool) -> Result<()> {
        let action = if visible {
            HostSessionAction::OpenOptionsDialog
        } else {
            HostSessionAction::CloseOptionsDialog
        };
        self.apply_action(action)
    }

    /// Fuehrt einen Undo-Schritt aus.
    pub fn undo(&mut self) -> Result<()> {
        self.apply_action(HostSessionAction::Undo)
    }

    /// Fuehrt einen Redo-Schritt aus.
    pub fn redo(&mut self) -> Result<()> {
        self.apply_action(HostSessionAction::Redo)
    }

    /// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen als Bridge-DTOs.
    ///
    /// Dies ist die kanonische oeffentliche Dialog-Drain-Seam der Bridge fuer
    /// Hosts ohne direkten Zugriff auf `AppController` und `AppState`.
    /// Chrome-Sichtbarkeits-Requests werden hier NICHT zurueckgegeben — sie
    /// werden durch `drain_engine_requests()` direkt in `chrome_state` verarbeitet.
    pub fn take_dialog_requests(&mut self) -> Vec<HostDialogRequest> {
        let requests = std::mem::take(&mut self.pending_dialog_requests);
        if !requests.is_empty() {
            self.snapshot_dirty = true;
        }
        requests
    }

    /// Reicht ein host-seitiges Dialog-Ergebnis an die Engine weiter.
    ///
    /// Dies ist das semantische Gegenstueck zur Dialog-Drain-Seam
    /// `take_dialog_requests()`.
    pub fn submit_dialog_result(&mut self, result: HostDialogResult) -> Result<()> {
        self.apply_action(HostSessionAction::SubmitDialogResult { result })
    }

    /// Liefert einen referenzierten Snapshot fuer Polling-Hosts.
    ///
    /// Der Snapshot wird nur nach einer erfolgreichen Session-Mutation neu
    /// aufgebaut, damit bei Polling ohne State-Aenderung keine neuen Allokationen
    /// entstehen.
    pub fn snapshot(&mut self) -> &HostSessionSnapshot {
        self.rebuild_snapshot_if_dirty();
        &self.snapshot_cache
    }

    /// Liefert eine besitzende Snapshot-Kopie.
    ///
    /// Diese Methode ist fuer Call-Sites gedacht, die den Snapshot vom Session-
    /// Lebenszyklus entkoppeln muessen.
    pub fn snapshot_owned(&mut self) -> HostSessionSnapshot {
        self.snapshot().clone()
    }

    /// Liefert einen host-neutralen Snapshot aller egui-Dialogzustaende.
    ///
    /// Der Snapshot liest sowohl den host-lokalen `chrome_state` als auch die
    /// fuer Dialog-Popups relevanten Engine-Optionen. Er ist bewusst von
    /// `HostSessionSnapshot` getrennt, damit Flutter und spaetere Hosts die
    /// komplexeren Dialog-Drafts als eigene serialisierbare Surface pollen
    /// koennen, ohne auf `dialog_ui_state_mut()` oder `chrome_state()`
    /// angewiesen zu sein.
    pub fn dialog_snapshot(&self) -> HostDialogSnapshot {
        build_dialog_snapshot(&self.state, &self.chrome_state)
    }

    /// Liefert einen serialisierbaren Snapshot fuer Properties-, Group-Edit- und Resample-Daten.
    ///
    /// Der Snapshot bildet die aktuell ueber `panel_properties_state_mut()` und
    /// `viewport_input_context_mut()` gelesenen Editing-Zustaende host-neutral ab,
    /// damit Flutter und spaetere Hosts dieselben Daten ohne Rust-spezifische
    /// Escape-Hatches pollen koennen.
    pub fn editing_snapshot(&self) -> HostEditingSnapshot {
        build_editing_snapshot(&self.state)
    }

    /// Liefert einen serialisierbaren Snapshot des aktuell relevanten Kontextmenues.
    ///
    /// Die Bridge spiegelt damit die egui-Preconditions fuer Kontextmenue-Aktionen
    /// host-neutral in einer flachen Aktionsliste. `focus_node_id` entspricht dem
    /// vom Host bereits ermittelten fokussierten Node; `None` bedeutet Klick in den
    /// leeren Bereich.
    pub fn context_menu_snapshot(&self, focus_node_id: Option<u64>) -> HostContextMenuSnapshot {
        context_menu::build_context_menu_snapshot(&self.state, focus_node_id)
    }

    /// Serialisiert den aktuell inspizierten Node als JSON fuer Flutter.
    pub fn node_details_json(&self) -> Option<String> {
        let snapshot = self
            .inspected_node_id
            .and_then(|node_id| self.node_details(node_id))?;
        serde_json::to_string(&snapshot).ok()
    }

    /// Serialisiert die aktuelle Marker-Liste als JSON fuer Flutter.
    pub fn marker_list_json(&self) -> String {
        serde_json::to_string(&self.marker_list())
            .unwrap_or_else(|_| "{\"markers\":[],\"groups\":[]}".to_string())
    }

    /// Baut den aktuellen per-frame Render-Vertrag fuer den angegebenen Viewport.
    pub fn build_render_scene(&self, viewport_size: [f32; 2]) -> RenderScene {
        engine_projections::build_render_scene(&self.state, viewport_size)
    }

    /// Baut den aktuellen Render-Asset-Snapshot.
    pub fn build_render_assets(&self) -> RenderAssetsSnapshot {
        engine_projections::build_render_assets(&self.state)
    }

    /// Baut einen gekoppelten Render-Snapshot aus Szene und Assets.
    ///
    /// Diese Hilfsmethode ist fuer Hosts gedacht, die pro Tick genau einen
    /// read-only Render-Output benoetigen und Szene/Assets nicht separat pollen
    /// wollen.
    pub fn build_render_frame(&self, viewport_size: [f32; 2]) -> HostRenderFrameSnapshot {
        HostRenderFrameSnapshot {
            scene: self.build_render_scene(viewport_size),
            assets: self.build_render_assets(),
        }
    }

    /// Baut einen vollstaendigen, serialisierbaren Viewport-Geometry-Snapshot.
    pub fn build_viewport_geometry_snapshot(
        &self,
        viewport_size: [f32; 2],
    ) -> HostViewportGeometrySnapshot {
        crate::dispatch::build_viewport_geometry_snapshot(&self.state, viewport_size)
    }

    /// Baut den host-neutralen Host-UI-Snapshot fuer sichtbare Panels.
    ///
    /// Host-native Datei- und Pfaddialoge laufen bewusst nicht ueber diesen
    /// Snapshot, sondern separat ueber `take_dialog_requests()`.
    /// Die Panel-Sichtbarkeit (`show_command_palette`, `show_options_dialog`)
    /// stammt aus dem `chrome_state` und wird hier eingefuegt.
    pub fn build_host_ui_snapshot(&self) -> HostUiSnapshot {
        let mut snapshot = engine_projections::build_host_ui_snapshot(&self.state);
        for panel in &mut snapshot.panels {
            match panel {
                PanelState::CommandPalette(state) => {
                    state.visible = self.chrome_state.show_command_palette;
                }
                PanelState::Options(state) => {
                    state.visible = self.chrome_state.show_options_dialog;
                }
                _ => {}
            }
        }
        snapshot
    }

    /// Baut den host-neutralen Chrome-Snapshot fuer Menues, Defaults und Status.
    ///
    /// Die Felder `show_command_palette` und `show_options_dialog` stammen aus
    /// `chrome_state`, das per `drain_engine_requests()` nach jedem Engine-Intent
    /// aktualisiert wird.
    pub fn build_host_chrome_snapshot(&self) -> HostChromeSnapshot {
        let mut snapshot = crate::dispatch::build_host_chrome_snapshot(&self.state);
        snapshot.show_command_palette = self.chrome_state.show_command_palette;
        snapshot.show_options_dialog = self.chrome_state.show_options_dialog;
        snapshot
    }

    /// Baut den host-neutralen Route-Tool-Viewport-Snapshot.
    pub fn build_route_tool_viewport_snapshot(&self) -> HostRouteToolViewportSnapshot {
        crate::dispatch::build_route_tool_viewport_snapshot(&self.state)
    }

    /// Baut den host-neutralen Overlay-Snapshot fuer den aktuellen Viewport.
    ///
    /// Die Methode benoetigt mutablen Zugriff, weil der App-Layer beim Aufbau
    /// bei Bedarf Overlay- und Boundary-Caches im `AppState` aufwaermt.
    pub fn build_viewport_overlay_snapshot(
        &mut self,
        cursor_world: Option<Vec2>,
    ) -> ViewportOverlaySnapshot {
        engine_projections::build_viewport_overlay_snapshot(&mut self.state, cursor_world)
    }

    fn build_node_details_for(&self, node_id: u64) -> Option<HostNodeDetails> {
        let road_map = self.state.road_map.as_deref()?;
        let node = road_map.node(node_id)?;

        Some(HostNodeDetails {
            id: node.id,
            position: [node.position.x, node.position.y],
            flag: HostNodeFlag::from(&node.flag),
            neighbors: road_map
                .connected_neighbors(node_id)
                .into_iter()
                .map(|neighbor| HostNodeNeighbor {
                    neighbor_id: neighbor.neighbor_id,
                    angle: neighbor.angle,
                    is_outgoing: neighbor.is_outgoing,
                })
                .collect(),
            marker: road_map
                .find_marker_by_node_id(node_id)
                .map(|marker| HostNodeMarkerInfo {
                    name: marker.name.clone(),
                    group: marker.group.clone(),
                }),
        })
    }

    fn build_marker_list_snapshot(&self) -> HostMarkerListSnapshot {
        let Some(road_map) = self.state.road_map.as_deref() else {
            return HostMarkerListSnapshot {
                markers: Vec::new(),
                groups: Vec::new(),
            };
        };

        let mut groups = BTreeSet::new();
        let mut markers: Vec<HostMarkerInfo> = road_map
            .map_markers()
            .iter()
            .filter_map(|marker| {
                let node = road_map.node(marker.id)?;
                groups.insert(marker.group.clone());

                Some(HostMarkerInfo {
                    node_id: marker.id,
                    name: marker.name.clone(),
                    group: marker.group.clone(),
                    marker_index: marker.marker_index,
                    is_debug: marker.is_debug,
                    position: [node.position.x, node.position.y],
                })
            })
            .collect();
        markers.sort_by_key(|marker| marker.marker_index);

        HostMarkerListSnapshot {
            markers,
            groups: groups.into_iter().collect(),
        }
    }

    fn build_connection_pair_snapshot(
        &self,
        node_a: u64,
        node_b: u64,
    ) -> HostConnectionPairSnapshot {
        let connections = self
            .state
            .road_map
            .as_deref()
            .map(|road_map| {
                road_map
                    .find_connections_between(node_a, node_b)
                    .into_iter()
                    .map(|connection| HostConnectionPairEntry {
                        start_id: connection.start_id,
                        end_id: connection.end_id,
                        direction: map_connection_direction(connection.direction),
                        priority: map_connection_priority(connection.priority),
                    })
                    .collect()
            })
            .unwrap_or_default();

        HostConnectionPairSnapshot {
            node_a,
            node_b,
            connections,
        }
    }

    fn rebuild_snapshot_if_dirty(&mut self) {
        if !self.snapshot_dirty {
            return;
        }

        self.snapshot_cache = build_snapshot(
            &self.state,
            &self.chrome_state,
            self.pending_dialog_requests.len(),
        );
        self.snapshot_dirty = false;
    }

    /// Verarbeitet ausstehende Engine-Requests: Chrome-Varianten werden in
    /// `chrome_state` ausgefuehrt, `PickPath`-Varianten in `pending_dialog_requests`
    /// gepuffert (fuer spaeteres `take_dialog_requests()`).
    fn drain_engine_requests(&mut self) {
        let requests = self.controller.take_dialog_requests(&mut self.state);
        let mut chrome_dirty = false;
        for req in requests {
            match req {
                DialogRequest::ToggleCommandPalette => {
                    self.chrome_state.show_command_palette =
                        !self.chrome_state.show_command_palette;
                    chrome_dirty = true;
                }
                DialogRequest::OpenOptionsDialog => {
                    self.chrome_state.show_options_dialog = true;
                    chrome_dirty = true;
                }
                DialogRequest::CloseOptionsDialog => {
                    self.chrome_state.show_options_dialog = false;
                    chrome_dirty = true;
                }
                DialogRequest::ShowHeightmapWarning => {
                    self.chrome_state.show_heightmap_warning = true;
                    chrome_dirty = true;
                }
                DialogRequest::DismissHeightmapWarning => {
                    self.chrome_state.show_heightmap_warning = false;
                    chrome_dirty = true;
                }
                DialogRequest::ShowDissolveGroupConfirm(id) => {
                    self.chrome_state.confirm_dissolve_group_id = Some(id);
                    chrome_dirty = true;
                }
                DialogRequest::PickPath {
                    kind,
                    suggested_file_name,
                } => {
                    self.pending_dialog_requests
                        .push(crate::dispatch::map_engine_dialog_request(
                            DialogRequest::PickPath {
                                kind,
                                suggested_file_name,
                            },
                        ));
                }
            }
        }
        if chrome_dirty {
            self.chrome_state.mark_dirty();
            self.snapshot_dirty = true;
        }
    }

    /// Spiegelt Engine-UI-Request-Flags in den host-lokalen Chrome-State.
    ///
    /// Wird nach jedem `apply_action()`/`apply_intent()` aufgerufen, damit
    /// `chrome_state` immer die aktuellen Engine-Werte fuer sichtbarkeits-relevante
    /// Felder enthaelt. Fuer Dialoge mit nutzer-mutierbaren Daten wird ein
    /// Transition-basiertes Sync verwendet: Beim Oeffen werden Daten kopiert,
    /// waehrend der Dialog offen ist wird der `chrome_state` NICHT ueberschrieben.
    fn sync_chrome_from_engine(&mut self) {
        let ui = &self.state.ui;

        // show_command_palette, show_options_dialog, show_heightmap_warning und
        // confirm_dissolve_group_id werden nach dem Drain-Refactoring nicht mehr als
        // Flags in EngineUiState gehalten, sondern als DialogRequest-Events emittiert
        // und bereits in drain_engine_requests() direkt in chrome_state verarbeitet.
        let new_hwconf = ui.heightmap_warning_confirmed;

        let mut dirty = false;

        if self.chrome_state.heightmap_warning_confirmed != new_hwconf {
            self.chrome_state.heightmap_warning_confirmed = new_hwconf;
            dirty = true;
        }

        // Dedup-Dialog: read-only im Frontend → immer spiegeln
        if self.chrome_state.dedup_dialog.visible != ui.dedup_dialog.visible
            || self.chrome_state.dedup_dialog.duplicate_count != ui.dedup_dialog.duplicate_count
            || self.chrome_state.dedup_dialog.group_count != ui.dedup_dialog.group_count
        {
            self.chrome_state.dedup_dialog = ui.dedup_dialog.clone();
            dirty = true;
        }

        // Save-Overview-Dialog: kein mutierbares Nutzerfeld → immer spiegeln
        if self.chrome_state.save_overview_dialog.visible != ui.save_overview_dialog.visible {
            self.chrome_state.save_overview_dialog = ui.save_overview_dialog.clone();
            dirty = true;
        }

        // Group-Settings-Popup: einfacher Trigger − beim Oeffen/Schliessen spiegeln
        if self.chrome_state.group_settings_popup.visible != ui.group_settings_popup.visible {
            self.chrome_state.group_settings_popup = ui.group_settings_popup.clone();
            dirty = true;
        }

        // Dialoge mit Nutzer-mutierbaren Feldern: nur beim Oeffen (Transition false→true)
        // kopieren; waehrend offen NICHT ueberschreiben.
        if ui.marker_dialog.visible && !self.chrome_state.marker_dialog.visible {
            self.chrome_state.marker_dialog = ui.marker_dialog.clone();
            dirty = true;
        } else if !ui.marker_dialog.visible && self.chrome_state.marker_dialog.visible {
            self.chrome_state.marker_dialog.visible = false;
            dirty = true;
        }

        if ui.trace_all_fields_dialog.visible && !self.chrome_state.trace_all_fields_dialog.visible
        {
            self.chrome_state.trace_all_fields_dialog = ui.trace_all_fields_dialog.clone();
            dirty = true;
        } else if !ui.trace_all_fields_dialog.visible
            && self.chrome_state.trace_all_fields_dialog.visible
        {
            self.chrome_state.trace_all_fields_dialog.visible = false;
            dirty = true;
        }

        if ui.overview_options_dialog.visible && !self.chrome_state.overview_options_dialog.visible
        {
            self.chrome_state.overview_options_dialog = ui.overview_options_dialog.clone();
            dirty = true;
        } else if !ui.overview_options_dialog.visible
            && self.chrome_state.overview_options_dialog.visible
        {
            self.chrome_state.overview_options_dialog.visible = false;
            dirty = true;
        }

        if ui.post_load_dialog.visible && !self.chrome_state.post_load_dialog.visible {
            self.chrome_state.post_load_dialog = ui.post_load_dialog.clone();
            dirty = true;
        } else if !ui.post_load_dialog.visible && self.chrome_state.post_load_dialog.visible {
            self.chrome_state.post_load_dialog.visible = false;
            dirty = true;
        }

        // ZIP-Browser: Option<ZipBrowserState> — beim Oeffnen kopieren
        let engine_zip_open = ui.zip_browser.is_some();
        let chrome_zip_open = self.chrome_state.zip_browser.is_some();
        if engine_zip_open && !chrome_zip_open {
            self.chrome_state.zip_browser = ui.zip_browser.clone();
            dirty = true;
        } else if !engine_zip_open && chrome_zip_open {
            self.chrome_state.zip_browser = None;
            dirty = true;
        }

        if dirty {
            self.chrome_state.mark_dirty();
        }
    }
}

impl Default for HostBridgeSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    use fs25_auto_drive_engine::app::handlers;
    use fs25_auto_drive_engine::app::tool_contract::RouteToolId;
    use fs25_auto_drive_engine::app::{
        AppIntent, Connection, ConnectionDirection, ConnectionPriority, FloatingMenuKind,
        GroupEditState, GroupRecord, MapMarker, MapNode, NodeFlag, OverviewSourceContext, RoadMap,
        ZipBrowserState,
    };
    use fs25_auto_drive_engine::core::ZipImageEntry;
    use fs25_auto_drive_engine::shared::{OverviewFieldDetectionSource, OverviewLayerOptions};
    use glam::Vec2;
    use std::sync::Arc;

    use crate::dto::{
        HostFieldDetectionSource, HostOverviewLayersSnapshot, HostOverviewOptionsDialogSnapshot,
        HostOverviewSourceContext, HostResampleMode, HostRouteToolId,
    };

    use crate::dto::{
        EngineSessionAction, HostActiveTool, HostConnectionPairEntry, HostConnectionPairSnapshot,
        HostDefaultConnectionDirection, HostDefaultConnectionPriority, HostDialogRequestKind,
        HostDialogResult, HostInputModifiers, HostMarkerListSnapshot, HostNodeDetails,
        HostPointerButton, HostSessionAction, HostTapKind, HostViewportInputBatch,
        HostViewportInputEvent,
    };

    use super::{EngineRenderFrameSnapshot, FlutterBridgeSession, HostBridgeSession};

    fn apply_test_intent(session: &mut HostBridgeSession, intent: AppIntent) {
        session
            .controller
            .handle_intent(&mut session.state, intent)
            .expect("Test-Intent muss verarbeitet werden");
        session.snapshot_dirty = true;
        session.drain_engine_requests();
        session.sync_chrome_from_engine();
    }

    fn viewport_test_map() -> RoadMap {
        let mut map = RoadMap::new(2);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(20.0, 0.0), NodeFlag::Regular));
        map.ensure_spatial_index();
        map
    }

    fn viewport_connected_path_map() -> RoadMap {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        ));
        map.add_connection(Connection::new(
            2,
            3,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
        ));
        map.ensure_spatial_index();
        map
    }

    fn node_details_marker_test_map() -> RoadMap {
        let mut map = viewport_connected_path_map();
        map.add_map_marker(MapMarker::new(
            2,
            "Hof".to_string(),
            "All".to_string(),
            3,
            false,
        ));
        map
    }

    fn group_boundary_test_map() -> RoadMap {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(10, Vec2::new(-10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(11, Vec2::new(30.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        ));
        map.add_connection(Connection::new(
            2,
            3,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
        ));
        map.add_connection(Connection::new(
            10,
            1,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(-10.0, 0.0),
            Vec2::new(0.0, 0.0),
        ));
        map.add_connection(Connection::new(
            3,
            11,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(20.0, 0.0),
            Vec2::new(30.0, 0.0),
        ));
        map.ensure_spatial_index();
        map
    }

    fn make_group_record(record_id: u64, node_ids: &[u64], road_map: &RoadMap) -> GroupRecord {
        GroupRecord {
            id: record_id,
            node_ids: node_ids.to_vec(),
            original_positions: node_ids
                .iter()
                .filter_map(|node_id| road_map.node(*node_id).map(|node| node.position))
                .collect(),
            marker_node_ids: Vec::new(),
            locked: false,
            entry_node_id: Some(1),
            exit_node_id: Some(3),
        }
    }

    fn snapshot_measurement_session(selected_count: usize) -> HostBridgeSession {
        let mut session = HostBridgeSession::new();
        let mut map = RoadMap::new(3);

        for id in 1..=selected_count as u64 {
            let x = id as f32;
            map.add_node(MapNode::new(id, Vec2::new(x, x * 0.25), NodeFlag::Regular));
            session.state.selection.ids_mut().insert(id);
        }

        session.state.road_map = Some(Arc::new(map));
        session.state.view.viewport_size = [1280.0, 720.0];
        session.snapshot_dirty = true;
        let _ = session.snapshot();
        session
    }

    fn resize_event(size_px: [f32; 2]) -> HostViewportInputEvent {
        HostViewportInputEvent::Resize { size_px }
    }

    fn tap_event(screen_pos: [f32; 2]) -> HostViewportInputEvent {
        HostViewportInputEvent::Tap {
            button: HostPointerButton::Primary,
            tap_kind: HostTapKind::Single,
            screen_pos,
            modifiers: HostInputModifiers::default(),
        }
    }

    fn double_tap_event(screen_pos: [f32; 2], additive: bool) -> HostViewportInputEvent {
        HostViewportInputEvent::Tap {
            button: HostPointerButton::Primary,
            tap_kind: HostTapKind::Double,
            screen_pos,
            modifiers: HostInputModifiers {
                shift: false,
                alt: false,
                command: additive,
            },
        }
    }

    fn screen_for_world(session: &HostBridgeSession, world_pos: Vec2) -> [f32; 2] {
        let viewport = session.state.view.viewport_size;
        let screen = session
            .state
            .view
            .camera
            .world_to_screen(world_pos, Vec2::new(viewport[0], viewport[1]));
        [screen.x, screen.y]
    }

    #[test]
    fn new_session_exposes_empty_snapshot() {
        let mut session = HostBridgeSession::new();
        let snapshot = session.snapshot();

        assert!(!snapshot.has_map);
        assert_eq!(snapshot.node_count, 0);
        assert_eq!(snapshot.connection_count, 0);
        assert_eq!(snapshot.active_tool, HostActiveTool::Select);
        assert!(!snapshot.can_undo);
        assert!(!snapshot.can_redo);
        assert_eq!(snapshot.pending_dialog_request_count, 0);
        assert!(snapshot.selection.selected_node_ids.is_empty());
    }

    #[test]
    fn dispatch_updates_snapshot_state() {
        let mut session = HostBridgeSession::new();

        session
            .apply_action(HostSessionAction::ToggleCommandPalette)
            .expect("ToggleCommandPalette muss funktionieren");

        let snapshot = session.snapshot();
        assert!(snapshot.show_command_palette);
    }

    #[test]
    fn active_tool_uses_explicit_stable_snapshot_identifier() {
        let mut session = HostBridgeSession::new();

        session
            .set_editor_tool(HostActiveTool::Route)
            .expect("SetEditorToolRequested muss funktionieren");

        let snapshot = session.snapshot();
        assert_eq!(snapshot.active_tool, HostActiveTool::Route);
    }

    #[test]
    fn options_dialog_visibility_is_controlled_via_explicit_actions() {
        let mut session = HostBridgeSession::new();

        session
            .set_options_dialog_visible(true)
            .expect("OpenOptionsDialog muss funktionieren");
        assert!(session.snapshot().show_options_dialog);

        session
            .set_options_dialog_visible(false)
            .expect("CloseOptionsDialog muss funktionieren");
        assert!(!session.snapshot().show_options_dialog);
    }

    #[test]
    fn undo_and_redo_actions_are_available_via_explicit_surface() {
        let mut session = HostBridgeSession::new();

        session.undo().expect("Undo muss verfuegbar sein");
        session.redo().expect("Redo muss verfuegbar sein");

        let snapshot = session.snapshot();
        assert!(!snapshot.can_undo);
        assert!(!snapshot.can_redo);
    }

    #[test]
    fn take_dialog_requests_drains_pending_queue_for_host_polling() {
        let mut session = HostBridgeSession::new();

        apply_test_intent(&mut session, AppIntent::CurseplayImportRequested);
        apply_test_intent(&mut session, AppIntent::CurseplayExportRequested);

        assert_eq!(session.snapshot().pending_dialog_request_count, 2);

        let requests = session.take_dialog_requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].kind, HostDialogRequestKind::CurseplayImport);
        assert_eq!(requests[1].kind, HostDialogRequestKind::CurseplayExport);
        assert_eq!(session.snapshot().pending_dialog_request_count, 0);
    }

    #[test]
    fn submit_dialog_result_roundtrips_heightmap_path_selected_into_state() {
        let mut session = HostBridgeSession::new();

        session
            .apply_action(HostSessionAction::RequestHeightmapSelection)
            .expect("RequestHeightmapSelection muss einen Host-Dialog anfordern");
        assert_eq!(session.snapshot().pending_dialog_request_count, 1);

        let requests = session.take_dialog_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].kind, HostDialogRequestKind::Heightmap);
        assert_eq!(session.snapshot().pending_dialog_request_count, 0);

        let selected_path = "/tmp/test_heightmap.png".to_string();
        session
            .submit_dialog_result(HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::Heightmap,
                path: selected_path.clone(),
            })
            .expect("PathSelected muss ueber die gemeinsame Dispatch-Seam verarbeitet werden");

        assert_eq!(session.state.ui.heightmap_path, Some(selected_path));
    }

    #[test]
    fn render_accessors_expose_scene_and_assets_without_state_leaks() {
        let session = HostBridgeSession::new();

        let scene = session.build_render_scene([800.0, 600.0]);
        let assets = session.build_render_assets();
        let frame = session.build_render_frame([320.0, 240.0]);

        assert!(!scene.has_map());
        assert_eq!(assets.background_asset_revision(), 0);
        assert!(assets.background().is_none());
        assert_eq!(frame.scene.viewport_size(), [320.0, 240.0]);
        assert_eq!(frame.assets.background_transform_revision(), 0);
    }

    #[test]
    fn host_ui_and_overlay_snapshots_are_available() {
        let mut session = HostBridgeSession::new();

        let host_ui = session.build_host_ui_snapshot();
        let chrome = session.build_host_chrome_snapshot();
        assert!(host_ui.command_palette_state().is_some());
        assert!(host_ui.options_panel_state().is_some());
        assert!(!chrome.has_map);
        assert_eq!(chrome.active_tool, HostActiveTool::Select);

        let overlay = session.build_viewport_overlay_snapshot(None);
        assert!(overlay.route_tool_preview.is_none());
        assert!(overlay.group_boundaries.is_empty());
    }

    #[test]
    fn node_details_read_is_typed_and_side_effect_free() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(node_details_marker_test_map()));

        let details = session
            .node_details(2)
            .expect("Node-Details muessen fuer vorhandenen Node lesbar sein");

        assert_eq!(details.id, 2);
        assert_eq!(details.position, [10.0, 0.0]);
        assert_eq!(details.neighbors.len(), 2);
        assert!(details
            .neighbors
            .iter()
            .any(|neighbor| { neighbor.neighbor_id == 1 && !neighbor.is_outgoing }));
        assert!(details
            .neighbors
            .iter()
            .any(|neighbor| { neighbor.neighbor_id == 3 && neighbor.is_outgoing }));
        assert_eq!(
            details.marker,
            Some(crate::dto::HostNodeMarkerInfo {
                name: "Hof".to_string(),
                group: "All".to_string(),
            })
        );
        assert_eq!(session.inspected_node_id(), None);
    }

    #[test]
    fn node_details_read_returns_none_for_unknown_node_id() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(node_details_marker_test_map()));

        assert_eq!(session.node_details(999), None);
    }

    #[test]
    fn node_details_json_serializes_current_inspected_node_via_typed_read_seam() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(node_details_marker_test_map()));
        session.set_inspected_node_id(Some(2));

        let expected = session
            .node_details(2)
            .expect("Typed Node-Details muessen verfuegbar sein");
        let payload = session
            .node_details_json()
            .expect("JSON-Node-Details muessen fuer inspizierten Node serialisierbar sein");
        let parsed: HostNodeDetails = serde_json::from_str(&payload)
            .expect("Node-Details-JSON muss wieder in das DTO lesbar sein");

        assert_eq!(parsed, expected);
        assert_eq!(session.inspected_node_id(), Some(2));
    }

    #[test]
    fn marker_list_typed_read_and_json_share_the_same_snapshot() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(node_details_marker_test_map()));

        let snapshot = session.marker_list();
        let parsed: HostMarkerListSnapshot = serde_json::from_str(&session.marker_list_json())
            .expect("Marker-List-JSON muss wieder in das DTO lesbar sein");

        assert_eq!(snapshot, parsed);
        assert_eq!(snapshot.groups, vec!["All".to_string()]);
        assert_eq!(snapshot.markers.len(), 1);
        assert_eq!(snapshot.markers[0].node_id, 2);
        assert_eq!(snapshot.markers[0].name, "Hof");
    }

    #[test]
    fn marker_list_read_returns_empty_snapshot_for_empty_road_map() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(RoadMap::new(2)));

        let snapshot = session.marker_list();

        assert!(snapshot.markers.is_empty());
        assert!(snapshot.groups.is_empty());
    }

    #[test]
    fn connection_pair_read_returns_bridge_snapshot_for_two_nodes() {
        let mut session = HostBridgeSession::new();
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        ));
        map.add_connection(Connection::new(
            2,
            1,
            ConnectionDirection::Reverse,
            ConnectionPriority::SubPriority,
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 0.0),
        ));
        session.state.road_map = Some(Arc::new(map));

        let snapshot = session.connection_pair(1, 2);

        assert_eq!(
            snapshot,
            HostConnectionPairSnapshot {
                node_a: 1,
                node_b: 2,
                connections: vec![
                    HostConnectionPairEntry {
                        start_id: 1,
                        end_id: 2,
                        direction: HostDefaultConnectionDirection::Dual,
                        priority: HostDefaultConnectionPriority::Regular,
                    },
                    HostConnectionPairEntry {
                        start_id: 2,
                        end_id: 1,
                        direction: HostDefaultConnectionDirection::Reverse,
                        priority: HostDefaultConnectionPriority::SubPriority,
                    },
                ],
            }
        );
    }

    #[test]
    fn connection_pair_read_returns_empty_connections_for_unconnected_nodes() {
        let mut session = HostBridgeSession::new();
        let mut map = RoadMap::new(2);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        session.state.road_map = Some(Arc::new(map));

        let snapshot = session.connection_pair(1, 2);

        assert_eq!(
            snapshot,
            HostConnectionPairSnapshot {
                node_a: 1,
                node_b: 2,
                connections: Vec::new(),
            }
        );
    }

    #[test]
    fn should_exit_surfaces_explicit_exit_seam() {
        let mut session = HostBridgeSession::new();

        assert!(!session.should_exit());

        session.state.should_exit = true;

        assert!(session.should_exit());
    }

    #[test]
    fn session_dirty_state_surfaces_via_snapshot() {
        let mut session = HostBridgeSession::new();
        let sample_path = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../ad_sample_data/AutoDrive_config-test.xml"
        ));

        fs25_auto_drive_engine::app::use_cases::file_io::load_selected_file(
            &mut session.state,
            sample_path.to_string_lossy().into_owned(),
        )
        .expect("Beispiel-XML muss fuer Dirty-Tracking ladbar sein");

        session.snapshot_dirty = true;
        assert!(!session.is_dirty());
        assert!(!session.snapshot().is_dirty);

        Arc::make_mut(
            session
                .state
                .road_map
                .as_mut()
                .expect("RoadMap muss nach dem Laden vorhanden sein"),
        )
        .add_node(MapNode::new(
            999_999,
            Vec2::new(1.0, 1.0),
            NodeFlag::Regular,
        ));

        session.snapshot_dirty = true;
        assert!(session.is_dirty());
        assert!(session.snapshot().is_dirty);
    }

    #[test]
    fn read_only_host_snapshots_do_not_mark_session_snapshot_dirty() {
        let session = snapshot_measurement_session(32);

        let _ = session.build_host_ui_snapshot();
        let _ = session.build_host_chrome_snapshot();
        let _ = session.editing_snapshot();
        let _ = session.build_route_tool_viewport_snapshot();
        let _ = session.build_viewport_geometry_snapshot([640.0, 480.0]);

        assert!(
            !session.snapshot_dirty,
            "Read-only Snapshot-Builder duerfen den Session-Cache nicht dirty markieren"
        );
    }

    #[test]
    fn local_ui_seams_do_not_mark_snapshot_dirty_for_local_state_reads() {
        let mut session = snapshot_measurement_session(32);

        {
            let panel_state = session.panel_properties_state_mut();
            assert_eq!(panel_state.selected_node_ids.len(), 32);
        }
        assert!(!session.snapshot_dirty);

        {
            let dialog_state = session.dialog_ui_state_mut();
            assert!(!dialog_state.ui.show_options_dialog);
        }
        assert!(!session.snapshot_dirty);

        {
            let viewport_state = session.viewport_input_context_mut();
            assert_eq!(viewport_state.selected_node_ids.len(), 32);
        }
        assert!(!session.snapshot_dirty);
    }

    #[test]
    fn dialog_ui_mutation_marks_snapshot_dirty_without_manual_invalidation() {
        let mut session = snapshot_measurement_session(32);
        let _ = session.snapshot_owned();
        assert!(!session.snapshot_dirty);

        {
            // HostLocalDialogState-Mutation: show_command_palette setzen.
            // Diese Mutation liegt im host-lokalen Chrome-State, nicht im Engine-State.
            let dialog_state = session.dialog_ui_state_mut();
            dialog_state.ui.show_command_palette = true;
        }
        assert!(
            session.snapshot_dirty,
            "Snapshot-relevante Dialog-Mutationen muessen den Session-Cache invalidieren"
        );

        let snapshot = session.snapshot_owned();
        assert!(snapshot.show_command_palette);
        assert!(!session.snapshot_dirty);
    }

    #[test]
    fn chrome_state_mutation_marks_snapshot_dirty_without_manual_invalidation() {
        let mut session = snapshot_measurement_session(32);
        let _ = session.snapshot_owned();
        assert!(!session.snapshot_dirty);

        {
            let chrome_state = session.chrome_state_mut();
            chrome_state.show_options_dialog = true;
        }

        assert!(
            session.snapshot_dirty,
            "chrome_state_mut muss stale Snapshots durch automatische Invalidation verhindern"
        );
        assert!(session.snapshot().show_options_dialog);
        assert!(!session.snapshot_dirty);
    }

    #[test]
    fn apply_intent_syncs_host_local_overview_options_before_generation() {
        let mut session = HostBridgeSession::new();
        let zip_path = "/tmp/host_bridge_overview_sync.zip".to_string();
        let expected_layers = OverviewLayerOptions {
            terrain: false,
            hillshade: false,
            farmlands: false,
            farmland_ids: true,
            pois: true,
            legend: true,
        };

        session
            .apply_intent(AppIntent::GenerateOverviewFromZip {
                path: zip_path.clone(),
            })
            .expect("Overview-Dialog muss geoeffnet werden");

        session.update_overview_options_dialog(HostOverviewOptionsDialogSnapshot {
            visible: true,
            zip_path: zip_path.clone(),
            layers: HostOverviewLayersSnapshot {
                terrain: expected_layers.terrain,
                hillshade: expected_layers.hillshade,
                farmlands: expected_layers.farmlands,
                farmland_ids: expected_layers.farmland_ids,
                pois: expected_layers.pois,
                legend: expected_layers.legend,
            },
            field_detection_source: HostFieldDetectionSource::GroundGdm,
            available_sources: vec![
                HostFieldDetectionSource::FromZip,
                HostFieldDetectionSource::GroundGdm,
            ],
        });

        assert!(session.chrome_state.chrome_dirty);
        assert_eq!(
            session
                .chrome_state
                .overview_options_dialog
                .field_detection_source,
            OverviewFieldDetectionSource::GroundGdm
        );
        assert!(
            session
                .app_state()
                .ui
                .overview_options_dialog
                .layers
                .terrain
        );

        let error = session
            .apply_intent(AppIntent::OverviewOptionsConfirmed)
            .expect_err("Fehlendes ZIP muss die Generierung scheitern lassen");

        assert!(
            error.to_string().contains(zip_path.as_str()),
            "Fehlermeldung soll den konfigurierten ZIP-Pfad referenzieren"
        );
        assert_eq!(session.app_state().options.overview_layers, expected_layers);
        assert_eq!(
            session.app_state().ui.overview_options_dialog.layers,
            expected_layers
        );
        assert_eq!(
            session.app_state().options.overview_field_detection_source,
            OverviewFieldDetectionSource::GroundGdm
        );
        assert_eq!(
            session
                .app_state()
                .ui
                .overview_options_dialog
                .field_detection_source,
            OverviewFieldDetectionSource::GroundGdm
        );
    }

    #[test]
    fn generate_overview_from_zip_closes_source_dialog_and_opens_options_dialog() {
        let mut session = HostBridgeSession::new();
        let zip_path = "/tmp/host_bridge_overview_source.zip".to_string();

        session
            .apply_intent(AppIntent::GenerateOverviewRequested)
            .expect("Source-Dialog muss geoeffnet werden");

        assert!(session.chrome_state.post_load_dialog.visible);
        assert_eq!(
            session.chrome_state.post_load_dialog.context,
            OverviewSourceContext::ManualMenu
        );

        session
            .apply_intent(AppIntent::GenerateOverviewFromZip {
                path: zip_path.clone(),
            })
            .expect("ZIP-Auswahl muss den Options-Dialog oeffnen");

        assert!(!session.chrome_state.post_load_dialog.visible);
        assert!(!session.app_state().ui.post_load_dialog.visible);
        assert!(session.chrome_state.overview_options_dialog.visible);
        assert!(session.app_state().ui.overview_options_dialog.visible);
        assert_eq!(
            session.app_state().ui.overview_options_dialog.zip_path,
            zip_path
        );
    }

    #[test]
    fn dialog_snapshot_reflects_host_local_dialog_state() {
        let mut session = HostBridgeSession::new();

        {
            let dialog_state = session.dialog_ui_state_mut();
            dialog_state.ui.show_heightmap_warning = true;
            dialog_state.ui.heightmap_warning_confirmed = true;
            dialog_state.ui.marker_dialog.visible = true;
            dialog_state.ui.marker_dialog.node_id = Some(17);
            dialog_state.ui.marker_dialog.name = "Hof".to_string();
            dialog_state.ui.marker_dialog.group = "All".to_string();
            dialog_state.ui.marker_dialog.is_new = false;
            dialog_state.ui.dedup_dialog.visible = true;
            dialog_state.ui.dedup_dialog.duplicate_count = 3;
            dialog_state.ui.dedup_dialog.group_count = 2;
            dialog_state.ui.zip_browser = Some(ZipBrowserState {
                zip_path: "/tmp/map.zip".to_string(),
                entries: vec![ZipImageEntry {
                    name: "overview.png".to_string(),
                    size: 4096,
                }],
                selected: Some(0),
                filter_overview: true,
            });
            dialog_state.ui.overview_options_dialog.visible = true;
            dialog_state.ui.overview_options_dialog.zip_path = "/tmp/map.zip".to_string();
            dialog_state.ui.overview_options_dialog.layers = OverviewLayerOptions {
                terrain: false,
                hillshade: false,
                farmlands: true,
                farmland_ids: true,
                pois: false,
                legend: true,
            };
            dialog_state
                .ui
                .overview_options_dialog
                .field_detection_source = OverviewFieldDetectionSource::ZipGroundGdm;
            dialog_state.ui.overview_options_dialog.available_sources = vec![
                OverviewFieldDetectionSource::FromZip,
                OverviewFieldDetectionSource::ZipGroundGdm,
            ];
            dialog_state.ui.post_load_dialog.visible = true;
            dialog_state.ui.post_load_dialog.context = OverviewSourceContext::PostLoadDetected;
            dialog_state.ui.post_load_dialog.heightmap_set = true;
            dialog_state.ui.post_load_dialog.heightmap_path = Some("/tmp/terrain.png".to_string());
            dialog_state.ui.post_load_dialog.overview_loaded = true;
            dialog_state.ui.post_load_dialog.matching_zips = vec![PathBuf::from("/mods/map.zip")];
            dialog_state.ui.post_load_dialog.selected_zip_index = 0;
            dialog_state.ui.post_load_dialog.map_name = "Elmcreek".to_string();
            dialog_state.ui.save_overview_dialog.visible = true;
            dialog_state.ui.save_overview_dialog.target_path = "/tmp/overview.png".to_string();
            dialog_state.ui.save_overview_dialog.is_overwrite = true;
            dialog_state.ui.trace_all_fields_dialog.visible = true;
            dialog_state.ui.trace_all_fields_dialog.spacing = 12.5;
            dialog_state.ui.trace_all_fields_dialog.offset = -1.0;
            dialog_state.ui.trace_all_fields_dialog.tolerance = 0.5;
            dialog_state
                .ui
                .trace_all_fields_dialog
                .corner_detection_enabled = true;
            dialog_state
                .ui
                .trace_all_fields_dialog
                .corner_angle_threshold_deg = 95.0;
            dialog_state
                .ui
                .trace_all_fields_dialog
                .corner_rounding_enabled = true;
            dialog_state
                .ui
                .trace_all_fields_dialog
                .corner_rounding_radius = 6.0;
            dialog_state
                .ui
                .trace_all_fields_dialog
                .corner_rounding_max_angle_deg = 18.0;
            dialog_state.ui.group_settings_popup.visible = true;
            dialog_state.ui.group_settings_popup.world_pos = Vec2::new(8.0, -4.0);
            dialog_state.ui.confirm_dissolve_group_id = Some(99);
            dialog_state.options.segment_stop_at_junction = true;
            dialog_state.options.segment_max_angle_deg = 42.5;
        }

        let snapshot = session.dialog_snapshot();

        assert!(snapshot.heightmap_warning.visible);
        assert!(snapshot.heightmap_warning.confirmed_for_current_save);
        assert_eq!(snapshot.marker_dialog.node_id, Some(17));
        assert_eq!(snapshot.marker_dialog.name, "Hof");
        assert_eq!(snapshot.dedup_dialog.duplicate_count, 3);
        assert!(snapshot.zip_browser.visible);
        assert_eq!(snapshot.zip_browser.entries.len(), 1);
        assert_eq!(snapshot.zip_browser.entries[0].name, "overview.png");
        assert!(!snapshot.overview_options_dialog.layers.terrain);
        assert_eq!(
            snapshot.overview_options_dialog.field_detection_source,
            HostFieldDetectionSource::ZipGroundGdm
        );
        assert_eq!(
            snapshot.post_load_dialog.context,
            HostOverviewSourceContext::PostLoadDetected
        );
        assert_eq!(
            snapshot.post_load_dialog.matching_zip_paths,
            vec!["/mods/map.zip".to_string()]
        );
        assert_eq!(snapshot.group_settings_popup.world_pos, [8.0, -4.0]);
        assert!(snapshot.group_settings_popup.segment_stop_at_junction);
        assert_eq!(snapshot.group_settings_popup.segment_max_angle_deg, 42.5);
        assert_eq!(snapshot.confirm_dissolve_group.segment_id, Some(99));
        assert!(snapshot.confirm_dissolve_group.visible);
    }

    #[test]
    fn editing_snapshot_reports_resample_metrics_for_connected_chain() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(viewport_connected_path_map()));
        session.state.selection.ids_mut().insert(1);
        session.state.selection.ids_mut().insert(2);
        session.state.selection.ids_mut().insert(3);
        session.state.ui.distanzen.active = true;
        session.state.ui.distanzen.by_count = true;
        session.state.ui.distanzen.count = 5;
        session.state.ui.distanzen.distance = 4.0;
        session.state.ui.distanzen.hide_original = true;

        let snapshot = session.editing_snapshot();

        assert!(snapshot.resample.active);
        assert!(snapshot.resample.can_resample_current_selection);
        assert_eq!(snapshot.resample.selected_node_count, 3);
        assert_eq!(snapshot.resample.mode, HostResampleMode::Count);
        assert_eq!(snapshot.resample.count, 5);
        assert_eq!(snapshot.resample.preview_count, 5);
        assert!((snapshot.resample.path_length - 20.0).abs() < 0.01);
    }

    #[test]
    fn editing_snapshot_reports_group_edit_boundary_candidates() {
        let mut session = HostBridgeSession::new();
        let road_map = group_boundary_test_map();
        let record_id = 42;
        let record = make_group_record(record_id, &[1, 2, 3], &road_map);

        session.state.road_map = Some(Arc::new(road_map));
        session.state.group_registry.register(record);
        session.state.group_editing = Some(GroupEditState {
            record_id,
            was_locked: true,
        });

        let snapshot = session.editing_snapshot();
        let group_edit = snapshot
            .group_edit
            .expect("Group-Edit-Snapshot muss vorhanden sein");

        assert_eq!(group_edit.record_id, record_id);
        assert!(!group_edit.locked);
        assert!(group_edit.was_locked_before_edit);
        assert_eq!(group_edit.entry_node_id, Some(1));
        assert_eq!(group_edit.exit_node_id, Some(3));
        assert_eq!(group_edit.boundary_candidates.len(), 3);

        let entry_candidate = group_edit
            .boundary_candidates
            .iter()
            .find(|candidate| candidate.node_id == 1)
            .expect("Entry-Kandidat muss enthalten sein");
        assert!(entry_candidate.has_external_incoming);
        assert!(!entry_candidate.has_external_outgoing);

        let middle_candidate = group_edit
            .boundary_candidates
            .iter()
            .find(|candidate| candidate.node_id == 2)
            .expect("Mittelknoten muss enthalten sein");
        assert!(!middle_candidate.has_external_incoming);
        assert!(!middle_candidate.has_external_outgoing);

        let exit_candidate = group_edit
            .boundary_candidates
            .iter()
            .find(|candidate| candidate.node_id == 3)
            .expect("Exit-Kandidat muss enthalten sein");
        assert!(!exit_candidate.has_external_incoming);
        assert!(exit_candidate.has_external_outgoing);
    }

    #[test]
    fn editing_snapshot_reports_tool_editable_groups_for_persisted_route_tool() {
        let mut session = HostBridgeSession::new();
        let mut road_map = RoadMap::new(3);
        road_map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(2, Vec2::new(20.0, 0.0), NodeFlag::Regular));
        road_map.ensure_spatial_index();
        session.state.road_map = Some(Arc::new(road_map));

        handlers::route_tool::select_with_anchors(&mut session.state, RouteToolId::Straight, 1, 2);

        let record = session
            .state
            .group_registry
            .records()
            .next()
            .expect("Persistierter Straight-Record muss vorhanden sein")
            .clone();
        session.state.selection.ids_mut().clear();
        if let Some(&first_group_node) = record.node_ids.first() {
            session.state.selection.ids_mut().insert(first_group_node);
        }

        let snapshot = session.editing_snapshot();

        assert_eq!(snapshot.editable_groups.len(), 1);
        assert_eq!(snapshot.editable_groups[0].record_id, record.id);
        assert!(snapshot.editable_groups[0].has_tool_edit);
        assert_eq!(
            snapshot.editable_groups[0].tool_id,
            Some(HostRouteToolId::Straight)
        );
    }

    #[test]
    fn snapshot_measurement_clean_poll_reports_zero_rebuild_candidates() {
        let mut session = snapshot_measurement_session(1024);
        let iterations = 256usize;
        let start = Instant::now();
        let mut rebuild_candidates = 0usize;

        for _ in 0..iterations {
            rebuild_candidates += usize::from(session.snapshot_dirty);
            black_box(session.snapshot_owned());
        }

        let elapsed_us_per_iter = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
        eprintln!(
            "snapshot_measurement_clean_poll selected_nodes=1024 iterations={iterations} rebuild_candidates={rebuild_candidates} elapsed_us_per_iter={elapsed_us_per_iter:.3}"
        );

        assert_eq!(rebuild_candidates, 0);
        assert!(!session.snapshot_dirty);
    }

    #[test]
    fn snapshot_measurement_read_mostly_flow_reports_zero_rebuild_candidates() {
        let mut session = snapshot_measurement_session(1024);
        let iterations = 256usize;
        let start = Instant::now();
        let mut rebuild_candidates = 0usize;

        for _ in 0..iterations {
            black_box(session.build_host_ui_snapshot());
            black_box(session.build_host_chrome_snapshot());

            {
                let panel_state = session.panel_properties_state_mut();
                black_box(panel_state.selected_node_ids.len());
            }
            {
                let dialog_state = session.dialog_ui_state_mut();
                black_box(dialog_state.ui.show_options_dialog);
            }
            {
                let viewport_state = session.viewport_input_context_mut();
                black_box(viewport_state.selected_node_ids.len());
            }

            rebuild_candidates += usize::from(session.snapshot_dirty);
            black_box(session.snapshot_owned());
        }

        let elapsed_us_per_iter = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
        eprintln!(
            "snapshot_measurement_read_mostly selected_nodes=1024 iterations={iterations} rebuild_candidates={rebuild_candidates} elapsed_us_per_iter={elapsed_us_per_iter:.3}"
        );

        assert_eq!(rebuild_candidates, 0);
        assert!(!session.snapshot_dirty);
    }

    #[test]
    fn flutter_session_alias_exposes_host_bridge_session_behavior() {
        let mut session = FlutterBridgeSession::new();

        session
            .apply_action(EngineSessionAction::ToggleCommandPalette)
            .expect("ToggleCommandPalette muss ueber den Alias funktionieren");

        assert!(session.snapshot().show_command_palette);
    }

    #[test]
    fn engine_render_frame_snapshot_alias_keeps_render_contract() {
        let session = HostBridgeSession::new();

        let frame: EngineRenderFrameSnapshot = session.build_render_frame([512.0, 256.0]);

        assert_eq!(frame.scene.viewport_size(), [512.0, 256.0]);
        assert_eq!(frame.assets.background_asset_revision(), 0);
    }

    #[test]
    fn viewport_input_resize_and_scroll_zoom_update_session_view() {
        let mut session = HostBridgeSession::new();

        session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![
                        resize_event([640.0, 480.0]),
                        HostViewportInputEvent::Scroll {
                            screen_pos: Some([320.0, 240.0]),
                            smooth_delta_y: 1.0,
                            raw_delta_y: 0.0,
                            modifiers: HostInputModifiers::default(),
                        },
                    ],
                },
            })
            .expect("Resize und Scroll-Zoom muessen ueber die Session funktionieren");

        assert_eq!(session.state.view.viewport_size, [640.0, 480.0]);
        assert!(session.state.view.camera.zoom > 1.0);
    }

    #[test]
    fn viewport_input_tap_routes_to_add_node_and_connect_without_new_ffi_surface() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(viewport_test_map()));
        session.state.view.viewport_size = [800.0, 600.0];

        let add_node_screen = screen_for_world(&session, Vec2::new(200.0, 0.0));
        let node1_screen = screen_for_world(&session, Vec2::new(0.0, 0.0));
        let node2_screen = screen_for_world(&session, Vec2::new(20.0, 0.0));

        session
            .set_editor_tool(HostActiveTool::AddNode)
            .expect("AddNode-Tool muss gesetzt werden koennen");
        session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![resize_event([800.0, 600.0]), tap_event(add_node_screen)],
                },
            })
            .expect("AddNode-Tap muss verarbeitet werden");

        assert_eq!(session.state.node_count(), 3);

        session
            .set_editor_tool(HostActiveTool::Connect)
            .expect("Connect-Tool muss gesetzt werden koennen");
        session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![tap_event(node1_screen), tap_event(node2_screen)],
                },
            })
            .expect("Connect-Taps muessen verarbeitet werden");

        assert_eq!(session.state.connection_count(), 1);
    }

    #[test]
    fn viewport_input_select_rect_and_move_drag_preserve_lifecycle() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(viewport_test_map()));
        session.state.view.viewport_size = [800.0, 600.0];

        let node1_screen = screen_for_world(&session, Vec2::new(0.0, 0.0));
        let rect_end = screen_for_world(&session, Vec2::new(5.0, 5.0));

        session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![
                        resize_event([800.0, 600.0]),
                        HostViewportInputEvent::DragStart {
                            button: HostPointerButton::Primary,
                            screen_pos: node1_screen,
                            modifiers: HostInputModifiers {
                                shift: true,
                                alt: false,
                                command: false,
                            },
                        },
                        HostViewportInputEvent::DragUpdate {
                            button: HostPointerButton::Primary,
                            screen_pos: rect_end,
                            delta_px: [
                                rect_end[0] - node1_screen[0],
                                rect_end[1] - node1_screen[1],
                            ],
                        },
                        HostViewportInputEvent::DragEnd {
                            button: HostPointerButton::Primary,
                            screen_pos: Some(rect_end),
                        },
                    ],
                },
            })
            .expect("Rect-Selektion muss verarbeitet werden");

        assert_eq!(session.state.selection.selected_node_ids.len(), 1);
        assert!(session.state.selection.selected_node_ids.contains(&1));

        let node_before = session
            .state
            .road_map
            .as_ref()
            .and_then(|map| map.node(1))
            .expect("Node 1 muss vorhanden sein")
            .position;

        session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![
                        HostViewportInputEvent::DragStart {
                            button: HostPointerButton::Primary,
                            screen_pos: node1_screen,
                            modifiers: HostInputModifiers::default(),
                        },
                        HostViewportInputEvent::DragUpdate {
                            button: HostPointerButton::Primary,
                            screen_pos: [node1_screen[0] + 10.0, node1_screen[1]],
                            delta_px: [10.0, 0.0],
                        },
                        HostViewportInputEvent::DragEnd {
                            button: HostPointerButton::Primary,
                            screen_pos: Some([node1_screen[0] + 10.0, node1_screen[1]]),
                        },
                    ],
                },
            })
            .expect("Move-Drag muss verarbeitet werden");

        let node_after = session
            .state
            .road_map
            .as_ref()
            .and_then(|map| map.node(1))
            .expect("Node 1 muss nach dem Drag vorhanden sein")
            .position;

        assert!(node_after.x > node_before.x);
        assert!(session.state.can_undo());
    }

    #[test]
    fn viewport_input_alt_drag_selects_lasso_polygon_via_bridge_contract() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(viewport_connected_path_map()));
        session.state.view.viewport_size = [800.0, 600.0];

        let node1_screen = screen_for_world(&session, Vec2::new(0.0, 0.0));
        let node2_screen = screen_for_world(&session, Vec2::new(10.0, 0.0));
        let start = [node1_screen[0] - 20.0, node1_screen[1] - 20.0];
        let mid = [node2_screen[0] + 20.0, node1_screen[1] - 20.0];
        let end = [node2_screen[0] + 20.0, node1_screen[1] + 20.0];
        let close = [node1_screen[0] - 20.0, node1_screen[1] + 20.0];

        session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![
                        resize_event([800.0, 600.0]),
                        HostViewportInputEvent::DragStart {
                            button: HostPointerButton::Primary,
                            screen_pos: start,
                            modifiers: HostInputModifiers {
                                shift: false,
                                alt: true,
                                command: false,
                            },
                        },
                        HostViewportInputEvent::DragUpdate {
                            button: HostPointerButton::Primary,
                            screen_pos: mid,
                            delta_px: [mid[0] - start[0], mid[1] - start[1]],
                        },
                        HostViewportInputEvent::DragUpdate {
                            button: HostPointerButton::Primary,
                            screen_pos: end,
                            delta_px: [end[0] - mid[0], end[1] - mid[1]],
                        },
                        HostViewportInputEvent::DragEnd {
                            button: HostPointerButton::Primary,
                            screen_pos: Some(close),
                        },
                    ],
                },
            })
            .expect("Alt-Drag-Lasso muss ueber die Bridge verarbeitet werden");

        assert_eq!(session.state.selection.selected_node_ids.len(), 3);
        assert!(session.state.selection.selected_node_ids.contains(&1));
        assert!(session.state.selection.selected_node_ids.contains(&2));
        assert!(session.state.selection.selected_node_ids.contains(&3));
    }

    #[test]
    fn viewport_input_double_tap_selects_segment_via_bridge_contract() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(viewport_connected_path_map()));
        session.state.view.viewport_size = [800.0, 600.0];

        let node2_screen = screen_for_world(&session, Vec2::new(10.0, 0.0));

        session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![
                        resize_event([800.0, 600.0]),
                        double_tap_event(node2_screen, false),
                    ],
                },
            })
            .expect("Double-Tap muss ueber die Bridge verarbeitet werden");

        assert_eq!(session.state.selection.selected_node_ids.len(), 3);
        assert!(session.state.selection.selected_node_ids.contains(&1));
        assert!(session.state.selection.selected_node_ids.contains(&2));
        assert!(session.state.selection.selected_node_ids.contains(&3));
    }

    #[test]
    fn viewport_input_requires_resize_before_position_dependent_events() {
        let mut session = HostBridgeSession::new();

        let error = session
            .apply_action(HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![tap_event([10.0, 20.0])],
                },
            })
            .expect_err("Tap ohne Resize muss einen Integrationsfehler liefern");

        assert!(error
            .to_string()
            .contains("viewport input requires a positive finite viewport size"));
    }

    #[test]
    fn floating_menu_seams_toggle_and_clear_without_full_state_escape() {
        let mut session = HostBridgeSession::new();

        session.toggle_floating_menu(FloatingMenuKind::Tools, Some(Vec2::new(10.0, 20.0)));
        let tools_menu = session
            .chrome_state()
            .floating_menu
            .expect("Tools-Menue muss geoeffnet sein");
        assert_eq!(tools_menu.kind, FloatingMenuKind::Tools);
        assert_eq!(tools_menu.pos, Vec2::new(10.0, 20.0));

        session.toggle_floating_menu(FloatingMenuKind::Tools, Some(Vec2::new(30.0, 40.0)));
        assert!(session.chrome_state().floating_menu.is_none());

        session.toggle_floating_menu(FloatingMenuKind::Zoom, None);
        assert!(session.chrome_state().floating_menu.is_none());

        session.toggle_floating_menu(FloatingMenuKind::Zoom, Some(Vec2::new(5.0, 6.0)));
        assert!(session.chrome_state().floating_menu.is_some());

        session.clear_floating_menu();
        assert!(session.chrome_state().floating_menu.is_none());
    }

    #[test]
    fn viewport_geometry_snapshot_is_available_via_session_surface() {
        let session = HostBridgeSession::new();

        let geometry = session.build_viewport_geometry_snapshot([300.0, 200.0]);

        assert!(!geometry.has_map);
        assert!(geometry.nodes.is_empty());
        assert!(geometry.connections.is_empty());
        assert!(geometry.markers.is_empty());
        assert_eq!(geometry.viewport_size, [300.0, 200.0]);
    }
}
