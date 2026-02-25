//! Multiple Nodes Menu: Rechtsklick bei ≥2 selektierten Nodes.

use super::button_intent;
use crate::app::{
    state::DistanzenState, AppIntent, ConnectionDirection, ConnectionPriority, RoadMap,
};
use std::collections::HashSet;

pub fn render_multiple_nodes_menu(
    ui: &mut egui::Ui,
    selected_node_ids: &HashSet<u64>,
    road_map: &RoadMap,
    distanzen_state: &mut DistanzenState,
    events: &mut Vec<AppIntent>,
) {
    ui.label(format!("📍 {} Nodes selektiert", selected_node_ids.len()));

    let connection_count = road_map
        .connections_iter()
        .filter(|c| {
            selected_node_ids.contains(&c.start_id) && selected_node_ids.contains(&c.end_id)
        })
        .count();
    let can_connect_two = selected_node_ids.len() == 2 && connection_count == 0;

    if can_connect_two {
        ui.separator();
        button_intent(
            ui,
            "🔗 Nodes verbinden",
            AppIntent::ConnectSelectedNodesRequested,
            events,
        );
    }

    if connection_count > 0 {
        ui.separator();
        ui.label(format!("🔗 {} Verbindung(en)", connection_count));
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

        ui.separator();
        if distanzen_state.active {
            ui.label("Streckenteilung:");

            let prev_distance = distanzen_state.distance;
            ui.horizontal(|ui| {
                ui.label("Abstand:");
                ui.add(
                    egui::DragValue::new(&mut distanzen_state.distance)
                        .speed(0.5)
                        .range(1.0..=25.0)
                        .suffix(" m"),
                );
            });
            if (distanzen_state.distance - prev_distance).abs() > f32::EPSILON {
                distanzen_state.by_count = false;
                distanzen_state.sync_from_distance();
            }

            let prev_count = distanzen_state.count;
            ui.horizontal(|ui| {
                ui.label("Nodes:");
                ui.add(
                    egui::DragValue::new(&mut distanzen_state.count)
                        .speed(1.0)
                        .range(2..=10000),
                );
            });
            if distanzen_state.count != prev_count {
                distanzen_state.by_count = true;
                distanzen_state.sync_from_count();
                if distanzen_state.distance < 1.0 {
                    distanzen_state.distance = 1.0;
                    distanzen_state.sync_from_distance();
                }
            }

            ui.add_space(4.0);
            if ui.button("✓ Übernehmen").clicked() {
                events.push(AppIntent::ResamplePathRequested);
                distanzen_state.deactivate();
                ui.close();
            }
            if ui.button("✕ Verwerfen").clicked() {
                distanzen_state.deactivate();
                ui.close();
            }
        } else {
            button_intent(
                ui,
                "✂ Streckenteilung",
                AppIntent::StreckenteilungAktivieren,
                events,
            );
        }
    }

    ui.separator();
    ui.label("📐 Selektion");
    button_intent(
        ui,
        "🔄 Invertieren",
        AppIntent::InvertSelectionRequested,
        events,
    );
    button_intent(ui, "Alles auswählen", AppIntent::SelectAllRequested, events);
    button_intent(
        ui,
        "✕ Auswahl löschen",
        AppIntent::ClearSelectionRequested,
        events,
    );

    ui.separator();
    button_intent(ui, "✂ Löschen", AppIntent::DeleteSelectedRequested, events);
    button_intent(
        ui,
        "⧉ Duplizieren",
        AppIntent::DuplicateSelectedNodesRequested,
        events,
    );
}
