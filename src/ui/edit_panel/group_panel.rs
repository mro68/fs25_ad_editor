use crate::app::state::GroupEditState;
use crate::app::{AppIntent, GroupRecord, RoadMap};
use crate::shared::EditorOptions;

/// Gruppen-Edit-Panel: Anzeige aktiver Edit-Modus mit Uebernehmen/Abbrechen.
/// Zeigt ausserdem ComboBoxen fuer Einfahrt- und Ausfahrt-Node-Zuweisung.
pub(super) fn render_group_edit_panel(
    ctx: &egui::Context,
    edit_state: &GroupEditState,
    group_record: Option<&GroupRecord>,
    road_map: Option<&RoadMap>,
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
        if let Some(rec) = group_record {
            if rec.is_tool_editable() && ui.button("🔧 Tool bearbeiten").clicked() {
                events.push(AppIntent::GroupEditToolRequested {
                    record_id: edit_state.record_id,
                });
            }
        }
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

        if let Some(record) = group_record {
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label("Einfahrt / Ausfahrt:");
            ui.add_space(2.0);

            let current_entry = record.entry_node_id;
            let current_exit = record.exit_node_id;
            let mut entry_sel = current_entry;
            let mut exit_sel = current_exit;

            ui.horizontal(|ui| {
                ui.label("Einfahrt:");
                egui::ComboBox::from_id_salt("grp_edit_entry")
                    .selected_text(format_node_label(entry_sel, road_map))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut entry_sel, None, "Keine");
                        for &nid in &record.node_ids {
                            let label = format_node_label(Some(nid), road_map);
                            ui.selectable_value(&mut entry_sel, Some(nid), label);
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Ausfahrt:");
                egui::ComboBox::from_id_salt("grp_edit_exit")
                    .selected_text(format_node_label(exit_sel, road_map))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut exit_sel, None, "Keine");
                        for &nid in &record.node_ids {
                            let label = format_node_label(Some(nid), road_map);
                            ui.selectable_value(&mut exit_sel, Some(nid), label);
                        }
                    });
            });

            if entry_sel != current_entry || exit_sel != current_exit {
                events.push(AppIntent::SetGroupBoundaryNodes {
                    record_id: record.id,
                    entry_node_id: entry_sel,
                    exit_node_id: exit_sel,
                });
            }
        }
    });
}

fn format_node_label(node_id: Option<u64>, road_map: Option<&RoadMap>) -> String {
    let Some(id) = node_id else {
        return "Keine".to_string();
    };
    let pos = road_map.and_then(|rm| rm.node(id)).map(|n| n.position);
    match pos {
        Some(p) => format!("#{} ({:.0}, {:.0})", id, p.x, p.y),
        None => format!("#{}", id),
    }
}
