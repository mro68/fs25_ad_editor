use anyhow::Result;
use fs25_auto_drive_engine::app::projections as engine_projections;
use fs25_auto_drive_engine::app::state::DistanzenState;
use fs25_auto_drive_engine::app::ui_contract::{
    HostUiSnapshot, PanelState, ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::{
    AppController, AppIntent, AppState, Camera2D, ConnectionDirection, ConnectionPriority,
    EditorTool, FloatingMenuKind, FloatingMenuState, GroupEditState, GroupRegistry, RoadMap,
    ToolEditStore,
};
use fs25_auto_drive_engine::shared::{EditorOptions, RenderAssetsSnapshot, RenderScene};
use glam::Vec2;
use indexmap::IndexSet;

mod chrome_state;
mod context_menu;
mod lifecycle;
mod read_models;
mod snapshots;
pub use chrome_state::HostLocalDialogState;

use crate::dispatch::HostViewportInputState;
use crate::dto::{
    HostActiveTool, HostChromeSnapshot, HostConnectionPairSnapshot, HostContextMenuSnapshot,
    HostDefaultConnectionDirection, HostDefaultConnectionPriority, HostDialogRequest,
    HostDialogResult, HostDialogSnapshot, HostEditingSnapshot, HostMarkerListSnapshot,
    HostNodeDetails, HostOverviewOptionsDialogSnapshot, HostRouteToolViewportSnapshot,
    HostSessionAction, HostSessionSnapshot, HostViewportGeometrySnapshot,
};
use snapshots::{
    build_dialog_snapshot, build_editing_snapshot, build_snapshot,
    map_host_field_detection_source_to_engine, map_host_overview_layers_to_engine,
};

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
    /// schmale UI-Seams, falls dabei ausnahmsweise Felder veraendert wurden,
    /// die in `HostSessionSnapshot` gespiegelt werden.
    pub fn mark_snapshot_dirty(&mut self) {
        self.snapshot_dirty = true;
    }

    /// Liefert eine read-only Referenz auf den host-lokalen Chrome-/Dialog-Zustand.
    pub fn chrome_state(&self) -> &HostLocalDialogState {
        &self.chrome_state
    }

    /// Liefert eine mutable Referenz auf den host-lokalen Chrome-/Dialog-Zustand.
    ///
    /// Aenderungen ueber diesen Accessor setzen automatisch `chrome_dirty` im
    /// `HostLocalDialogState`. Der Session-Snapshot wird *nicht* automatisch
    /// als dirty markiert — bei Bedarf `mark_snapshot_dirty()` aufrufen.
    pub fn chrome_state_mut(&mut self) -> &mut HostLocalDialogState {
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
    /// Der Accessor invalidiert den Session-Snapshot nicht automatisch; falls
    /// Snapshot-relevante Felder geaendert werden, muss __mark_snapshot_dirty()__
    /// explizit aufgerufen werden.
    pub fn dialog_ui_state_mut(&mut self) -> HostDialogUiState<'_> {
        HostDialogUiState {
            road_map: self.state.road_map.as_deref(),
            ui: &mut self.chrome_state,
            options: &mut self.state.options,
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

    /// Baut einen minimalen, serialisierbaren Viewport-Geometry-Snapshot.
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
}

impl Default for HostBridgeSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
