use anyhow::Result;
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};

use crate::dto::{
    EngineSelectionSnapshot, EngineSessionSnapshot, EngineViewportSnapshot,
};

/// Kleine Session-Fassade ueber der host-neutralen Engine.
///
/// Die Bridge kapselt `AppController` und `AppState`, bleibt aber absichtlich rein
/// Rust-seitig. Ein spaeteres Flutter-Transport- oder FFI-Layer kann diese API
/// adaptieren, ohne die Engine selbst an ein bestimmtes SDK zu koppeln.
pub struct FlutterBridgeSession {
    controller: AppController,
    state: AppState,
}

impl FlutterBridgeSession {
    /// Erstellt eine neue Bridge-Session mit leerem Engine-State.
    pub fn new() -> Self {
        Self {
            controller: AppController::new(),
            state: AppState::new(),
        }
    }

    /// Wendet einen bestehenden Engine-Intent auf die Session an.
    pub fn dispatch(&mut self, intent: AppIntent) -> Result<()> {
        self.controller.handle_intent(&mut self.state, intent)
    }

    /// Liefert einen serialisierbaren Snapshot fuer Host-Frontends.
    pub fn snapshot(&self) -> EngineSessionSnapshot {
        EngineSessionSnapshot {
            has_map: self.state.road_map.is_some(),
            node_count: self.state.node_count(),
            connection_count: self.state.connection_count(),
            active_tool: format!("{:?}", self.state.editor.active_tool),
            status_message: self.state.ui.status_message.clone(),
            show_command_palette: self.state.ui.show_command_palette,
            show_options_dialog: self.state.show_options_dialog,
            selection: EngineSelectionSnapshot {
                selected_node_ids: self.state.selection.selected_node_ids.iter().copied().collect(),
            },
            viewport: EngineViewportSnapshot {
                camera_position: [
                    self.state.view.camera.position.x,
                    self.state.view.camera.position.y,
                ],
                zoom: self.state.view.camera.zoom,
            },
        }
    }

    /// Gibt read-only Zugriff auf den zugrundeliegenden App-State.
    pub fn state(&self) -> &AppState {
        &self.state
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

    use super::FlutterBridgeSession;

    #[test]
    fn new_session_exposes_empty_snapshot() {
        let session = FlutterBridgeSession::new();
        let snapshot = session.snapshot();

        assert!(!snapshot.has_map);
        assert_eq!(snapshot.node_count, 0);
        assert_eq!(snapshot.connection_count, 0);
        assert_eq!(snapshot.active_tool, format!("{:?}", EditorTool::Select));
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
}
