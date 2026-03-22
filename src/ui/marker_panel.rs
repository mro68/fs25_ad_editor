//! Rechte Sidebar: Alle Map-Marker gruppiert anzeigen.
//!
//! Bei Klick auf einen Marker wird `CenterOnNodeRequested` emittiert,
//! damit die Kamera den zugehoerigen Node zentriert.

use std::collections::BTreeMap;

use eframe::egui;

use crate::app::{AppIntent, MapMarker, RoadMap};

/// Rendert den Marker-Inhalt in den übergebenen UI-Bereich.
///
/// Gibt eine Liste von `AppIntent`-Events zurück, die bei Klick auf
/// einen Marker ausgelöst werden.
pub fn render_marker_content(ui: &mut egui::Ui, road_map: Option<&RoadMap>) -> Vec<AppIntent> {
    let mut events = Vec::new();

    let Some(rm) = road_map else {
        ui.label("Keine Datei geladen");
        return events;
    };

    if rm.map_markers.is_empty() {
        ui.label("Keine Marker vorhanden");
        return events;
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

    for (group_name, markers) in &groups {
        egui::CollapsingHeader::new(*group_name)
            .default_open(true)
            .show(ui, |ui| {
                for marker in markers {
                    if ui.selectable_label(false, &marker.name).clicked() {
                        events.push(AppIntent::CenterOnNodeRequested { node_id: marker.id });
                    }
                }
            });
    }

    events
}
