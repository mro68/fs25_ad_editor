//! Schwebendes Bearbeitungspanel fuer aktive Edit-Modi.
//!
//! Wird ueber dem Viewport angezeigt, wenn ein Edit-Modus aktiv ist
//! (Streckenteilung, Route-Tool). Zeigt nur die modi-spezifischen
//! Einstellungen mit Uebernehmen/Abbrechen-Buttons.

use crate::app::state::DistanzenState;
use crate::app::state::GroupEditState;
use crate::app::tools::common::wheel_dir;
use crate::app::tools::ToolManager;
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, EditorTool, RoadMap};
use crate::shared::EditorOptions;
use crate::ui::properties::selectors::{
    render_direction_icon_selector, render_priority_icon_selector,
};
use indexmap::IndexSet;
use std::hash::{Hash, Hasher};

/// Rendert das Floating-Edit-Panel und gibt erzeugte Events zurueck.
///
/// Das Panel erscheint an `panel_pos` (Bildschirmkoordinaten) und zeigt
/// nur die Steuerung fuer den gerade aktiven Edit-Modus.
#[allow(clippy::too_many_arguments)]
pub fn render_edit_panel(
    ctx: &egui::Context,
    road_map: Option<&RoadMap>,
    selected_node_ids: &IndexSet<u64>,
    distanzen_state: &mut DistanzenState,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    distance_wheel_step_m: f32,
    active_tool: EditorTool,
    tool_manager: Option<&mut ToolManager>,
    panel_pos: Option<egui::Pos2>,
    group_editing: Option<&GroupEditState>,
    options: &mut EditorOptions,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    // Gruppen-Edit-Panel (hat Vorrang vor Streckenteilung)
    if let Some(edit_state) = group_editing {
        render_group_edit_panel(ctx, edit_state, panel_pos, options, &mut events);
        return events;
    }

    // Streckenteilung Edit-Modus
    if distanzen_state.active {
        render_streckenteilung_panel(
            ctx,
            road_map,
            selected_node_ids,
            distanzen_state,
            distance_wheel_step_m,
            panel_pos,
            &mut events,
        );
        return events;
    }

    // Route-Tool Edit-Modus (immer wenn Tool aktiv)
    if active_tool == EditorTool::Route {
        if let Some(manager) = tool_manager {
            render_route_tool_panel(
                ctx,
                manager,
                default_direction,
                default_priority,
                distance_wheel_step_m,
                panel_pos,
                &mut events,
            );
        }
    }

    events
}

/// Gruppen-Edit-Panel: Anzeige aktiver Edit-Modus mit Uebernehmen/Abbrechen.
fn render_group_edit_panel(
    ctx: &egui::Context,
    edit_state: &GroupEditState,
    panel_pos: Option<egui::Pos2>,
    options: &mut EditorOptions,
    events: &mut Vec<AppIntent>,
) {
    let mut window = egui::Window::new("✏ Gruppen-Bearbeitung")
        .collapsible(false)
        .resizable(false)
        .auto_sized();

    if let Some(pos) = panel_pos {
        window = window.default_pos(pos);
    }

    window.show(ctx, |ui| {
        ui.label(format!("Gruppe #{} bearbeiten", edit_state.record_id));
        ui.label("Nodes verschieben, hinzufuegen oder loeschen.");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button("✓ Uebernehmen").clicked() {
                events.push(AppIntent::GroupEditApplyRequested);
            }
            if ui.button("✕ Abbrechen").clicked() {
                events.push(AppIntent::GroupEditCancelRequested);
            }
        });
        // Keyboard-Shortcuts
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            events.push(AppIntent::GroupEditApplyRequested);
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            events.push(AppIntent::GroupEditCancelRequested);
        }
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        ui.checkbox(
            &mut options.show_all_group_boundaries,
            "Rand-Icons an allen Gruppen-Grenzknoten anzeigen",
        );
    });
}

