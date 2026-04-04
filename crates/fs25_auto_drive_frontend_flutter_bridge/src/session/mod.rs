use anyhow::Result;
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState, EditorTool};
use fs25_auto_drive_engine::app::ui_contract::{
    dialog_result_to_intent, DialogRequest, DialogRequestKind, DialogResult,
};
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};

use crate::dto::{
    EngineActiveTool, EngineDialogRequest, EngineDialogRequestKind, EngineDialogResult,
    EngineSelectionSnapshot, EngineSessionAction, EngineSessionSnapshot, EngineViewportSnapshot,
};

fn map_active_tool(tool: EditorTool) -> EngineActiveTool {
    match tool {
        EditorTool::Select => EngineActiveTool::Select,
        EditorTool::Connect => EngineActiveTool::Connect,
        EditorTool::AddNode => EngineActiveTool::AddNode,
        EditorTool::Route => EngineActiveTool::Route,
    }
}

fn map_editor_tool(tool: EngineActiveTool) -> EditorTool {
    match tool {
        EngineActiveTool::Select => EditorTool::Select,
        EngineActiveTool::Connect => EditorTool::Connect,
        EngineActiveTool::AddNode => EditorTool::AddNode,
        EngineActiveTool::Route => EditorTool::Route,
    }
}

fn map_dialog_request_kind(kind: DialogRequestKind) -> EngineDialogRequestKind {
    match kind {
        DialogRequestKind::OpenFile => EngineDialogRequestKind::OpenFile,
        DialogRequestKind::SaveFile => EngineDialogRequestKind::SaveFile,
        DialogRequestKind::Heightmap => EngineDialogRequestKind::Heightmap,
        DialogRequestKind::BackgroundMap => EngineDialogRequestKind::BackgroundMap,
        DialogRequestKind::OverviewZip => EngineDialogRequestKind::OverviewZip,
        DialogRequestKind::CurseplayImport => EngineDialogRequestKind::CurseplayImport,
        DialogRequestKind::CurseplayExport => EngineDialogRequestKind::CurseplayExport,
    }
}

fn map_engine_dialog_request_kind(kind: EngineDialogRequestKind) -> DialogRequestKind {
    match kind {
        EngineDialogRequestKind::OpenFile => DialogRequestKind::OpenFile,
        EngineDialogRequestKind::SaveFile => DialogRequestKind::SaveFile,
        EngineDialogRequestKind::Heightmap => DialogRequestKind::Heightmap,
        EngineDialogRequestKind::BackgroundMap => DialogRequestKind::BackgroundMap,
        EngineDialogRequestKind::OverviewZip => DialogRequestKind::OverviewZip,
        EngineDialogRequestKind::CurseplayImport => DialogRequestKind::CurseplayImport,
        EngineDialogRequestKind::CurseplayExport => DialogRequestKind::CurseplayExport,
    }
}

fn map_dialog_request(request: DialogRequest) -> EngineDialogRequest {
    EngineDialogRequest {
        kind: map_dialog_request_kind(request.kind()),
        suggested_file_name: request.suggested_file_name().map(str::to_owned),
    }
}

fn map_dialog_result(result: EngineDialogResult) -> DialogResult {
    match result {
        EngineDialogResult::Cancelled { kind } => DialogResult::Cancelled {
            kind: map_engine_dialog_request_kind(kind),
        },
        EngineDialogResult::PathSelected { kind, path } => DialogResult::PathSelected {
            kind: map_engine_dialog_request_kind(kind),
            path,
        },
    }
}

fn action_to_intent(action: EngineSessionAction) -> Option<AppIntent> {
    match action {
        EngineSessionAction::ToggleCommandPalette => Some(AppIntent::CommandPaletteToggled),
        EngineSessionAction::SetEditorTool { tool } => Some(AppIntent::SetEditorToolRequested {
            tool: map_editor_tool(tool),
        }),
        EngineSessionAction::OpenOptionsDialog => Some(AppIntent::OpenOptionsDialogRequested),
        EngineSessionAction::CloseOptionsDialog => Some(AppIntent::CloseOptionsDialogRequested),
        EngineSessionAction::Undo => Some(AppIntent::UndoRequested),
        EngineSessionAction::Redo => Some(AppIntent::RedoRequested),
        EngineSessionAction::SubmitDialogResult { result } => {
            dialog_result_to_intent(map_dialog_result(result))
        }
    }
}

