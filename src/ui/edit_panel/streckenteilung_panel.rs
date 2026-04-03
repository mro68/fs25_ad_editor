use crate::app::state::DistanzenState;
use crate::app::{AppIntent, RoadMap};
use crate::ui::common::{apply_wheel_step, apply_wheel_step_usize};
use indexmap::IndexSet;
use std::hash::{Hash, Hasher};

/// Streckenteilung-Panel: Abstand/Nodes + Vorschau + Uebernehmen/Verwerfen.
pub(super) fn render_streckenteilung_panel(
    ctx: &egui::Context,
    road_map: Option<&RoadMap>,
    selected_node_ids: &IndexSet<u64>,
    distanzen_state: &mut DistanzenState,
    distance_wheel_step_m: f32,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
    use crate::shared::spline_geometry::{catmull_rom_chain_with_tangents, polyline_length};

    if let Some(rm) = road_map {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        selected_node_ids.iter().for_each(|id| id.hash(&mut hasher));
        distanzen_state.by_count.hash(&mut hasher);
        distanzen_state.count.hash(&mut hasher);
        distanzen_state.distance.to_bits().hash(&mut hasher);
        let signature = hasher.finish();

        let chain = rm.ordered_chain_nodes(selected_node_ids);
        if let Some(ordered) = chain {
            if signature != distanzen_state.preview_cache_signature
                || distanzen_state.preview_positions.is_empty()
            {
                let positions: Vec<glam::Vec2> = ordered
                    .iter()
                    .filter_map(|id| rm.node(*id).map(|n| n.position))
                    .collect();

                if positions.len() >= 2 {
                    let dense = catmull_rom_chain_with_tangents(&positions, 16, None, None);
                    let path_len = polyline_length(&dense);
                    distanzen_state.path_length = path_len;
                    distanzen_state.preview_positions =
                        compute_resample_preview(&dense, distanzen_state);
                    distanzen_state.preview_cache_signature = signature;
                }
            }
        } else {
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
            "Streckenlaenge: {:.1} m",
            distanzen_state.path_length
        ));
        ui.add_space(4.0);

        render_streckenteilung_controls(ui, distanzen_state, distance_wheel_step_m);

        ui.add_space(4.0);
        ui.checkbox(&mut distanzen_state.hide_original, "Originale ausblenden");
        ui.label(format!(
            "Vorschau: {} Nodes",
            distanzen_state.preview_positions.len()
        ));

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("✓ Uebernehmen").clicked() {
                events.push(AppIntent::ResamplePathRequested);
                distanzen_state.deactivate();
            }
            if ui.button("✕ Verwerfen").clicked() {
                distanzen_state.deactivate();
            }
        });

        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            events.push(AppIntent::ResamplePathRequested);
            distanzen_state.deactivate();
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            distanzen_state.deactivate();
        }
    });
}

/// Gemeinsame Steuerelemente fuer Streckenteilung (Abstand + Nodes DragValues).
///
/// Wird sowohl vom Floating-Panel als auch vom Properties-Panel verwendet.
pub fn render_streckenteilung_controls(
    ui: &mut egui::Ui,
    distanzen_state: &mut DistanzenState,
    distance_wheel_step_m: f32,
) {
    let prev_distance = distanzen_state.distance;
    ui.horizontal(|ui| {
        ui.label("Abstand:");
        let response = ui.add(
            egui::DragValue::new(&mut distanzen_state.distance)
                .speed(0.5)
                .range(1.0..=25.0)
                .suffix(" m"),
        );
        if distance_wheel_step_m > 0.0 {
            let _ = apply_wheel_step(
                ui,
                &response,
                &mut distanzen_state.distance,
                distance_wheel_step_m,
                1.0..=25.0,
            );
        }
    });
    if (distanzen_state.distance - prev_distance).abs() > f32::EPSILON {
        distanzen_state.by_count = false;
        distanzen_state.sync_from_distance();
    }

    let prev_count = distanzen_state.count;
    ui.horizontal(|ui| {
        ui.label("Nodes:");
        let response = ui.add(
            egui::DragValue::new(&mut distanzen_state.count)
                .speed(1.0)
                .range(2..=10000),
        );
        let mut count = distanzen_state.count as usize;
        let _ = apply_wheel_step_usize(
            ui,
            &response,
            &mut count,
            2..=10_000,
            distance_wheel_step_m > 0.0,
        );
        distanzen_state.count = count as u32;
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
