use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

use crate::app::state::DistanzenState;

/// Maximale Anzahl selektierter Nodes fuer die Ketten-Analyse.
/// Oberhalb dieses Limits wird die O(N·C)-Berechnung uebersprungen.
const MAX_CHAIN_NODES: usize = 500;
use crate::app::tools::common::wheel_dir;
use crate::app::{AppIntent, RoadMap};

/// Rendert das Distanzen-Panel: Aktivierung, Spline-Vorschau und Resample-Steuerung.
///
/// Ablauf: Button aktiviert Vorschau → Werte live anpassen → Enter uebernimmt, Esc verwirft.
pub fn render_distance_panel(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    distance_state: &mut DistanzenState,
    distance_wheel_step_m: f32,
    events: &mut Vec<AppIntent>,
) {
    use crate::shared::spline_geometry::{catmull_rom_chain_with_tangents, polyline_length};

    // Sicherheits-Limit: Ketten-Analyse ist O(N·C) und wird bei grosser Selektion uebersprungen
    if selected_node_ids.len() > MAX_CHAIN_NODES {
        if distance_state.active {
            distance_state.deactivate();
        }
        ui.separator();
        ui.label(format!(
            "⚠ Ketten-Analyse: zu viele Nodes ({} > {MAX_CHAIN_NODES}).",
            selected_node_ids.len()
        ));
        return;
    }

    let chain = order_chain_for_distance(selected_node_ids, road_map);
    let Some(ordered) = chain else {
        if distance_state.active {
            distance_state.deactivate();
        }
        ui.separator();
        ui.label("⚠ Selektierte Nodes bilden keine zusammenhaengende Kette.");
        return;
    };

    let positions: Vec<glam::Vec2> = ordered
        .iter()
        .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
        .collect();

    if positions.len() < 2 {
        if distance_state.active {
            distance_state.deactivate();
        }
        return;
    }

    let dense = catmull_rom_chain_with_tangents(&positions, 16, None, None);
    let path_len = polyline_length(&dense);
    distance_state.path_length = path_len;

    ui.separator();
    ui.heading("Streckenteilung");
    ui.label(format!("Streckenlaenge: {:.1} m", path_len));

    if !distance_state.active {
        if ui.button("▶ Einteilung aendern").clicked() {
            distance_state.active = true;
            distance_state.distance = distance_state.distance.max(1.0);
            if distance_state.count < 2 {
                distance_state.sync_from_distance();
            }
            distance_state.preview_positions = compute_resample_preview(&dense, distance_state);
        }
        return;
    }

    distance_state.distance = distance_state.distance.max(1.0);

    let prev_distance = distance_state.distance;
    ui.horizontal(|ui| {
        ui.label("Abstand:");
        let response = ui.add(
            egui::DragValue::new(&mut distance_state.distance)
                .speed(0.5)
                .range(1.0..=25.0)
                .suffix(" m"),
        );
        let wheel_dir = wheel_dir(ui, &response);
        if distance_wheel_step_m > 0.0 && wheel_dir != 0.0 {
            distance_state.distance =
                (distance_state.distance + wheel_dir * distance_wheel_step_m).clamp(1.0, 25.0);
        }
    });
    if (distance_state.distance - prev_distance).abs() > f32::EPSILON {
        distance_state.by_count = false;
        distance_state.sync_from_distance();
    }

    let prev_count = distance_state.count;
    ui.horizontal(|ui| {
        ui.label("Nodes:");
        let response = ui.add(
            egui::DragValue::new(&mut distance_state.count)
                .speed(1.0)
                .range(2..=10000),
        );
        let wheel_dir = wheel_dir(ui, &response);
        if distance_wheel_step_m > 0.0 && wheel_dir > 0.0 {
            distance_state.count = distance_state.count.saturating_add(1).min(10_000);
        } else if distance_wheel_step_m > 0.0 && wheel_dir < 0.0 {
            distance_state.count = distance_state.count.saturating_sub(1).max(2);
        }
    });
    if distance_state.count != prev_count {
        distance_state.by_count = true;
        distance_state.sync_from_count();
        if distance_state.distance < 1.0 {
            distance_state.distance = 1.0;
            distance_state.sync_from_distance();
        }
    }

    let changed = (distance_state.distance - prev_distance).abs() > f32::EPSILON
        || distance_state.count != prev_count;
    if changed || distance_state.preview_positions.is_empty() {
        distance_state.preview_positions = compute_resample_preview(&dense, distance_state);
    }

    ui.add_space(4.0);
    ui.checkbox(&mut distance_state.hide_original, "Originale ausblenden");
    ui.label(format!(
        "Vorschau: {} Nodes",
        distance_state.preview_positions.len()
    ));
    ui.label("⏎ Enter → uebernehmen  |  Esc → verwerfen");

    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        events.push(AppIntent::ResamplePathRequested);
        distance_state.deactivate();
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        distance_state.deactivate();
    }
}

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

/// Baut eine geordnete Kette aus selektierten Nodes anhand der Verbindungsrichtung.
///
/// Algorithmus: Adjacency-Map O(C) einmalig aufbauen, danach Chain-Building O(N).
/// Vorbedingung: `node_ids.len() <= MAX_CHAIN_NODES` (Aufrufer prueft dies).
fn order_chain_for_distance(node_ids: &IndexSet<u64>, road_map: &RoadMap) -> Option<Vec<u64>> {
    // Adjacency-Map: start_id → end_id, gefiltert auf selektierte Nodes (O(C) einmalig)
    let node_set: HashSet<u64> = node_ids.iter().copied().collect();
    let forward: HashMap<u64, u64> = road_map
        .connections_iter()
        .filter(|c| node_set.contains(&c.start_id) && node_set.contains(&c.end_id))
        .map(|c| (c.start_id, c.end_id))
        .collect();

    // Start-Node: kein eingehender Pfeil im selektierten Subgraph
    let has_incoming: HashSet<u64> = forward.values().copied().collect();
    let start = node_ids
        .iter()
        .find(|&&id| !has_incoming.contains(&id))
        .copied()
        .or_else(|| node_ids.iter().next().copied())?;

    let mut path = Vec::with_capacity(node_ids.len());
    let mut visited = HashSet::new();
    let mut current = start;

    // Chain-Building via Adjacency-Map: O(N) statt O(N·C)
    loop {
        path.push(current);
        visited.insert(current);

        match forward.get(&current).copied() {
            Some(next) if !visited.contains(&next) => current = next,
            _ => break,
        }
    }

    if path.len() == node_ids.len() {
        Some(path)
    } else {
        None
    }
}
