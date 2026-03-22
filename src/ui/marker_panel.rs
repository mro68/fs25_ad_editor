//! Rechte Sidebar: Alle Map-Marker gruppiert anzeigen.
//!
//! Bei Klick auf einen Marker wird `CenterOnNodeRequested` emittiert,
//! damit die Kamera den zugehoerigen Node zentriert.

use std::collections::BTreeMap;

use eframe::egui;

use crate::app::{AppIntent, MapMarker, RoadMap};

/// Rechte Sidebar: Zeigt alle Marker gruppiert nach `MapMarker.group` an.
///
/// Gibt eine Liste von `AppIntent`-Events zurueck, die bei Klick auf
/// einen Marker ausgeloest werden.
pub fn render_marker_panel(ctx: &egui::Context, road_map: Option<&RoadMap>) -> Vec<AppIntent> {
    let mut events = Vec::new();

    egui::SidePanel::right("marker_panel")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Marker");
            ui.separator();

            let Some(rm) = road_map else {
                ui.label("Keine Datei geladen");
                return;
            };

            if rm.map_markers.is_empty() {
                ui.label("Keine Marker vorhanden");
                return;
            }

            // Debug-Marker ausblenden; Marker nach Gruppe gruppieren
            let mut groups: BTreeMap<&str, Vec<&MapMarker>> = BTreeMap::new();
            for marker in &rm.map_markers {
                if marker.is_debug {
                    continue;
                }
                groups
                    .entry(marker.group.as_str())
                    .or_default()
                    .push(marker);
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (group_name, markers) in &groups {
                    egui::CollapsingHeader::new(*group_name)
                        .default_open(true)
                        .show(ui, |ui| {
                            for marker in markers {
                                if ui.selectable_label(false, &marker.name).clicked() {
                                    events.push(AppIntent::CenterOnNodeRequested {
                                        node_id: marker.id,
                                    });
                                }
                            }
                        });
                }
            });
        });

    events
}
