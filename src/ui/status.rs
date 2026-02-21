//! Status-Bar am unteren Bildschirmrand.

use crate::app::{AppState, EditorTool};

/// Rendert die Status-Bar
pub fn render_status_bar(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Some(road_map) = &state.road_map {
                ui.label(format!(
                    "Nodes: {} | Connections: {} | Markers: {}",
                    road_map.node_count(),
                    road_map.connection_count(),
                    road_map.marker_count()
                ));

                ui.separator();

                if let Some(map_name) = &road_map.map_name {
                    ui.label(format!("Map: {}", map_name));
                    ui.separator();
                }
            } else {
                ui.label("No file loaded");
            }

            ui.separator();

            ui.label(format!(
                "Zoom: {:.2}x | Position: ({:.1}, {:.1})",
                state.view.camera.zoom, state.view.camera.position.x, state.view.camera.position.y
            ));

            ui.separator();

            // Heightmap-Status
            if let Some(ref hm_path) = state.ui.heightmap_path {
                let filename = std::path::Path::new(hm_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                ui.label(format!("Heightmap: {}", filename));
            } else {
                ui.label("Heightmap: None");
            }

            ui.separator();

            let selected_count = state.selection.selected_node_ids.len();
            if selected_count > 0 {
                let example_id = state
                    .selection
                    .selected_node_ids
                    .iter()
                    .next()
                    .copied()
                    .unwrap_or_default();
                ui.label(format!(
                    "Selected Nodes: {} (z.B. {})",
                    selected_count, example_id
                ));
            } else {
                ui.label("Selected Nodes: 0");
            }

            ui.separator();

            // Aktives Werkzeug
            let tool_name = match state.editor.active_tool {
                EditorTool::Select => "Select",
                EditorTool::Connect => "Connect",
                EditorTool::AddNode => "Add Node",
            };
            ui.label(format!("Tool: {}", tool_name));

            // Statusnachricht (z.B. Duplikat-Bereinigung)
            if let Some(ref msg) = state.ui.status_message {
                ui.separator();
                ui.label(egui::RichText::new(format!("⚠ {}", msg)).color(egui::Color32::YELLOW));
            }

            // FPS-Anzeige (rechts)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("FPS: {:.0}", ctx.input(|i| 1.0 / i.stable_dt)));
            });
        });
    });
}
