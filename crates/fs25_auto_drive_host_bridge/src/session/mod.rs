use anyhow::Result;
use fs25_auto_drive_engine::app::ui_contract::{
    DialogRequest, DialogRequestKind, HostUiSnapshot, ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::{AppController, AppState, EditorTool};
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use glam::Vec2;

use crate::dto::{
    HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult,
    HostSelectionSnapshot, HostSessionAction, HostSessionSnapshot, HostViewportSnapshot,
};

fn map_active_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
    }
}

fn map_dialog_request_kind(kind: DialogRequestKind) -> HostDialogRequestKind {
    match kind {
        DialogRequestKind::OpenFile => HostDialogRequestKind::OpenFile,
        DialogRequestKind::SaveFile => HostDialogRequestKind::SaveFile,
        DialogRequestKind::Heightmap => HostDialogRequestKind::Heightmap,
        DialogRequestKind::BackgroundMap => HostDialogRequestKind::BackgroundMap,
        DialogRequestKind::OverviewZip => HostDialogRequestKind::OverviewZip,
        DialogRequestKind::CurseplayImport => HostDialogRequestKind::CurseplayImport,
        DialogRequestKind::CurseplayExport => HostDialogRequestKind::CurseplayExport,
    }
}

fn map_dialog_request(request: DialogRequest) -> HostDialogRequest {
    HostDialogRequest {
        kind: map_dialog_request_kind(request.kind()),
        suggested_file_name: request.suggested_file_name().map(str::to_owned),
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

/// Kleine Session-Fassade ueber der host-neutralen Engine.
///
/// Die Bridge kapselt `AppController` und `AppState`, bleibt aber absichtlich
/// toolkit-frei. Host-spezifische Runtime-, Dialog- oder FFI-Details bleiben in
/// den jeweiligen Host-Adaptern.
pub struct HostBridgeSession {
    controller: AppController,
    state: AppState,
    snapshot_cache: HostSessionSnapshot,
    snapshot_dirty: bool,
}

impl HostBridgeSession {
    /// Erstellt eine neue Host-Bridge-Session mit leerem Engine-State.
    pub fn new() -> Self {
        let state = AppState::new();
        let snapshot_cache = build_snapshot(&state);

        Self {
            controller: AppController::new(),
            state,
            snapshot_cache,
            snapshot_dirty: false,
        }
    }

    /// Wendet eine explizite Host-Aktion auf die Session an.
    pub fn apply_action(&mut self, action: HostSessionAction) -> Result<()> {
        let handled =
            crate::dispatch::apply_host_action(&mut self.controller, &mut self.state, action)?;
        if handled {
            self.snapshot_dirty = true;
        }
        Ok(())
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
    pub fn take_dialog_requests(&mut self) -> Vec<HostDialogRequest> {
        let requests = self.controller.take_dialog_requests(&mut self.state);
        if !requests.is_empty() {
            self.snapshot_dirty = true;
        }
        requests.into_iter().map(map_dialog_request).collect()
    }

    /// Reicht ein host-seitiges Dialog-Ergebnis an die Engine weiter.
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

    /// Baut den host-neutralen Host-UI-Snapshot (Panels).
    pub fn build_host_ui_snapshot(&self) -> HostUiSnapshot {
        self.controller.build_host_ui_snapshot(&self.state)
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
}

impl Default for HostBridgeSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::AppIntent;

    use crate::dto::{HostActiveTool, HostDialogRequestKind, HostSessionAction};

    use super::HostBridgeSession;

    fn apply_test_intent(session: &mut HostBridgeSession, intent: AppIntent) {
        session
            .controller
            .handle_intent(&mut session.state, intent)
            .expect("Test-Intent muss verarbeitet werden");
        session.snapshot_dirty = true;
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
        assert!(host_ui.command_palette_state().is_some());
        assert!(host_ui.options_panel_state().is_some());

        let overlay = session.build_viewport_overlay_snapshot(None);
        assert!(overlay.route_tool_preview.is_none());
        assert!(overlay.group_boundaries.is_empty());
    }
}
