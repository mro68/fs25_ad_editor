//! Event-Sammlung fuer Panels, Toolbar und die rechte Seitenleiste.

use crate::app::ui_contract::HostUiSnapshot;
use crate::shared::EditorOptions;
use crate::ui;
use eframe::egui;
use fs25_auto_drive_host_bridge::{HostChromeSnapshot, HostMarkerListSnapshot};

use super::{map_intent_to_collected_event, CollectedEvent, EditorApp};

impl EditorApp {
    /// Sammelt Events aus Menue, Toolbar und Properties-Panel.
    pub(super) fn collect_panel_events(
        &mut self,
        ctx: &egui::Context,
        host_ui_snapshot: &HostUiSnapshot,
        host_chrome_snapshot: &HostChromeSnapshot,
        marker_list: &HostMarkerListSnapshot,
        top_ui: &mut egui::Ui,
    ) -> Vec<CollectedEvent> {
        let mut events = Vec::new();
        let should_close_floating_menu = {
            ui::status::render_status_bar_inside(top_ui, host_chrome_snapshot);
            events.extend(
                ui::menu::render_menu_inside(top_ui, host_chrome_snapshot)
                    .into_iter()
                    .map(map_intent_to_collected_event),
            );
            let (floating_events, should_close) = ui::render_floating_menu(
                ctx,
                self.session.chrome_state().floating_menu,
                host_chrome_snapshot,
            );
            events.extend(
                floating_events
                    .into_iter()
                    .map(map_intent_to_collected_event),
            );
            events.extend(
                ui::defaults_panel::render_route_defaults_panel_inside(
                    top_ui,
                    host_chrome_snapshot,
                )
                .into_iter()
                .map(map_intent_to_collected_event),
            );
            should_close
        };
        if should_close_floating_menu {
            self.session.clear_floating_menu();
        }

        let route_tool_panel = host_ui_snapshot.route_tool_panel_state().cloned();
        let selected_node_ids: Vec<u64> = self
            .session
            .app_state()
            .selection
            .selected_node_ids
            .iter()
            .copied()
            .collect();
        let node_details = selected_node_ids
            .first()
            .and_then(|&node_id| self.session.node_details(node_id));
        let connection_pair = match selected_node_ids.as_slice() {
            [node_a, node_b] => Some(self.session.connection_pair(*node_a, *node_b)),
            _ => None,
        };
        let panel_state = self.session.panel_properties_state_mut();
        let distance_wheel_step_m = numeric_distance_wheel_step(panel_state.options);
        let lang = panel_state.options.language;

        // Rechte Sidebar: Marker + Eigenschaften untereinander, einklappbar
        // (muss vor CentralPanel aufgerufen werden)
        egui::Panel::right("right_sidebar")
            .resizable(true)
            .default_size(200.0)
            .min_size(160.0)
            .show_inside(top_ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::CollapsingHeader::new("Marker")
                        .default_open(true)
                        .show(ui, |ui| {
                            events.extend(
                                ui::render_marker_content(
                                    ui,
                                    marker_list,
                                    panel_state.road_map.is_some(),
                                )
                                .into_iter()
                                .map(map_intent_to_collected_event),
                            );
                        });

                    ui.separator();

                    egui::CollapsingHeader::new("Eigenschaften")
                        .default_open(true)
                        .show(ui, |ui| {
                            events.extend(
                                ui::render_properties_content(
                                    ui,
                                    panel_state.road_map,
                                    panel_state.selected_node_ids,
                                    node_details.as_ref(),
                                    connection_pair.as_ref(),
                                    panel_state.default_direction,
                                    panel_state.default_priority,
                                    distance_wheel_step_m,
                                    Some(panel_state.group_registry),
                                    Some(panel_state.tool_edit_store),
                                    panel_state.distanzen,
                                )
                                .into_iter()
                                .map(map_intent_to_collected_event),
                            );
                        });
                });
            });

        // Floating Edit-Panel (Streckenteilung / Route-Tool)
        let panel_pos = self
            .input
            .edit_panel_pos
            .map(|p| egui::Pos2::new(p[0], p[1]));
        let group_record = panel_state
            .group_editing
            .and_then(|edit_state| panel_state.group_registry.get(edit_state.record_id));
        events.extend(
            ui::render_edit_panel(
                ctx,
                panel_state.road_map,
                panel_state.selected_node_ids,
                panel_state.distanzen,
                panel_state.default_direction,
                panel_state.default_priority,
                distance_wheel_step_m,
                panel_state.active_tool,
                route_tool_panel,
                panel_pos,
                panel_state.group_editing,
                group_record,
                Some(panel_state.tool_edit_store),
                panel_state.options,
                lang,
            )
            .into_iter()
            .map(map_intent_to_collected_event),
        );

        events
    }
}

/// Berechnet die Schrittweite fuer numerische Distanz-Eingaben aus den Editor-Optionen.
pub(super) fn numeric_distance_wheel_step(options: &EditorOptions) -> f32 {
    options.mouse_wheel_distance_step_m.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::numeric_distance_wheel_step;
    use crate::shared::{EditorOptions, ValueAdjustInputMode};

    fn options_with(mode: ValueAdjustInputMode, step: f32) -> EditorOptions {
        EditorOptions {
            value_adjust_input_mode: mode,
            mouse_wheel_distance_step_m: step,
            ..EditorOptions::default()
        }
    }

    #[test]
    fn numeric_distance_wheel_step_is_independent_from_input_mode() {
        let drag_options = options_with(ValueAdjustInputMode::DragHorizontal, 0.25);
        let wheel_options = options_with(ValueAdjustInputMode::MouseWheel, 0.25);

        assert_eq!(numeric_distance_wheel_step(&drag_options), 0.25);
        assert_eq!(numeric_distance_wheel_step(&wheel_options), 0.25);
    }

    #[test]
    fn numeric_distance_wheel_step_clamps_negative_values() {
        let options = options_with(ValueAdjustInputMode::DragHorizontal, -0.25);
        assert_eq!(numeric_distance_wheel_step(&options), 0.0);
    }
}
