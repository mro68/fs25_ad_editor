//! Rechte Sidebar: Alle Map-Marker gruppiert anzeigen.
//!
//! Bei Klick auf einen Marker wird `CenterOnNodeRequested` emittiert,
//! damit die Kamera den zugehoerigen Node zentriert.

use std::collections::BTreeMap;

use eframe::egui;
use fs25_auto_drive_host_bridge::{HostMarkerInfo, HostMarkerListSnapshot};

use crate::app::AppIntent;

/// Rendert den Marker-Inhalt in den übergebenen UI-Bereich.
///
/// Gibt eine Liste von `AppIntent`-Events zurück, die bei Klick auf
/// einen Marker ausgelöst werden.
pub fn render_marker_content(
    ui: &mut egui::Ui,
    marker_list: &HostMarkerListSnapshot,
    has_map: bool,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !has_map {
        ui.label("Keine Datei geladen");
        return events;
    }

    // Debug-Marker ausblenden; Marker nach Gruppe gruppieren
    let mut groups: BTreeMap<&str, Vec<&HostMarkerInfo>> = BTreeMap::new();
    for marker in &marker_list.markers {
        if marker.is_debug {
            continue;
        }
        groups
            .entry(marker.group.as_str())
            .or_default()
            .push(marker);
    }

    if groups.is_empty() {
        ui.label("Keine Marker vorhanden");
        return events;
    }

    for (group_name, markers) in &groups {
        egui::CollapsingHeader::new(*group_name)
            .default_open(true)
            .show(ui, |ui| {
                for marker in markers {
                    if ui.selectable_label(false, &marker.name).clicked() {
                        events.push(AppIntent::CenterOnNodeRequested {
                            node_id: marker.node_id,
                        });
                    }
                }
            });
    }

    events
}
