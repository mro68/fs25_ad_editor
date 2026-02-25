//! Single Node Menu: Rechtsklick auf einzelnen Node (selektiert/nicht).

use super::button_intent;
use crate::app::{AppIntent, RoadMap};

pub fn render_single_node_unselected_menu(
    ui: &mut egui::Ui,
    node_id: u64,
    road_map: &RoadMap,
    events: &mut Vec<AppIntent>,
) {
    if let Some(node) = road_map.nodes.get(&node_id) {
        ui.label(format!("📍 Node {}", node_id));
        ui.label(format!("Pos: ({:.1}, {:.1})", node.position.x, node.position.y));
        let in_count = road_map
            .connections_iter()
            .filter(|c| c.end_id == node_id)
            .count();
        let out_count = road_map
            .connections_iter()
            .filter(|c| c.start_id == node_id)
            .count();
        ui.label(format!("Verb.: {} ↦ {} ↤", out_count, in_count));
        ui.separator();

        button_intent(
            ui,
            "✓ Selektieren",
            AppIntent::NodePickRequested {
                world_pos: node.position,
                additive: false,
                extend_path: false,
            },
            events,
        );
        button_intent(
            ui,
            "⬚ Hinzufügen",
            AppIntent::NodePickRequested {
                world_pos: node.position,
                additive: true,
                extend_path: false,
            },
            events,
        );

        ui.separator();
        ui.label("🗺 Marker");
        button_intent(
            ui,
            "🗺 Erstellen...",
            AppIntent::CreateMarkerRequested { node_id },
            events,
        );

        ui.separator();
        button_intent(ui, "✂ Löschen", AppIntent::DeleteSelectedRequested, events);
    }
}

pub fn render_single_node_selected_menu(
    ui: &mut egui::Ui,
    node_id: u64,
    road_map: &RoadMap,
    events: &mut Vec<AppIntent>,
) {
    if let Some(node) = road_map.nodes.get(&node_id) {
        ui.label(format!("📍 Node {} ✓", node_id));
        ui.label(format!("Pos: ({:.1}, {:.1})", node.position.x, node.position.y));
        ui.separator();

        button_intent(
            ui,
            "⬚ Abwählen",
            AppIntent::NodePickRequested {
                world_pos: node.position,
                additive: true,
                extend_path: false,
            },
            events,
        );

        ui.separator();
        ui.label("🗺 Marker");
        let has_marker = road_map.has_marker(node_id);
        if has_marker {
            button_intent(
                ui,
                "✏ Bearbeiten...",
                AppIntent::EditMarkerRequested { node_id },
                events,
            );
            button_intent(
                ui,
                "✕ Löschen",
                AppIntent::RemoveMarkerRequested { node_id },
                events,
            );
        } else {
            button_intent(
                ui,
                "🗺 Erstellen...",
                AppIntent::CreateMarkerRequested { node_id },
                events,
            );
        }

        ui.separator();
        button_intent(ui, "✂ Löschen", AppIntent::DeleteSelectedRequested, events);
        button_intent(
            ui,
            "⧉ Duplizieren",
            AppIntent::DuplicateSelectedNodesRequested,
            events,
        );
    }
}
