//! Tangenten-Auswahl und Node-Info-Submenu für das Kontextmenü.

use crate::app::tool_contract::TangentSource;
use crate::app::AppIntent;
use fs25_auto_drive_host_bridge::{HostNodeDetails, HostTangentMenuSnapshot, HostTangentSource};

fn map_host_tangent_source_to_engine(source: HostTangentSource) -> TangentSource {
    match source {
        HostTangentSource::None => TangentSource::None,
        HostTangentSource::Connection { neighbor_id, angle } => {
            TangentSource::Connection { neighbor_id, angle }
        }
    }
}

/// Info-Submenu für einen Node (öffnet bei Hover, zeigt Details).
pub(super) fn render_node_info_submenu(
    ui: &mut egui::Ui,
    node_id: u64,
    node_details: Option<&HostNodeDetails>,
) {
    ui.menu_button("ℹ Info", |ui| {
        if let Some(details) = node_details {
            ui.label(format!("📍 Node {}", node_id));
            ui.label(format!(
                "Position: ({:.1}, {:.1})",
                details.position[0], details.position[1]
            ));
            ui.label(format!("Flag: {:?}", details.flag));
            ui.separator();
            let out_count = details
                .neighbors
                .iter()
                .filter(|neighbor| neighbor.is_outgoing)
                .count();
            let in_count = details
                .neighbors
                .iter()
                .filter(|neighbor| !neighbor.is_outgoing)
                .count();
            ui.label(format!("Ausgehend: {}", out_count));
            ui.label(format!("Eingehend: {}", in_count));
            if let Some(marker) = &details.marker {
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
    data: &HostTangentMenuSnapshot,
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
        for option in &data.start_options {
            let is_sel = option.source == data.current_start;
            if ui.selectable_label(is_sel, &option.label).clicked() {
                events.push(AppIntent::RouteToolTangentSelected {
                    start: map_host_tangent_source_to_engine(option.source),
                    end: map_host_tangent_source_to_engine(data.current_end),
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
        for option in &data.end_options {
            let is_sel = option.source == data.current_end;
            if ui.selectable_label(is_sel, &option.label).clicked() {
                events.push(AppIntent::RouteToolTangentSelected {
                    start: map_host_tangent_source_to_engine(data.current_start),
                    end: map_host_tangent_source_to_engine(option.source),
                });
                ui.close();
            }
        }
    }
}
