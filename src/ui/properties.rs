//! Properties-Panel (rechte Seitenleiste) für Node- und Connection-Eigenschaften.

use std::collections::HashSet;

use crate::app::{
    segment_registry::SegmentRegistry, tools::ToolManager, AppIntent, ConnectionDirection,
    ConnectionPriority, EditorTool, RoadMap,
};
use crate::shared::EditorOptions;
use glam;

/// Rendert das Properties-Panel und gibt erzeugte Events zurück.
#[allow(clippy::too_many_arguments)]
pub fn render_properties_panel(
    ctx: &egui::Context,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    active_tool: EditorTool,
    tool_manager: Option<&mut ToolManager>,
    segment_registry: Option<&SegmentRegistry>,
    options: &EditorOptions,
    distanzen: &mut crate::app::state::DistanzenState,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    egui::SidePanel::right("properties_panel")
        .default_width(200.0)
        .min_width(160.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Eigenschaften");
            ui.separator();

            if selected_node_ids.is_empty() {
                ui.label("Keine Selektion");
            } else if let Some(road_map) = road_map {
                render_selection_info(
                    ui,
                    road_map,
                    selected_node_ids,
                    default_direction,
                    default_priority,
                    segment_registry,
                    &mut events,
                );
            }

            // Distanzen-Panel: immer sichtbar wenn 2+ Nodes selektiert
            if selected_node_ids.len() >= 2 {
                if let Some(rm) = road_map {
                    render_distanzen_panel(ui, rm, selected_node_ids, distanzen, &mut events);
                }
            } else if distanzen.active {
                // Selektion verloren → Vorschau deaktivieren
                distanzen.deactivate();
            }

            ui.separator();
            // Node-Verhalten: immer sichtbar
            render_node_behavior_options(ui, options, &mut events);

            ui.separator();
            render_default_direction_selector(ui, default_direction, default_priority, &mut events);

            // Route-Tool-Konfiguration (Distanz/Anzahl-Slider wenn Tool aktiv)
            if active_tool == EditorTool::Route {
                events.extend(render_route_tool_config(ui, tool_manager));
            }
        });

    events
}

