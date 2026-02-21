//! Properties-Panel (rechte Seitenleiste) für Node- und Connection-Eigenschaften.

use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority};

/// Rendert das Properties-Panel und gibt erzeugte Events zurück.
pub fn render_properties_panel(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    egui::SidePanel::right("properties_panel")
        .default_width(200.0)
        .min_width(160.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Eigenschaften");
            ui.separator();

            let selected: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();

            if selected.is_empty() {
                ui.label("Keine Selektion");
                ui.separator();
                render_default_direction_selector(ui, state, &mut events);
                return;
            }

            let road_map = match state.road_map.as_deref() {
                Some(rm) => rm,
                None => return,
            };

            // Node-Info
            match selected.len() {
                1 => {
                    let node_id = selected[0];
                    if let Some(node) = road_map.nodes.get(&node_id) {
                        ui.label(format!("Node ID: {}", node.id));
                        ui.label(format!(
                            "Position: ({:.1}, {:.1})",
                            node.position.x, node.position.y
                        ));
                        ui.label(format!("Flag: {:?}", node.flag));

                        // Map-Marker Info
                        ui.separator();
                        if let Some(marker) = road_map.find_marker_by_node_id(node_id) {
                            ui.label("🗺 Map-Marker");
                            ui.label(format!("Name: {}", marker.name));
                            ui.label(format!("Gruppe: {}", marker.group));

                            if ui.small_button("✏ Marker ändern").clicked() {
                                events.push(AppIntent::EditMarkerRequested { node_id });
                            }
                            if ui.small_button("✕ Marker löschen").clicked() {
                                events.push(AppIntent::RemoveMarkerRequested { node_id });
                            }
                        } else if ui.button("🗺 Marker erstellen").clicked() {
                            events.push(AppIntent::CreateMarkerRequested { node_id });
                        }
                    }
                }
                2 => {
                    let (a, b) = (selected[0], selected[1]);
                    ui.label(format!("Nodes: {}, {}", a, b));
                    ui.separator();

                    // Verbindungen zwischen diesen beiden Nodes
                    let conns = road_map.find_connections_between(a, b);

                    if conns.is_empty() {
                        ui.label("Keine Verbindung");
                        ui.separator();

                        // Verbinden-Buttons
                        let direction = state.editor.default_direction;
                        let priority = state.editor.default_priority;
                        let dir_label = direction_label(direction);
                        let prio_label = priority_label(priority);
                        if ui
                            .button(format!("Verbinden ({}, {})", dir_label, prio_label))
                            .clicked()
                        {
                            events.push(AppIntent::AddConnectionRequested {
                                from_id: a,
                                to_id: b,
                                direction,
                                priority,
                            });
                        }
                    } else {
                        for conn in &conns {
                            ui.group(|ui| {
                                ui.label(format!("{}→{}", conn.start_id, conn.end_id));

                                // Direction-Dropdown
                                let current_dir = conn.direction;
                                let start_id = conn.start_id;
                                let end_id = conn.end_id;

                                let mut selected_dir = current_dir;
                                egui::ComboBox::from_id_salt(format!(
                                    "dir_{}_{}",
                                    start_id, end_id
                                ))
                                .selected_text(direction_label(selected_dir))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut selected_dir,
                                        ConnectionDirection::Regular,
                                        "Regular (Einbahn)",
                                    );
                                    ui.selectable_value(
                                        &mut selected_dir,
                                        ConnectionDirection::Dual,
                                        "Dual (Bidirektional)",
                                    );
                                    ui.selectable_value(
                                        &mut selected_dir,
                                        ConnectionDirection::Reverse,
                                        "Reverse (Rückwärts)",
                                    );
                                });

                                if selected_dir != current_dir {
                                    events.push(AppIntent::SetConnectionDirectionRequested {
                                        start_id,
                                        end_id,
                                        direction: selected_dir,
                                    });
                                }

                                // Priority-Dropdown
                                let current_prio = conn.priority;
                                let mut selected_prio = current_prio;
                                egui::ComboBox::from_id_salt(format!(
                                    "prio_{}_{}",
                                    start_id, end_id
                                ))
                                .selected_text(priority_label(selected_prio))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut selected_prio,
                                        ConnectionPriority::Regular,
                                        "Hauptstraße",
                                    );
                                    ui.selectable_value(
                                        &mut selected_prio,
                                        ConnectionPriority::SubPriority,
                                        "Nebenstraße",
                                    );
                                });

                                if selected_prio != current_prio {
                                    events.push(AppIntent::SetConnectionPriorityRequested {
                                        start_id,
                                        end_id,
                                        priority: selected_prio,
                                    });
                                }

                                // Trennen-Button
                                if ui.small_button("✕ Trennen").clicked() {
                                    events.push(AppIntent::RemoveConnectionBetweenRequested {
                                        node_a: start_id,
                                        node_b: end_id,
                                    });
                                }
                            });
                        }
                    }
                }
                n => {
                    ui.label(format!("{} Nodes selektiert", n));
                }
            }

            ui.separator();
            render_default_direction_selector(ui, state, &mut events);
        });

    events
}

fn render_default_direction_selector(
    ui: &mut egui::Ui,
    state: &AppState,
    events: &mut Vec<AppIntent>,
) {
    ui.label("Standard-Richtung:");
    let current = state.editor.default_direction;
    let mut selected = current;

    egui::ComboBox::from_id_salt("default_direction")
        .selected_text(direction_label(selected))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut selected,
                ConnectionDirection::Regular,
                "Regular (Einbahn)",
            );
            ui.selectable_value(
                &mut selected,
                ConnectionDirection::Dual,
                "Dual (Bidirektional)",
            );
            ui.selectable_value(
                &mut selected,
                ConnectionDirection::Reverse,
                "Reverse (Rückwärts)",
            );
        });

    if selected != current {
        events.push(AppIntent::SetDefaultDirectionRequested {
            direction: selected,
        });
    }

    ui.add_space(4.0);
    ui.label("Standard-Straßenart:");
    let current_prio = state.editor.default_priority;
    let mut selected_prio = current_prio;

    egui::ComboBox::from_id_salt("default_priority")
        .selected_text(priority_label(selected_prio))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut selected_prio,
                ConnectionPriority::Regular,
                "Hauptstraße",
            );
            ui.selectable_value(
                &mut selected_prio,
                ConnectionPriority::SubPriority,
                "Nebenstraße",
            );
        });

    if selected_prio != current_prio {
        events.push(AppIntent::SetDefaultPriorityRequested {
            priority: selected_prio,
        });
    }
}

fn direction_label(dir: ConnectionDirection) -> &'static str {
    match dir {
        ConnectionDirection::Regular => "Regular",
        ConnectionDirection::Dual => "Dual",
        ConnectionDirection::Reverse => "Reverse",
    }
}

fn priority_label(prio: ConnectionPriority) -> &'static str {
    match prio {
        ConnectionPriority::Regular => "Hauptstraße",
        ConnectionPriority::SubPriority => "Nebenstraße",
    }
}
