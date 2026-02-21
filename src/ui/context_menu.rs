//! Kontextmenü für Bulk-Verbindungsoperationen im Viewport.

use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, RoadMap};
use std::collections::HashSet;

/// Helper-Funktion: Erstellt einen Button, der bei Klick einen Intent emittiert und das Menü schließt.
fn button_intent(ui: &mut egui::Ui, label: &str, intent: AppIntent, events: &mut Vec<AppIntent>) {
    if ui.button(label).clicked() {
        events.push(intent);
        ui.close();
    }
}

/// Zeigt das Kontextmenü für Bulk-Verbindungsänderungen bei Rechtsklick.
pub(super) fn show_connection_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    events: &mut Vec<AppIntent>,
) {
    if selected_node_ids.len() < 2 {
        return;
    }

    let Some(rm) = road_map else {
        return;
    };

    let connection_count = rm
        .connections_iter()
        .filter(|c| {
            selected_node_ids.contains(&c.start_id) && selected_node_ids.contains(&c.end_id)
        })
        .count();

    let can_connect_two = selected_node_ids.len() == 2 && connection_count == 0;

    if connection_count == 0 && !can_connect_two {
        return;
    }

    response.context_menu(|ui| {
        // Verbinden-Option wenn genau 2 Nodes ohne Verbindung
        if can_connect_two {
            button_intent(
                ui,
                "🔗 Nodes verbinden",
                AppIntent::ConnectSelectedNodesRequested,
                events,
            );
            if connection_count > 0 {
                ui.separator();
            }
        }

        if connection_count > 0 {
            ui.label(format!("{} Verbindung(en)", connection_count));
            ui.separator();

            ui.label("Richtung:");
            button_intent(
                ui,
                "↦ Regular (Einbahn)",
                AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                    direction: ConnectionDirection::Regular,
                },
                events,
            );
            button_intent(
                ui,
                "⇆ Dual (beidseitig)",
                AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                    direction: ConnectionDirection::Dual,
                },
                events,
            );
            button_intent(
                ui,
                "↤ Reverse (rückwärts)",
                AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                    direction: ConnectionDirection::Reverse,
                },
                events,
            );
            button_intent(
                ui,
                "⇄ Invertieren",
                AppIntent::InvertAllConnectionsBetweenSelectedRequested,
                events,
            );

            ui.separator();
            ui.label("Straßenart:");
            button_intent(
                ui,
                "🛣 Hauptstraße",
                AppIntent::SetAllConnectionsPriorityBetweenSelectedRequested {
                    priority: ConnectionPriority::Regular,
                },
                events,
            );
            button_intent(
                ui,
                "🛤 Nebenstraße",
                AppIntent::SetAllConnectionsPriorityBetweenSelectedRequested {
                    priority: ConnectionPriority::SubPriority,
                },
                events,
            );

            ui.separator();
            button_intent(
                ui,
                "✕ Alle trennen",
                AppIntent::RemoveAllConnectionsBetweenSelectedRequested,
                events,
            );
        }
    });
}

/// Zeigt das Kontextmenü für Map-Marker bei Rechtsklick auf einzelnen Node.
pub(super) fn show_node_marker_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    node_id: u64,
    events: &mut Vec<AppIntent>,
) {
    let Some(rm) = road_map else {
        return;
    };

    // Prüfen ob Node existiert
    if !rm.nodes.contains_key(&node_id) {
        return;
    }

    let has_marker = rm.has_marker(node_id);

    response.context_menu(|ui| {
        ui.label(format!("Node {}", node_id));
        ui.separator();

        if has_marker {
            button_intent(
                ui,
                "✏ Marker ändern",
                AppIntent::EditMarkerRequested { node_id },
                events,
            );
            button_intent(
                ui,
                "✕ Marker löschen",
                AppIntent::RemoveMarkerRequested { node_id },
                events,
            );
        } else {
            button_intent(
                ui,
                "🗺 Marker erstellen",
                AppIntent::CreateMarkerRequested { node_id },
                events,
            );
        }
    });
}