/// Streckenteilung-Panel: Abstand/Nodes + Vorschau + Uebernehmen/Verwerfen.
fn render_streckenteilung_panel(
    ctx: &egui::Context,
    road_map: Option<&RoadMap>,
    selected_node_ids: &IndexSet<u64>,
    distanzen_state: &mut DistanzenState,
    distance_wheel_step_m: f32,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
    use crate::shared::spline_geometry::{catmull_rom_chain_with_tangents, polyline_length};

    // Ketten-basierte Spline-Berechnung fuer Vorschau
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
                    .filter_map(|id| rm.nodes.get(id).map(|n| n.position))
                    .collect();

                if positions.len() >= 2 {
                    let dense = catmull_rom_chain_with_tangents(&positions, 16, None, None);
                    let path_len = polyline_length(&dense);
                    distanzen_state.path_length = path_len;

                    // Vorschau aktualisieren
                    distanzen_state.preview_positions =
                        compute_resample_preview(&dense, distanzen_state);
                    distanzen_state.preview_cache_signature = signature;
                }
            }
        } else {
            // Kette aufgeloest → deaktivieren
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
        let wheel_dir = wheel_dir(ui, &response);
        if distance_wheel_step_m > 0.0 && wheel_dir != 0.0 {
            distanzen_state.distance =
                (distanzen_state.distance + wheel_dir * distance_wheel_step_m).clamp(1.0, 25.0);
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
        let wheel_dir = wheel_dir(ui, &response);
        if distance_wheel_step_m > 0.0 && wheel_dir > 0.0 {
            distanzen_state.count = distanzen_state.count.saturating_add(1).min(10_000);
        } else if distance_wheel_step_m > 0.0 && wheel_dir < 0.0 {
            distanzen_state.count = distanzen_state.count.saturating_sub(1).max(2);
        }
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

/// Route-Tool-Panel: Tool-Config + Ausfuehren/Abbrechen.
#[allow(clippy::too_many_arguments)]
fn render_route_tool_panel(
    ctx: &egui::Context,
    tool_manager: &mut ToolManager,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    distance_wheel_step_m: f32,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
    let has_pending_input = tool_manager
        .active_tool()
        .map(|tool| tool.has_pending_input())
        .unwrap_or(false);

    let mut window = egui::Window::new("📐 Route-Tool")
        .collapsible(false)
        .resizable(false)
        .default_width(360.0)
        .min_width(320.0)
        .max_width(420.0)
        .auto_sized();

    if let Some(pos) = panel_pos {
        window = window.default_pos(pos);
    }

    window.show(ctx, |ui| {
        // Breite stabil halten, damit lange Eintraege das Fenster nicht ueberdehnen.
        ui.set_min_width(320.0);
        ui.set_max_width(420.0);

        if let Some(tool) = tool_manager.active_tool() {
            ui.label(tool.status_text());
        }

        ui.add_space(6.0);
        let mut selected_dir = default_direction;
        render_direction_icon_selector(ui, &mut selected_dir, "route_tool_floating");
        if selected_dir != default_direction {
            events.push(AppIntent::SetDefaultDirectionRequested {
                direction: selected_dir,
            });
        }

        ui.add_space(4.0);
        let mut selected_prio = default_priority;
        render_priority_icon_selector(ui, &mut selected_prio, "route_tool_floating");
        if selected_prio != default_priority {
            events.push(AppIntent::SetDefaultPriorityRequested {
                priority: selected_prio,
            });
        }

        ui.add_space(6.0);

        if let Some(tool) = tool_manager.active_tool_mut() {
            let changed = tool.render_config(ui, distance_wheel_step_m);
            if changed && tool.needs_recreate() {
                events.push(AppIntent::RouteToolConfigChanged);
            }
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(has_pending_input, egui::Button::new("✓ Ausfuehren"))
                .clicked()
            {
                events.push(AppIntent::RouteToolExecuteRequested);
            }
            if ui.button("✕ Abbrechen").clicked() {
                events.push(AppIntent::RouteToolCancelled);
            }
        });
    });
}

/// Berechnet die Vorschau-Positionen fuer die Streckenteilung.
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
