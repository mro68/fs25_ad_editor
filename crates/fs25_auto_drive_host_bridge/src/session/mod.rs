use anyhow::Result;
use fs25_auto_drive_engine::app::ui_contract::{HostUiSnapshot, ViewportOverlaySnapshot};
use fs25_auto_drive_engine::app::{AppController, AppState, EditorTool};
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use glam::Vec2;

use crate::dispatch::HostViewportInputState;
use crate::dto::{
    HostActiveTool, HostDialogRequest, HostDialogResult, HostSelectionSnapshot, HostSessionAction,
    HostSessionSnapshot, HostViewportGeometrySnapshot, HostViewportSnapshot,
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
    use fs25_auto_drive_engine::app::{AppIntent, MapNode, NodeFlag, RoadMap};
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
        assert!(host_ui.command_palette_state().is_some());
        assert!(host_ui.options_panel_state().is_some());

        let overlay = session.build_viewport_overlay_snapshot(None);
        assert!(overlay.route_tool_preview.is_none());
        assert!(overlay.group_boundaries.is_empty());
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