fn build_snapshot(state: &AppState) -> EngineSessionSnapshot {
    EngineSessionSnapshot {
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
        selection: EngineSelectionSnapshot {
            selected_node_ids: state.selection.selected_node_ids.iter().copied().collect(),
        },
        viewport: EngineViewportSnapshot {
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
pub struct EngineRenderFrameSnapshot {
    /// Per-frame Render-Vertrag mit Kamera-, Sichtbarkeits- und Geometriedaten.
    pub scene: RenderScene,
    /// Langlebige Render-Assets inklusive Revisionen.
    pub assets: RenderAssetsSnapshot,
}

/// Kleine Session-Fassade ueber der host-neutralen Engine.
///
/// Die Bridge kapselt `AppController` und `AppState`, bleibt aber absichtlich rein
/// Rust-seitig. Ein spaeteres Flutter-Transport- oder FFI-Layer kann diese API
/// adaptieren, ohne die Engine selbst an ein bestimmtes SDK zu koppeln.
pub struct FlutterBridgeSession {
    controller: AppController,
    state: AppState,
    snapshot_cache: EngineSessionSnapshot,
    snapshot_dirty: bool,
}

impl FlutterBridgeSession {
    /// Erstellt eine neue Bridge-Session mit leerem Engine-State.
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
    pub fn apply_action(&mut self, action: EngineSessionAction) -> Result<()> {
        if let Some(intent) = action_to_intent(action) {
            self.apply_intent(intent)?;
        }
        Ok(())
    }

    /// Schaltet die Command-Palette um.
    pub fn toggle_command_palette(&mut self) -> Result<()> {
        self.apply_action(EngineSessionAction::ToggleCommandPalette)
    }

    /// Setzt das aktive Editor-Tool.
    pub fn set_editor_tool(&mut self, tool: EngineActiveTool) -> Result<()> {
        self.apply_action(EngineSessionAction::SetEditorTool { tool })
    }

    /// Oeffnet oder schliesst den Optionen-Dialog explizit.
    pub fn set_options_dialog_visible(&mut self, visible: bool) -> Result<()> {
        let action = if visible {
            EngineSessionAction::OpenOptionsDialog
        } else {
            EngineSessionAction::CloseOptionsDialog
        };
        self.apply_action(action)
    }

    /// Fuehrt einen Undo-Schritt aus.
    pub fn undo(&mut self) -> Result<()> {
        self.apply_action(EngineSessionAction::Undo)
    }

    /// Fuehrt einen Redo-Schritt aus.
    pub fn redo(&mut self) -> Result<()> {
        self.apply_action(EngineSessionAction::Redo)
    }

    /// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen als Bridge-DTOs.
    pub fn take_dialog_requests(&mut self) -> Vec<EngineDialogRequest> {
        let requests = self.state.ui.take_dialog_requests();
        if !requests.is_empty() {
            self.snapshot_dirty = true;
        }
        requests.into_iter().map(map_dialog_request).collect()
    }

    /// Reicht ein host-seitiges Dialog-Ergebnis an die Engine weiter.
    pub fn submit_dialog_result(&mut self, result: EngineDialogResult) -> Result<()> {
        self.apply_action(EngineSessionAction::SubmitDialogResult { result })
    }

    /// Liefert einen referenzierten Snapshot fuer Polling-Hosts.
    ///
    /// Der Snapshot wird nur nach einer erfolgreichen Session-Mutation neu aufgebaut,
    /// damit bei Polling ohne State-Aenderung keine neuen Allokationen entstehen.
    pub fn snapshot(&mut self) -> &EngineSessionSnapshot {
        self.rebuild_snapshot_if_dirty();
        &self.snapshot_cache
    }

    /// Liefert eine besitzende Snapshot-Kopie.
    ///
    /// Diese Methode ist fuer Call-Sites gedacht, die den Snapshot vom Session-
    /// Lebenszyklus entkoppeln muessen.
    pub fn snapshot_owned(&mut self) -> EngineSessionSnapshot {
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
    pub fn build_render_frame(&self, viewport_size: [f32; 2]) -> EngineRenderFrameSnapshot {
        EngineRenderFrameSnapshot {
            scene: self.build_render_scene(viewport_size),
            assets: self.build_render_assets(),
        }
    }

    fn rebuild_snapshot_if_dirty(&mut self) {
        if !self.snapshot_dirty {
            return;
        }

        self.snapshot_cache = build_snapshot(&self.state);
        self.snapshot_dirty = false;
    }

    fn apply_intent(&mut self, intent: AppIntent) -> Result<()> {
        self.controller.handle_intent(&mut self.state, intent)?;
        self.snapshot_dirty = true;
        Ok(())
    }
}

impl Default for FlutterBridgeSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::AppIntent;

    use crate::dto::{
        EngineActiveTool, EngineDialogRequestKind, EngineSessionAction,
    };

    use super::FlutterBridgeSession;

    #[test]
    fn new_session_exposes_empty_snapshot() {
        let mut session = FlutterBridgeSession::new();
        let snapshot = session.snapshot();

        assert!(!snapshot.has_map);
        assert_eq!(snapshot.node_count, 0);
        assert_eq!(snapshot.connection_count, 0);
        assert_eq!(snapshot.active_tool, EngineActiveTool::Select);
        assert!(!snapshot.can_undo);
        assert!(!snapshot.can_redo);
        assert_eq!(snapshot.pending_dialog_request_count, 0);
        assert!(snapshot.selection.selected_node_ids.is_empty());
    }

    #[test]
    fn dispatch_updates_snapshot_state() {
        let mut session = FlutterBridgeSession::new();

        session
            .apply_action(EngineSessionAction::ToggleCommandPalette)
            .expect("ToggleCommandPalette muss funktionieren");

        let snapshot = session.snapshot();
        assert!(snapshot.show_command_palette);
    }

    #[test]
    fn active_tool_uses_explicit_stable_snapshot_identifier() {
        let mut session = FlutterBridgeSession::new();

        session
            .set_editor_tool(EngineActiveTool::Route)
            .expect("SetEditorToolRequested muss funktionieren");

        let snapshot = session.snapshot();
        assert_eq!(snapshot.active_tool, EngineActiveTool::Route);
    }

    #[test]
    fn options_dialog_visibility_is_controlled_via_explicit_actions() {
        let mut session = FlutterBridgeSession::new();

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
        let mut session = FlutterBridgeSession::new();

        session.undo().expect("Undo muss verfuegbar sein");
        session.redo().expect("Redo muss verfuegbar sein");

        let snapshot = session.snapshot();
        assert!(!snapshot.can_undo);
        assert!(!snapshot.can_redo);
    }

    #[test]
    fn take_dialog_requests_drains_pending_queue_for_host_polling() {
        let mut session = FlutterBridgeSession::new();

        session
            .apply_intent(AppIntent::CurseplayImportRequested)
            .expect("CurseplayImportRequested muss Dialog anfordern");
        session
            .apply_intent(AppIntent::CurseplayExportRequested)
            .expect("CurseplayExportRequested muss Dialog anfordern");

        assert_eq!(session.snapshot().pending_dialog_request_count, 2);

        let requests = session.take_dialog_requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].kind, EngineDialogRequestKind::CurseplayImport);
        assert_eq!(requests[1].kind, EngineDialogRequestKind::CurseplayExport);
        assert_eq!(session.snapshot().pending_dialog_request_count, 0);
    }

    #[test]
    fn render_accessors_expose_scene_and_assets_without_state_leaks() {
        let session = FlutterBridgeSession::new();

        let scene = session.build_render_scene([800.0, 600.0]);
        let assets = session.build_render_assets();
        let frame = session.build_render_frame([320.0, 240.0]);

        assert!(!scene.has_map());
        assert_eq!(assets.background_asset_revision(), 0);
        assert!(assets.background().is_none());
        assert_eq!(frame.scene.viewport_size(), [320.0, 240.0]);
        assert_eq!(frame.assets.background_transform_revision(), 0);
    }
}
