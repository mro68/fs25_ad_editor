//! Use-Case: Bulk-Änderungen an Verbindungen zwischen selektierten Nodes.

use crate::app::AppState;
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use std::collections::HashSet;
use std::sync::Arc;

/// Sammelt alle Connection-Keys, deren Start- und End-Node in der Selektion liegen.
fn collect_selected_connection_keys(
    road_map: &RoadMap,
    selected: &HashSet<u64>,
) -> Vec<(u64, u64)> {
    road_map
        .connections_iter()
        .filter(|c| selected.contains(&c.start_id) && selected.contains(&c.end_id))
        .map(|c| (c.start_id, c.end_id))
        .collect()
}

/// Gemeinsame Logik für Bulk-Operationen auf Verbindungen zwischen selektierten Nodes.
///
/// Validiert die Selektion, erstellt einen Undo-Snapshot, führt die Operation aus
/// und aktualisiert die Node-Flags. Gibt die Anzahl betroffener Verbindungen zurück.
fn mutate_connections_between_selected<F>(
    state: &mut AppState,
    operation_name: &str,
    mut mutator: F,
) -> u32
where
    F: FnMut(&mut RoadMap, &HashSet<u64>) -> u32,
{
    // Validierung in eigenem Scope, damit Borrows vor dem Snapshot enden
    {
        let selected = &state.selection.selected_node_ids;
        if selected.len() < 2 {
            return 0;
        }

        let Some(road_map_arc) = state.road_map.as_ref() else {
            return 0;
        };

        let has_affected = road_map_arc
            .connections_iter()
            .any(|c| selected.contains(&c.start_id) && selected.contains(&c.end_id));

        if !has_affected {
            log::debug!(
                "Keine Verbindungen zwischen selektierten Nodes für Operation '{}'",
                operation_name
            );
            return 0;
        }
    }

    state.record_undo_snapshot();

    let selected = state.selection.selected_node_ids.clone();
    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!(
            "Bulk-Operation '{}' abgebrochen: keine RoadMap geladen",
            operation_name
        );
        return 0;
    };
    let road_map = Arc::make_mut(road_map_arc);

    let count = mutator(road_map, &selected);

    // Flags der betroffenen Nodes neu berechnen
    let affected: Vec<u64> = selected.iter().copied().collect();
    road_map.recalculate_node_flags(&affected);

    count
}

/// Ändert die Richtung aller Verbindungen zwischen den selektierten Nodes.
///
/// Findet alle Connections, bei denen sowohl start_id als auch end_id
/// in der Selektion enthalten sind, und setzt sie auf die angegebene Richtung.
/// Bei Wechsel weg von Dual werden Duplikat-Gegen-Connections entfernt.
pub fn set_all_connections_direction_between_selected(
    state: &mut AppState,
    direction: ConnectionDirection,
) {
    let count =
        mutate_connections_between_selected(state, "set_direction", |road_map, selected| {
            // Bei Wechsel auf nicht-Dual: erst Duplikat-Paare einsammeln und bereinigen
            if direction != ConnectionDirection::Dual {
                let mut seen_pairs = HashSet::new();
                let mut to_remove = Vec::new();
                for conn in road_map.connections_between_ids(selected) {
                    let pair = (
                        conn.start_id.min(conn.end_id),
                        conn.start_id.max(conn.end_id),
                    );
                    if seen_pairs.contains(&pair) {
                        to_remove.push((conn.start_id, conn.end_id));
                    } else {
                        seen_pairs.insert(pair);
                    }
                }
                for (s, e) in to_remove {
                    road_map.remove_connection(s, e);
                }
            }

            let keys = collect_selected_connection_keys(road_map, selected);
            let count = keys.len() as u32;
            for (s, e) in keys {
                road_map.set_connection_direction(s, e, direction);
            }
            count
        });

    if count > 0 {
        log::info!(
            "{} Verbindung(en) zwischen selektierten Nodes auf {:?} geändert",
            count,
            direction
        );
    }
}

/// Entfernt alle Verbindungen zwischen den selektierten Nodes.
///
/// Entfernt alle Connections, bei denen sowohl start_id als auch end_id
/// in der Selektion enthalten sind.
pub fn remove_all_connections_between_selected(state: &mut AppState) {
    let count = mutate_connections_between_selected(state, "remove", |road_map, selected| {
        let keys = collect_selected_connection_keys(road_map, selected);
        let count = keys.len() as u32;
        for (s, e) in keys {
            road_map.remove_connection(s, e);
        }
        count
    });

    if count > 0 {
        log::info!(
            "{} Verbindung(en) zwischen selektierten Nodes entfernt",
            count
        );
    }
}

/// Invertiert die Richtung aller Verbindungen zwischen den selektierten Nodes.
///
/// Tauscht start_id und end_id jeder betroffenen Connection und
/// aktualisiert die Geometrie (Mittelpunkt/Winkel).
pub fn invert_all_connections_between_selected(state: &mut AppState) {
    let count = mutate_connections_between_selected(state, "invert", |road_map, selected| {
        let keys = collect_selected_connection_keys(road_map, selected);
        let count = keys.len() as u32;
        for (s, e) in keys {
            road_map.invert_connection(s, e);
        }
        count
    });

    if count > 0 {
        log::info!(
            "{} Verbindung(en) zwischen selektierten Nodes invertiert",
            count
        );
    }
}

/// Ändert die Priorität aller Verbindungen zwischen den selektierten Nodes.
pub fn set_all_connections_priority_between_selected(
    state: &mut AppState,
    priority: ConnectionPriority,
) {
    let count = mutate_connections_between_selected(state, "set_priority", |road_map, selected| {
        let keys = collect_selected_connection_keys(road_map, selected);
        let count = keys.len() as u32;
        for (s, e) in keys {
            road_map.set_connection_priority(s, e, priority);
        }
        count
    });

    if count > 0 {
        log::info!(
            "{} Verbindung(en) zwischen selektierten Nodes auf {:?} geändert",
            count,
            priority
        );
    }
}

#[cfg(test)]
mod tests {
    use super::collect_selected_connection_keys;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;
    use std::collections::HashSet;

    #[test]
    fn collects_only_connections_with_both_ends_selected() {
        let mut road_map = RoadMap::new(3);
        road_map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(2, Vec2::new(1.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(3, Vec2::new(2.0, 0.0), NodeFlag::Regular));

        road_map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
        ));
        road_map.add_connection(Connection::new(
            2,
            3,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
        ));

        let selected = HashSet::from([1_u64, 2_u64]);
        let keys = collect_selected_connection_keys(&road_map, &selected);

        assert_eq!(keys, vec![(1, 2)]);
    }
}
