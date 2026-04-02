//! Use-Case: Rotation selektierter Nodes um ihr gemeinsames Zentrum.

use crate::AppState;
use glam::Vec2;
use std::collections::HashSet;
use std::sync::Arc;

/// Rotiert alle selektierten Nodes um den gegebenen Winkel (Radiant).
///
/// Der Drehpunkt ist das arithmetische Mittel aller betroffenen Node-Positionen.
/// Nodes aus gesperrten Segmenten werden analog zum Move-Use-Case mit-rotiert.
///
/// Der Spatial-Index wird **nicht** rebuilt — das muss der Aufrufer am Ende
/// des Rotation-Lifecycles separat anstoßen.
pub fn rotate_selected_nodes(state: &mut AppState, angle_rad: f32) {
    if angle_rad == 0.0 {
        return;
    }

    let Some(road_map) = state.road_map.as_mut() else {
        return;
    };

    if state.selection.selected_node_ids.is_empty() {
        return;
    }

    // Selektion um Nodes von locked Segments erweitern (analog move_nodes.rs)
    let selected: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    let extra = state.group_registry.expand_locked_selection(&selected);

    let rotate_ids: HashSet<u64> = selected.iter().copied().chain(extra).collect();
    let rotate_ids_vec: Vec<u64> = rotate_ids.iter().copied().collect();

    let road_map_ref = road_map.as_ref();

    // Geometrisches Zentrum aller betroffenen Nodes berechnen
    let mut sum = Vec2::ZERO;
    let mut count = 0usize;
    for &node_id in &rotate_ids {
        if let Some(node) = road_map_ref.node(node_id) {
            sum += node.position;
            count += 1;
        }
    }
    if count == 0 {
        return;
    }
    let center = sum / count as f32;

    let road_map_mut = Arc::make_mut(road_map);
    road_map_mut.rotate_nodes(&rotate_ids_vec, center, angle_rad);

    // Locked-Segment-original_positions aktualisieren
    let locked_segment_ids: Vec<u64> = state
        .group_registry
        .records()
        .filter(|r| r.locked && r.node_ids.iter().any(|id| rotate_ids.contains(id)))
        .map(|r| r.id)
        .collect();

    for seg_id in locked_segment_ids {
        state
            .group_registry
            .update_original_positions(seg_id, road_map_mut);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MapNode, NodeFlag, RoadMap};
    use std::f32::consts::PI;

    fn make_state_with_nodes(positions: &[(u64, f32, f32)]) -> AppState {
        let mut map = RoadMap::new(positions.len() as u32 + 1);
        for &(id, x, y) in positions {
            map.add_node(MapNode::new(id, Vec2::new(x, y), NodeFlag::Regular));
        }
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state
    }

    #[test]
    fn rotate_single_node_around_self() {
        // Ein einzelner Node rotiert um sich selbst → Position unveraendert
        let mut state = make_state_with_nodes(&[(1, 3.0, 4.0)]);
        state.selection.ids_mut().insert(1);

        rotate_selected_nodes(&mut state, PI / 2.0);

        let rm = state.road_map.as_ref().unwrap();
        let pos = rm.node(1).expect("node 1 vorhanden").position;
        assert!((pos.x - 3.0).abs() < 1e-5, "x={}", pos.x);
        assert!((pos.y - 4.0).abs() < 1e-5, "y={}", pos.y);
    }

    #[test]
    fn rotate_two_nodes_ninety_degrees() {
        // Zwei Nodes symmetric um Ursprung: (1,0) und (-1,0)
        // Bei 90°: (1,0) → (0,1) und (-1,0) → (0,-1)
        let mut state = make_state_with_nodes(&[(1, 1.0, 0.0), (2, -1.0, 0.0)]);
        state.selection.ids_mut().insert(1);
        state.selection.ids_mut().insert(2);

        rotate_selected_nodes(&mut state, PI / 2.0);

        let rm = state.road_map.as_ref().unwrap();
        let p1 = rm.node(1).expect("node 1 vorhanden").position;
        let p2 = rm.node(2).expect("node 2 vorhanden").position;

        // Zentrum = (0,0); 90° Rotation im Gegenuhrzeigersinn
        // (1,0) → (0,1)
        assert!((p1.x - 0.0).abs() < 1e-5, "p1.x={}", p1.x);
        assert!((p1.y - 1.0).abs() < 1e-5, "p1.y={}", p1.y);
        // (-1,0) → (0,-1)
        assert!((p2.x - 0.0).abs() < 1e-5, "p2.x={}", p2.x);
        assert!((p2.y + 1.0).abs() < 1e-5, "p2.y={}", p2.y);
    }

    #[test]
    fn rotate_three_nodes_around_centroid() {
        // Drei Nodes bei (0,0), (2,0), (1,2)
        // Zentrum = (1.0, 2/3)
        let mut state = make_state_with_nodes(&[(1, 0.0, 0.0), (2, 2.0, 0.0), (3, 1.0, 2.0)]);
        state.selection.ids_mut().insert(1);
        state.selection.ids_mut().insert(2);
        state.selection.ids_mut().insert(3);

        let rm_before = state.road_map.as_ref().unwrap();
        let center_before = {
            let p1 = rm_before.node(1).expect("node 1 vorhanden").position;
            let p2 = rm_before.node(2).expect("node 2 vorhanden").position;
            let p3 = rm_before.node(3).expect("node 3 vorhanden").position;
            (p1 + p2 + p3) / 3.0
        };

        rotate_selected_nodes(&mut state, PI);

        let rm = state.road_map.as_ref().unwrap();
        let center_after = {
            let p1 = rm.node(1).expect("node 1 vorhanden").position;
            let p2 = rm.node(2).expect("node 2 vorhanden").position;
            let p3 = rm.node(3).expect("node 3 vorhanden").position;
            (p1 + p2 + p3) / 3.0
        };

        // Schwerpunkt darf sich nicht verschieben
        assert!(
            (center_before.x - center_after.x).abs() < 1e-4,
            "centroid.x geaendert"
        );
        assert!(
            (center_before.y - center_after.y).abs() < 1e-4,
            "centroid.y geaendert"
        );
    }

    #[test]
    fn rotate_zero_angle_noop() {
        let mut state = make_state_with_nodes(&[(1, 5.0, 7.0)]);
        state.selection.ids_mut().insert(1);

        rotate_selected_nodes(&mut state, 0.0);

        let rm = state.road_map.as_ref().unwrap();
        let pos = rm.node(1).expect("node 1 vorhanden").position;
        assert_eq!(pos, Vec2::new(5.0, 7.0));
    }
}