fn render_selection_info(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected: &HashSet<u64>,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    segment_registry: Option<&SegmentRegistry>,
    events: &mut Vec<AppIntent>,
) {
    match selected.len() {
        1 => {
            let node_id = *selected.iter().next().unwrap();
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
            let mut iter = selected.iter().copied();
            let a = iter.next().unwrap();
            let b = iter.next().unwrap();
            ui.label(format!("Nodes: {}, {}", a, b));
            ui.separator();

            let conns = road_map.find_connections_between(a, b);

            if conns.is_empty() {
                ui.label("Keine Verbindung");
                ui.separator();

                let direction = default_direction;
                let priority = default_priority;
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

                        let current_dir = conn.direction;
                        let start_id = conn.start_id;
                        let end_id = conn.end_id;

                        let mut selected_dir = current_dir;
                        egui::ComboBox::from_id_salt(format!("dir_{}_{}", start_id, end_id))
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

                        let current_prio = conn.priority;
                        let mut selected_prio = current_prio;
                        egui::ComboBox::from_id_salt(format!("prio_{}_{}", start_id, end_id))
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

    // Segment-Bearbeitung für alle Selektionsgrößen anbieten
    if let Some(registry) = segment_registry {
        let matching = registry.find_by_node_ids(selected);
        if !matching.is_empty() {
            ui.separator();
            ui.label("Segment bearbeiten:");
            for record in matching {
                let label = match &record.kind {
                    crate::app::segment_registry::SegmentKind::Straight { .. } => {
                        "✏ Gerade Strecke"
                    }
                    crate::app::segment_registry::SegmentKind::CurveQuad { .. } => "✏ Kurve Grad 2",
                    crate::app::segment_registry::SegmentKind::CurveCubic { .. } => {
                        "✏ Kurve Grad 3"
                    }
                    crate::app::segment_registry::SegmentKind::Spline { .. } => "✏ Spline",
                };
                if ui.button(label).clicked() {
                    events.push(AppIntent::EditSegmentRequested {
                        record_id: record.id,
                    });
                }
            }
        }
    }
}

fn render_route_tool_config(
    ui: &mut egui::Ui,
    tool_manager: Option<&mut ToolManager>,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    ui.separator();
    ui.heading("Route-Tool");

    if let Some(tool) = tool_manager
        .as_deref()
        .and_then(|manager| manager.active_tool())
    {
        ui.label(tool.status_text());
        ui.add_space(4.0);
    }

    if let Some(tool) = tool_manager.and_then(|manager| manager.active_tool_mut()) {
        let changed = tool.render_config(ui);
        if changed && tool.needs_recreate() {
            events.push(AppIntent::RouteToolConfigChanged);
        }
    }

    events
}

fn render_default_direction_selector(
    ui: &mut egui::Ui,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    events: &mut Vec<AppIntent>,
) {
    ui.label("Standard-Richtung:");
    let current = default_direction;
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
    let current_prio = default_priority;
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

/// Rendert die Node-Verhalten-Checkboxen (immer sichtbar im Side-Panel).
fn render_node_behavior_options(
    ui: &mut egui::Ui,
    options: &EditorOptions,
    events: &mut Vec<AppIntent>,
) {
    ui.heading("Node-Verhalten");

    // Checkbox A: Reconnect beim Löschen
    let mut reconnect = options.reconnect_on_delete;
    if ui
        .checkbox(&mut reconnect, "Nach Löschen verbinden")
        .on_hover_text(
            "Wenn aktiviert: Wird ein Node mit jeweils genau einem Vorgänger und Nachfolger \
             gelöscht, werden Vorgänger und Nachfolger direkt miteinander verbunden.",
        )
        .changed()
    {
        let mut new_options = options.clone();
        new_options.reconnect_on_delete = reconnect;
        events.push(AppIntent::OptionsChanged {
            options: new_options,
        });
    }

    // Checkbox B: Connection beim Platzieren aufteilen
    let mut split = options.split_connection_on_place;
    if ui
        .checkbox(&mut split, "Verbindung beim Platzieren teilen")
        .on_hover_text(
            "Wenn aktiviert: Wird ein neuer Node nahe einer bestehenden Verbindung \
             platziert, wird diese Verbindung durch den neuen Node aufgeteilt.",
        )
        .changed()
    {
        let mut new_options = options.clone();
        new_options.split_connection_on_place = split;
        events.push(AppIntent::OptionsChanged {
            options: new_options,
        });
    }
}

/// Rendert das Distanzen-Panel: Aktivierung, Spline-Vorschau und Resample-Steuerung.
///
/// Ablauf: Button aktiviert Vorschau → Werte live anpassen → Enter übernimmt, Esc verwirft.
fn render_distanzen_panel(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected_node_ids: &HashSet<u64>,
    distanzen: &mut crate::app::state::DistanzenState,
    events: &mut Vec<AppIntent>,
) {
    use crate::shared::spline_geometry::{catmull_rom_chain_with_tangents, polyline_length};

    // Kette ordnen und Streckenlänge berechnen
    let chain = order_chain_for_distanzen(selected_node_ids, road_map);
    let Some(ordered) = chain else {
        if distanzen.active {
            distanzen.deactivate();
        }
        ui.separator();
        ui.label("⚠ Selektierte Nodes bilden keine zusammenhängende Kette.");
        return;
    };

    let positions: Vec<glam::Vec2> = ordered
        .iter()
        .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
        .collect();

    if positions.len() < 2 {
        if distanzen.active {
            distanzen.deactivate();
        }
        return;
    }

    let dense = catmull_rom_chain_with_tangents(&positions, 16, None, None);
    let path_len = polyline_length(&dense);
    distanzen.path_length = path_len;

    ui.separator();
    ui.heading("Strecke aufteilen");
    ui.label(format!("Streckenlänge: {:.1} m", path_len));

    if !distanzen.active {
        // Inaktiv: Button zum Aktivieren
        if ui.button("▶ Vorschau starten").clicked() {
            distanzen.active = true;
            distanzen.distance = distanzen.distance.max(6.0);
            if distanzen.count < 2 {
                distanzen.sync_from_distance();
            }
            distanzen.preview_positions = compute_resample_preview(&dense, distanzen);
        }
        return;
    }

    // — Aktiv: Steuerung + Vorschau-Update —
    distanzen.distance = distanzen.distance.max(6.0);

    let prev_distance = distanzen.distance;
    ui.horizontal(|ui| {
        ui.label("Abstand:");
        ui.add(
            egui::DragValue::new(&mut distanzen.distance)
                .speed(0.5)
                .range(6.0..=500.0)
                .suffix(" m"),
        );
    });
    if (distanzen.distance - prev_distance).abs() > f32::EPSILON {
        distanzen.by_count = false;
        distanzen.sync_from_distance();
    }

    let prev_count = distanzen.count;
    ui.horizontal(|ui| {
        ui.label("Nodes:");
        ui.add(
            egui::DragValue::new(&mut distanzen.count)
                .speed(1.0)
                .range(2..=10000),
        );
    });
    if distanzen.count != prev_count {
        distanzen.by_count = true;
        distanzen.sync_from_count();
        if distanzen.distance < 6.0 {
            distanzen.distance = 6.0;
            distanzen.sync_from_distance();
        }
    }

    // Vorschau bei Parameteränderung aktualisieren
    let changed =
        (distanzen.distance - prev_distance).abs() > f32::EPSILON || distanzen.count != prev_count;
    if changed || distanzen.preview_positions.is_empty() {
        distanzen.preview_positions = compute_resample_preview(&dense, distanzen);
    }

    ui.add_space(4.0);
    ui.checkbox(&mut distanzen.hide_original, "Originale ausblenden");
    ui.label(format!(
        "Vorschau: {} Nodes",
        distanzen.preview_positions.len()
    ));
    ui.label("⏎ Enter → übernehmen  |  Esc → verwerfen");

    // Enter → Resample ausführen + deaktivieren
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        events.push(AppIntent::ResamplePathRequested);
        distanzen.deactivate();
    }

    // Esc → Vorschau verwerfen
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        distanzen.deactivate();
    }
}

/// Berechnet die Vorschau-Positionen für das Resample.
fn compute_resample_preview(
    dense: &[glam::Vec2],
    distanzen: &crate::app::state::DistanzenState,
) -> Vec<glam::Vec2> {
    use crate::shared::spline_geometry::{polyline_length, resample_by_distance};
    if distanzen.by_count {
        let n = distanzen.count.max(2) as usize;
        let total = polyline_length(dense);
        let step = total / (n - 1) as f32;
        resample_by_distance(dense, step)
    } else {
        let d = distanzen.distance.max(0.1);
        resample_by_distance(dense, d)
    }
}

/// Ordnet selektierte Nodes zu einer linearen Kette (für Distanzen-Vorschau).
fn order_chain_for_distanzen(node_ids: &HashSet<u64>, road_map: &RoadMap) -> Option<Vec<u64>> {
    let start = node_ids
        .iter()
        .find(|&&id| {
            road_map
                .connections_iter()
                .filter(|c| c.end_id == id && node_ids.contains(&c.start_id))
                .count()
                == 0
        })
        .copied()
        .or_else(|| node_ids.iter().next().copied())?;

    let mut path = Vec::with_capacity(node_ids.len());
    let mut visited = HashSet::new();
    let mut current = start;

    loop {
        path.push(current);
        visited.insert(current);

        let next = road_map
            .connections_iter()
            .find(|c| {
                c.start_id == current
                    && node_ids.contains(&c.end_id)
                    && !visited.contains(&c.end_id)
            })
            .map(|c| c.end_id);

        match next {
            Some(n) => current = n,
            None => break,
        }
    }

    if path.len() == node_ids.len() {
        Some(path)
    } else {
        None
    }
}
