use anyhow::Result;
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState, EditorTool};

use crate::dto::{
    EngineActiveTool, EngineSelectionSnapshot, EngineSessionSnapshot, EngineViewportSnapshot,
};

fn map_active_tool(tool: EditorTool) -> EngineActiveTool {
    match tool {
        EditorTool::Select => EngineActiveTool::Select,
        EditorTool::Connect => EngineActiveTool::Connect,
        EditorTool::AddNode => EngineActiveTool::AddNode,
        EditorTool::Route => EngineActiveTool::Route,
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
        show_options_dialog: state.show_options_dialog,
        selection: EngineSelectionSnapshot {
            selected_node_ids: state.selection.selected_node_ids.iter().copied().collect(),
        },
        viewport: EngineViewportSnapshot {
            camera_position: [state.view.camera.position.x, state.view.camera.position.y],
            zoom: state.view.camera.zoom,
        },
    }
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

    /// Wendet einen bestehenden Engine-Intent auf die Session an.
    pub fn dispatch(&mut self, intent: AppIntent) -> Result<()> {
        self.controller.handle_intent(&mut self.state, intent)?;
        self.snapshot_dirty = true;
        Ok(())
    }

    /// Liefert einen referenzierten Snapshot fuer Polling-Hosts.
    ///
    /// Der Snapshot wird nur nach einem erfolgreichen `dispatch()` neu aufgebaut,
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

    /// Gibt read-only Zugriff auf den zugrundeliegenden App-State.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    fn rebuild_snapshot_if_dirty(&mut self) {
        if !self.snapshot_dirty {
            return;
        }

        self.snapshot_cache = build_snapshot(&self.state);
        self.snapshot_dirty = false;
    }
}

impl Default for FlutterBridgeSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::{AppIntent, EditorTool};

    use crate::dto::EngineActiveTool;

    use super::FlutterBridgeSession;

    #[test]
    fn new_session_exposes_empty_snapshot() {
        let mut session = FlutterBridgeSession::new();
        let snapshot = session.snapshot();

        assert!(!snapshot.has_map);
        assert_eq!(snapshot.node_count, 0);
        assert_eq!(snapshot.connection_count, 0);
        assert_eq!(snapshot.active_tool, EngineActiveTool::Select);
        assert!(snapshot.selection.selected_node_ids.is_empty());
    }

    #[test]
    fn dispatch_updates_snapshot_state() {
        let mut session = FlutterBridgeSession::new();

        session
            .dispatch(AppIntent::CommandPaletteToggled)
            .expect("CommandPaletteToggled muss funktionieren");

        let snapshot = session.snapshot();
        assert!(snapshot.show_command_palette);
    }

    #[test]
    fn active_tool_uses_explicit_stable_snapshot_identifier() {
        let mut session = FlutterBridgeSession::new();

        session
            .dispatch(AppIntent::SetEditorToolRequested {
                tool: EditorTool::Route,
            })
            .expect("SetEditorToolRequested muss funktionieren");

        let snapshot = session.snapshot();
        assert_eq!(snapshot.active_tool, EngineActiveTool::Route);
    }
}
