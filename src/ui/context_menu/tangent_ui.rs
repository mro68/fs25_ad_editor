//! Tangenten-Auswahl und Node-Info-Submenu für das Kontextmenü.

use crate::app::tools::common::TangentMenuData;
use crate::app::{AppIntent, RoadMap};

/// Info-Submenu für einen Node (öffnet bei Hover, zeigt Details).
pub(super) fn render_node_info_submenu(ui: &mut egui::Ui, node_id: u64, road_map: &RoadMap) {
    ui.menu_button("ℹ Info", |ui| {
        if let Some(node) = road_map.nodes.get(&node_id) {
            ui.label(format!("📍 Node {}", node_id));
            ui.label(format!(
                "Position: ({:.1}, {:.1})",
                node.position.x, node.position.y
            ));
            ui.label(format!("Flag: {:?}", node.flag));
            ui.separator();
            let out_count = road_map
                .connections_iter()
                .filter(|c| c.start_id == node_id)
                .count();
            let in_count = road_map
                .connections_iter()
                .filter(|c| c.end_id == node_id)
                .count();
            ui.label(format!("Ausgehend: {}", out_count));
            ui.label(format!("Eingehend: {}", in_count));
            if let Some(marker) = road_map.find_marker_by_node_id(node_id) {
                ui.separator();
                ui.label(format!("🗺 Marker: {}", marker.name));
                ui.label(format!("Gruppe: {}", marker.group));
            }
        } else {
            ui.label("Node nicht gefunden");
        }
    });
}

/// Tangenten-Auswahl für Route-Tool (ComboBox, nicht als Command).
pub(super) fn render_tangent_selection(
    ui: &mut egui::Ui,
    data: &TangentMenuData,
    events: &mut Vec<AppIntent>,
) {
    let has_start = !data.start_options.is_empty();
    let has_end = !data.end_options.is_empty();

    if !has_start && !has_end {
        return;
    }

    ui.separator();
    ui.label("🎯 Tangenten");

    if has_start {
        ui.label("Start:");
        for (source, label) in &data.start_options {
            let is_sel = *source == data.current_start;
            if ui.selectable_label(is_sel, label).clicked() {
                events.push(AppIntent::RouteToolTangentSelected {
                    start: *source,
                    end: data.current_end,
                });
                ui.close();
            }
        }
    }

    if has_start && has_end {
        ui.separator();
    }

    if has_end {
        ui.label("Ende:");
        for (source, label) in &data.end_options {
            let is_sel = *source == data.current_end;
            if ui.selectable_label(is_sel, label).clicked() {
                events.push(AppIntent::RouteToolTangentSelected {
                    start: data.current_start,
                    end: *source,
                });
                ui.close();
            }
        }
    }
}
