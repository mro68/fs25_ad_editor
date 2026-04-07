use anyhow::Result;
use fs25_auto_drive_engine::app::state::DistanzenState;
use fs25_auto_drive_engine::app::ui_contract::{HostUiSnapshot, ViewportOverlaySnapshot};
use fs25_auto_drive_engine::app::{
    AppController, AppIntent, AppState, Camera2D, ConnectionDirection, ConnectionPriority,
    EditorTool, FloatingMenuKind, FloatingMenuState, GroupEditState, GroupRegistry, RoadMap,
    ToolEditStore, UiState,
};
use fs25_auto_drive_engine::shared::{EditorOptions, RenderAssetsSnapshot, RenderScene};
use glam::Vec2;
use indexmap::IndexSet;

mod chrome_state;
pub use chrome_state::HostLocalDialogState;

use crate::dispatch::HostViewportInputState;
use crate::dto::{
    HostActiveTool, HostChromeSnapshot, HostDialogRequest, HostDialogResult,
    HostRouteToolViewportSnapshot, HostSelectionSnapshot, HostSessionAction, HostSessionSnapshot,
    HostViewportGeometrySnapshot, HostViewportSnapshot,
};

fn map_active_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
    }
}

fn build_snapshot(state: &AppState) -> HostSessionSnapshot {
    HostSessionSnapshot {
        has_map: state.road_map.is_some(),
        node_count: state.node_count(),
        connection_count: state.connection_count(),
        active_tool: map_active_tool(state.editor.active_tool),
        status_message: state.ui.status_message.clone(),
        show_command_palette: state.ui.show_command_palette,
        show_options_dialog: state.ui.show_options_dialog,
        can_undo: state.can_undo(),
        can_redo: state.can_redo(),
        pending_dialog_request_count: state.ui.dialog_requests.len(),
        selection: HostSelectionSnapshot {
            selected_node_ids: state.selection.selected_node_ids.iter().copied().collect(),
        },
        viewport: HostViewportSnapshot {
            camera_position: [state.view.camera.position.x, state.view.camera.position.y],
            zoom: state.view.camera.zoom,
        },
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
/// gesamten `AppState` zu erhalten.
pub struct HostDialogUiState<'a> {
    /// Aktuelle Karte (falls geladen).
    pub road_map: Option<&'a RoadMap>,
    /// Lokaler UI-Dialogzustand.
    pub ui: &'a mut UiState,
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
    /// Host-lokaler Chrome- und Dialog-Sichtbarkeitszustand.
    chrome_state: HostLocalDialogState,
}

impl HostBridgeSession {
    /// Erstellt eine neue Host-Bridge-Session mit leerem Engine-State.
    pub fn new() -> Self {
        let state = AppState::new();
        let snapshot_cache = build_snapshot(&state);

        Self {
            controller: AppController::new(),
            state,
            viewport_input_state: HostViewportInputState::default(),
            snapshot_cache,
            snapshot_dirty: false,
            chrome_state: HostLocalDialogState::new(),
        }
    }

    /// Wendet eine explizite Host-Aktion auf die Session an.
    ///
    /// Die Methode delegiert auf die gemeinsame Rust-Host-Dispatch-Seam in
    /// `crate::dispatch::apply_host_action(...)` und markiert den Snapshot-
    /// Cache nur nach erfolgreich verarbeiteten Aktionen als dirty.
    pub fn apply_action(&mut self, action: HostSessionAction) -> Result<()> {
        let handled = crate::dispatch::apply_host_action_with_viewport_input_state(
            &mut self.controller,
            &mut self.state,
            &mut self.viewport_input_state,
            action,
        )?;
        if handled {
            self.snapshot_dirty = true;
            self.sync_chrome_from_engine();
        }
        Ok(())
    }

    /// Verarbeitet einen `AppIntent` direkt ueber den App-Controller.
    ///
    /// Diese Methode bleibt als Uebergangs-Seam fuer Hosts, die bereits auf
    /// Session-Ownership umgestellt sind, aber noch nicht alle Schreibpfade auf
    /// `HostSessionAction` umgehangen haben.
    pub fn apply_intent(&mut self, intent: AppIntent) -> Result<()> {
        self.controller.handle_intent(&mut self.state, intent)?;
        self.snapshot_dirty = true;
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

    /// Liefert eine mutable Referenz auf den aktuellen `AppState`.
    ///
    /// Mutationen ueber diese Uebergangs-Seam markieren den Session-Snapshot als
    /// dirty. Fachliche Mutationen sollen langfristig ueber `apply_action(...)`
    /// laufen.
    pub fn app_state_mut(&mut self) -> &mut AppState {
        self.snapshot_dirty = true;
        &mut self.state
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
    /// Der Zugriff invalidiert den Session-Snapshot nicht automatisch. Falls
    /// ein Rust-Host ueber diesen Escape-Hatch Felder aendert, die in
    /// `HostSessionSnapshot` sichtbar sind, muss danach explizit
    /// `mark_snapshot_dirty()` aufgerufen werden.
    pub fn dialog_ui_state_mut(&mut self) -> HostDialogUiState<'_> {
        let state = &mut self.state;

        HostDialogUiState {
            road_map: state.road_map.as_deref(),
            ui: &mut state.ui,
            options: &mut state.options,
        }
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
    pub fn take_dialog_requests(&mut self) -> Vec<HostDialogRequest> {
        let requests =
            crate::dispatch::take_host_dialog_requests(&self.controller, &mut self.state);
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

    /// Baut den aktuellen per-frame Render-Vertrag fuer den angegebenen Viewport.
    pub fn build_render_scene(&self, viewport_size: [f32; 2]) -> RenderScene {
        self.controller
            .build_render_scene(&self.state, viewport_size)
    }

    /// Baut den aktuellen Render-Asset-Snapshot.
    pub fn build_render_assets(&self) -> RenderAssetsSnapshot {
        self.controller.build_render_assets(&self.state)
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
        crate::dispatch::build_viewport_geometry_snapshot(
            &self.controller,
            &self.state,
            viewport_size,
        )
    }

    /// Baut den host-neutralen Host-UI-Snapshot fuer sichtbare Panels.
    ///
    /// Host-native Datei- und Pfaddialoge laufen bewusst nicht ueber diesen
    /// Snapshot, sondern separat ueber `take_dialog_requests()`.
    pub fn build_host_ui_snapshot(&self) -> HostUiSnapshot {
        self.controller.build_host_ui_snapshot(&self.state)
    }

    /// Baut den host-neutralen Chrome-Snapshot fuer Menues, Defaults und Status.
    ///
    /// Die Felder `show_command_palette` und `show_options_dialog` stammen aus
    /// `chrome_state`, das per Drain nach jedem Engine-Intent synchronisiert wird.
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
        self.controller
            .build_viewport_overlay_snapshot(&mut self.state, cursor_world)
    }

    fn rebuild_snapshot_if_dirty(&mut self) {
        if !self.snapshot_dirty {
            return;
        }

        self.snapshot_cache = build_snapshot(&self.state);
        self.snapshot_dirty = false;
    }

    /// Spiegelt Engine-UI-Request-Flags in den host-lokalen Chrome-State.
    ///
    /// Wird nach jedem `apply_action()`/`apply_intent()` aufgerufen, damit
    /// `chrome_state` immer die aktuellen Engine-Werte fuer `show_command_palette`
    /// und `show_options_dialog` enthaelt.
    fn sync_chrome_from_engine(&mut self) {
        let show_cmd = self.state.ui.show_command_palette;
        let show_opts = self.state.ui.show_options_dialog;
        if self.chrome_state.show_command_palette != show_cmd
            || self.chrome_state.show_options_dialog != show_opts
        {
            self.chrome_state.show_command_palette = show_cmd;
            self.chrome_state.show_options_dialog = show_opts;
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
    use std::time::Instant;

    use fs25_auto_drive_engine::app::{
        AppIntent, Connection, ConnectionDirection, ConnectionPriority, FloatingMenuKind, MapNode,
        NodeFlag, RoadMap,
    };
    use glam::Vec2;
    use std::sync::Arc;

    use crate::dto::{
        EngineSessionAction, HostActiveTool, HostDialogRequestKind, HostDialogResult,
        HostInputModifiers, HostPointerButton, HostSessionAction, HostTapKind,
        HostViewportInputBatch, HostViewportInputEvent,
    };

    use super::{EngineRenderFrameSnapshot, FlutterBridgeSession, HostBridgeSession};

    fn apply_test_intent(session: &mut HostBridgeSession, intent: AppIntent) {
        session
            .controller
            .handle_intent(&mut session.state, intent)
            .expect("Test-Intent muss verarbeitet werden");
        session.snapshot_dirty = true;
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
    fn read_only_host_snapshots_do_not_mark_session_snapshot_dirty() {
        let session = snapshot_measurement_session(32);

        let _ = session.build_host_ui_snapshot();
        let _ = session.build_host_chrome_snapshot();
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
    fn explicit_snapshot_invalidation_keeps_local_ui_mutation_visible() {
        let mut session = snapshot_measurement_session(32);

        {
            let dialog_state = session.dialog_ui_state_mut();
            dialog_state.ui.status_message = Some("Lokale Mutation".to_string());
        }
        assert!(
            !session.snapshot_dirty,
            "Lokale UI-Seams invalidieren den Snapshot nicht implizit"
        );

        session.mark_snapshot_dirty();
        assert!(session.snapshot_dirty);

        let snapshot = session.snapshot_owned();
        assert_eq!(snapshot.status_message.as_deref(), Some("Lokale Mutation"));
        assert!(!session.snapshot_dirty);
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
            .app_state()
            .ui
            .floating_menu
            .expect("Tools-Menue muss geoeffnet sein");
        assert_eq!(tools_menu.kind, FloatingMenuKind::Tools);
        assert_eq!(tools_menu.pos, Vec2::new(10.0, 20.0));

        session.toggle_floating_menu(FloatingMenuKind::Tools, Some(Vec2::new(30.0, 40.0)));
        assert!(session.app_state().ui.floating_menu.is_none());

        session.toggle_floating_menu(FloatingMenuKind::Zoom, None);
        assert!(session.app_state().ui.floating_menu.is_none());

        session.toggle_floating_menu(FloatingMenuKind::Zoom, Some(Vec2::new(5.0, 6.0)));
        assert!(session.app_state().ui.floating_menu.is_some());

        session.clear_floating_menu();
        assert!(session.app_state().ui.floating_menu.is_none());
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
