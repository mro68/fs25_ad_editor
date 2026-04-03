//! Properties-Panel (rechte Seitenleiste) für Node- und Connection-Eigenschaften.

mod distances;
pub(crate) mod selectors;

use indexmap::IndexSet;

use crate::app::{
    group_registry::GroupRegistry, tool_editing::ToolEditStore, tools::route_tool_descriptor,
    AppIntent, Connection, ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap,
};
use distances::render_distance_panel;
use selectors::{render_direction_icon_selector, render_priority_icon_selector};

struct SelectionInfoContext<'a> {
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    group_registry: Option<&'a GroupRegistry>,
    tool_edit_store: Option<&'a ToolEditStore>,
    events: &'a mut Vec<AppIntent>,
}

/// Rendert den Properties-Inhalt in den übergebenen UI-Bereich.
///
/// Gibt eine Liste von `AppIntent`-Events zurück, die bei Interaktion erzeugt werden.
#[allow(clippy::too_many_arguments)]
pub fn render_properties_content(
    ui: &mut egui::Ui,
    road_map: Option<&RoadMap>,
    selected_node_ids: &IndexSet<u64>,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    distance_wheel_step_m: f32,
    group_registry: Option<&GroupRegistry>,
    tool_edit_store: Option<&ToolEditStore>,
    distance_state: &mut crate::app::state::DistanzenState,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if selected_node_ids.is_empty() {
        ui.label("Keine Selektion");
    } else if let Some(road_map) = road_map {
        render_selection_info(
            ui,
            road_map,
            selected_node_ids,
            SelectionInfoContext {
                default_direction,
                default_priority,
                group_registry,
                tool_edit_store,
                events: &mut events,
            },
        );
    }

    // Distanzen-Panel: immer sichtbar wenn 2+ Nodes selektiert
    if selected_node_ids.len() >= 2 {
        if let Some(rm) = road_map {
            render_distance_panel(
                ui,
                rm,
                selected_node_ids,
                distance_state,
                distance_wheel_step_m,
                &mut events,
            );
        }
    } else if distance_state.active {
        // Selektion verloren → Vorschau deaktivieren
        distance_state.deactivate();
    }

    events
}

fn render_selection_info(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected: &IndexSet<u64>,
    context: SelectionInfoContext<'_>,
) {
    let SelectionInfoContext {
        default_direction,
        default_priority,
        group_registry,
        tool_edit_store,
        events,
    } = context;

    match selected.len() {
        1 => render_single_node_info(ui, road_map, selected, events),
        2 => render_two_nodes_info(
            ui,
            road_map,
            selected,
            default_direction,
            default_priority,
            events,
        ),
        n => {
            ui.label(format!("{} Nodes selektiert", n));
        }
    }

    render_segment_edit_buttons(ui, selected, group_registry, tool_edit_store, events);
}

/// Zeigt Einzelnode-Info: Position, Flag, Marker-Optionen.
fn render_single_node_info(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected: &IndexSet<u64>,
    events: &mut Vec<AppIntent>,
) {
    let node_id = *selected.iter().next().unwrap();
    let Some(node) = road_map.node(node_id) else {
        return;
    };

    ui.label(format!("Node ID: {}", node.id));
    ui.label(format!(
        "Position: ({:.1}, {:.1})",
        node.position.x, node.position.y
    ));

    // Editierbare Flags — nur Regular und SubPrio sind user-gesetzt.
    let editable_flags = [
        (NodeFlag::Regular, "Regular (Hauptstrasse)"),
        (NodeFlag::SubPrio, "SubPrio (Nebenstrasse)"),
    ];
    let mut current_flag = node.flag;
    ui.horizontal(|ui| {
        ui.label("Flag:");
        egui::ComboBox::from_id_salt(("node_flag_editor", node_id))
            .selected_text(format!("{:?}", current_flag))
            .show_ui(ui, |ui| {
                for (flag, label) in &editable_flags {
                    if ui
                        .selectable_value(&mut current_flag, *flag, *label)
                        .changed()
                    {
                        events.push(AppIntent::NodeFlagChangeRequested {
                            node_id,
                            flag: *flag,
                        });
                    }
                }
            });
    });

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

    let neighbors = road_map.connected_neighbors(node_id);
    if !neighbors.is_empty() {
        ui.separator();
        ui.label(format!("Verbindungen ({})", neighbors.len()));
        for n in &neighbors {
            let arrow = if n.is_outgoing { "→" } else { "←" };
            ui.horizontal(|ui| {
                ui.label(format!("{} Node #{}", arrow, n.neighbor_id));
            });
        }
    }
}

/// Zeigt Zwei-Node-Info: Verbindungen, Richtungs-/Prioritätsauswahl, Verbinden/Trennen.
fn render_two_nodes_info(
    ui: &mut egui::Ui,
    road_map: &RoadMap,
    selected: &IndexSet<u64>,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    events: &mut Vec<AppIntent>,
) {
    let mut iter = selected.iter().copied();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    ui.label(format!("Nodes: {}, {}", a, b));
    ui.separator();

    let conns = road_map.find_connections_between(a, b);

    if conns.is_empty() {
        ui.label("Keine Verbindung");
        ui.separator();

        if ui.button("Verbinden").clicked() {
            events.push(AppIntent::AddConnectionRequested {
                from_id: a,
                to_id: b,
                direction: default_direction,
                priority: default_priority,
            });
        }
    } else {
        for conn in &conns {
            render_connection_editor(ui, conn, events);
        }
    }
}

/// Zeigt Editor-Controls für eine einzelne Verbindung (Richtung, Priorität, Trennen).
fn render_connection_editor(ui: &mut egui::Ui, conn: &Connection, events: &mut Vec<AppIntent>) {
    ui.group(|ui| {
        ui.label(format!("{}→{}", conn.start_id, conn.end_id));

        let current_dir = conn.direction;
        let start_id = conn.start_id;
        let end_id = conn.end_id;

        let mut selected_dir = current_dir;
        render_direction_icon_selector(ui, &mut selected_dir, &format!("{}_{}", start_id, end_id));

        if selected_dir != current_dir {
            events.push(AppIntent::SetConnectionDirectionRequested {
                start_id,
                end_id,
                direction: selected_dir,
            });
        }

        let current_prio = conn.priority;
        let mut selected_prio = current_prio;
        render_priority_icon_selector(ui, &mut selected_prio, &format!("{}_{}", start_id, end_id));

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

/// Zeigt Gruppen-Bearbeiten-Buttons wenn passende Gruppen im Registry existieren.
fn render_segment_edit_buttons(
    ui: &mut egui::Ui,
    selected: &IndexSet<u64>,
    group_registry: Option<&GroupRegistry>,
    tool_edit_store: Option<&ToolEditStore>,
    events: &mut Vec<AppIntent>,
) {
    let Some(registry) = group_registry else {
        return;
    };
    let matching = registry.find_by_node_ids(selected);
    if matching.is_empty() {
        return;
    }

    ui.separator();
    ui.label("Gruppe bearbeiten:");
    for record in matching {
        let label = tool_edit_store
            .and_then(|store| store.tool_id_for(record.id))
            .map(|tool_id| format!("✏ {}", route_tool_descriptor(tool_id).name))
            .unwrap_or_else(|| "✏ Manuelle Gruppe".to_string());
        if ui.button(label).clicked() {
            events.push(AppIntent::GroupEditStartRequested {
                record_id: record.id,
            });
        }
    }
}
