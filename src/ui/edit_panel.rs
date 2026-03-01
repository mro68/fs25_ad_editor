//! Schwebendes Bearbeitungspanel für aktive Edit-Modi.
//!
//! Wird über dem Viewport angezeigt, wenn ein Edit-Modus aktiv ist
//! (Streckenteilung, Route-Tool). Zeigt nur die modi-spezifischen
//! Einstellungen mit Übernehmen/Abbrechen-Buttons.

use crate::app::state::DistanzenState;
use crate::app::tools::ToolManager;
use crate::app::{AppIntent, EditorTool, RoadMap};
use std::collections::HashSet;

/// Rendert das Floating-Edit-Panel und gibt erzeugte Events zurück.
///
/// Das Panel erscheint an `panel_pos` (Bildschirmkoordinaten) und zeigt
/// nur die Steuerung für den gerade aktiven Edit-Modus.
pub fn render_edit_panel(
    ctx: &egui::Context,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    distanzen_state: &mut DistanzenState,
    active_tool: EditorTool,
    tool_manager: Option<&mut ToolManager>,
    panel_pos: Option<egui::Pos2>,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    // Streckenteilung Edit-Modus
    if distanzen_state.active {
        render_streckenteilung_panel(
            ctx,
            road_map,
            selected_node_ids,
            distanzen_state,
            panel_pos,
            &mut events,
        );
        return events;
    }

    // Route-Tool Edit-Modus (nur wenn Tool aktiv mit pending input)
    if active_tool == EditorTool::Route {
        if let Some(manager) = tool_manager {
            let has_input = manager
                .active_tool()
                .map(|t| t.has_pending_input())
                .unwrap_or(false);
            if has_input {
                render_route_tool_panel(ctx, manager, panel_pos, &mut events);
            }
        }
    }

    events
}

/// Streckenteilung-Panel: Abstand/Nodes + Vorschau + Übernehmen/Verwerfen.
fn render_streckenteilung_panel(
    ctx: &egui::Context,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    distanzen_state: &mut DistanzenState,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
    use crate::shared::spline_geometry::{catmull_rom_chain_with_tangents, polyline_length};

    // Ketten-basierte Spline-Berechnung für Vorschau
    if let Some(rm) = road_map {
        let chain = order_chain(selected_node_ids, rm);
        if let Some(ordered) = chain {
            let positions: Vec<glam::Vec2> = ordered
                .iter()
                .filter_map(|id| rm.nodes.get(id).map(|n| n.position))
                .collect();

            if positions.len() >= 2 {
                let dense = catmull_rom_chain_with_tangents(&positions, 16, None, None);
                let path_len = polyline_length(&dense);
                distanzen_state.path_length = path_len;

                // Vorschau aktualisieren
                distanzen_state.preview_positions =
                    compute_resample_preview(&dense, distanzen_state);
            }
        } else {
            // Kette aufgelöst → deaktivieren
            distanzen_state.deactivate();
            return;
        }
    }

    let mut window = egui::Window::new("📏 Streckenteilung")
        .collapsible(false)
        .resizable(false)
        .auto_sized();

    if let Some(pos) = panel_pos {
        window = window.default_pos(pos);
    }

    window.show(ctx, |ui| {
        ui.label(format!(
            "Streckenlänge: {:.1} m",
            distanzen_state.path_length
        ));
        ui.add_space(4.0);

        render_streckenteilung_controls(ui, distanzen_state);

        ui.add_space(4.0);
        ui.checkbox(&mut distanzen_state.hide_original, "Originale ausblenden");
        ui.label(format!(
            "Vorschau: {} Nodes",
            distanzen_state.preview_positions.len()
        ));

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("✓ Übernehmen").clicked() {
                events.push(AppIntent::ResamplePathRequested);
                distanzen_state.deactivate();
            }
            if ui.button("✕ Verwerfen").clicked() {
                distanzen_state.deactivate();
            }
        });

        // Keyboard-Shortcuts
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            events.push(AppIntent::ResamplePathRequested);
            distanzen_state.deactivate();
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            distanzen_state.deactivate();
        }
    });
}

/// Gemeinsame Steuerelemente für Streckenteilung (Abstand + Nodes DragValues).
///
/// Wird sowohl vom Floating-Panel als auch vom Properties-Panel verwendet.
pub fn render_streckenteilung_controls(ui: &mut egui::Ui, distanzen_state: &mut DistanzenState) {
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
}

/// Route-Tool-Panel: Tool-Config + Ausführen/Abbrechen.
fn render_route_tool_panel(
    ctx: &egui::Context,
    tool_manager: &mut ToolManager,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
    let mut window = egui::Window::new("📐 Route-Tool")
        .collapsible(false)
        .resizable(false)
        .auto_sized();

    if let Some(pos) = panel_pos {
        window = window.default_pos(pos);
    }

    window.show(ctx, |ui| {
        if let Some(tool) = tool_manager.active_tool() {
            ui.label(tool.status_text());
        }

        if let Some(tool) = tool_manager.active_tool_mut() {
            let changed = tool.render_config(ui);
            if changed && tool.needs_recreate() {
                events.push(AppIntent::RouteToolConfigChanged);
            }
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("✓ Ausführen").clicked() {
                events.push(AppIntent::RouteToolExecuteRequested);
            }
            if ui.button("✕ Abbrechen").clicked() {
                events.push(AppIntent::RouteToolCancelled);
            }
        });
    });
}

/// Ordnet selektierte Node-IDs als zusammenhängende Kette.
fn order_chain(node_ids: &HashSet<u64>, road_map: &RoadMap) -> Option<Vec<u64>> {
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

/// Berechnet die Vorschau-Positionen für die Streckenteilung.
fn compute_resample_preview(
    dense: &[glam::Vec2],
    distance_state: &DistanzenState,
) -> Vec<glam::Vec2> {
    use crate::shared::spline_geometry::{polyline_length, resample_by_distance};
    if distance_state.by_count {
        let n = distance_state.count.max(2) as usize;
        let total = polyline_length(dense);
        let step = total / (n - 1) as f32;
        resample_by_distance(dense, step)
    } else {
        let d = distance_state.distance.max(0.1);
        resample_by_distance(dense, d)
    }
}
