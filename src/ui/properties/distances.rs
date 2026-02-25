use std::collections::HashSet;

use crate::app::state::DistanzenState;
use crate::app::{AppIntent, RoadMap};

/// Rendert das Distanzen-Panel: Aktivierung, Spline-Vorschau und Resample-Steuerung.
///
/// Ablauf: Button aktiviert Vorschau → Werte live anpassen → Enter übernimmt, Esc verwirft.
pub fn render_distance_panel(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected_node_ids: &HashSet<u64>,
    distance_state: &mut DistanzenState,
    events: &mut Vec<AppIntent>,
) {
    use crate::shared::spline_geometry::{catmull_rom_chain_with_tangents, polyline_length};

    let chain = order_chain_for_distance(selected_node_ids, road_map);
    let Some(ordered) = chain else {
        if distance_state.active {
            distance_state.deactivate();
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
    ui.label(format!("Streckenlänge: {:.1} m", path_len));

    if !distance_state.active {
        if ui.button("▶ Einteilung ändern").clicked() {
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
        ui.add(
            egui::DragValue::new(&mut distance_state.distance)
                .speed(0.5)
                .range(1.0..=25.0)
                .suffix(" m"),
        );
    });
    if (distance_state.distance - prev_distance).abs() > f32::EPSILON {
        distance_state.by_count = false;
        distance_state.sync_from_distance();
    }

    let prev_count = distance_state.count;
    ui.horizontal(|ui| {
        ui.label("Nodes:");
        ui.add(
            egui::DragValue::new(&mut distance_state.count)
                .speed(1.0)
                .range(2..=10000),
        );
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
    ui.label("⏎ Enter → übernehmen  |  Esc → verwerfen");

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

fn order_chain_for_distance(node_ids: &HashSet<u64>, road_map: &RoadMap) -> Option<Vec<u64>> {
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
