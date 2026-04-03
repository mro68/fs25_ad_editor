//! Use-Case: Verschieben selektierter Nodes.

use crate::AppState;
use std::collections::HashSet;
use std::sync::Arc;

/// Verschiebt alle selektierten Nodes um den gegebenen Welt-Offset.
///
/// Wenn ein selektierter Node zu einem gesperrten (locked) Segment gehoert,
/// werden alle Nodes dieses Segments gemeinsam verschoben. Anschliessend
/// werden die `original_positions` der betroffenen locked Segments
/// aktualisiert, damit das Segment-Overlay gueltig bleibt.
pub fn move_selected_nodes(state: &mut AppState, delta_world: glam::Vec2) {
    if delta_world == glam::Vec2::ZERO {
        return;
    }

    let Some(road_map) = state.road_map.as_mut() else {
        return;
    };

    if state.selection.selected_node_ids.is_empty() {
        return;
    }

    // Selektion um Nodes von locked Segments erweitern
    let selected: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    let extra = state.group_registry.expand_locked_selection(&selected);

    let move_ids: HashSet<u64> = selected.iter().copied().chain(extra).collect();
    let move_ids_vec: Vec<u64> = move_ids.iter().copied().collect();

    let road_map_mut = Arc::make_mut(road_map);
    let moved_any = road_map_mut.translate_nodes(&move_ids_vec, delta_world);

    if moved_any {
        road_map_mut.rebuild_spatial_index();

        // IDs der betroffenen locked Segments sammeln (Borrow endet vor update-Loop)
        let locked_segment_ids: Vec<u64> = state
            .group_registry
            .records()
            .filter(|r| r.locked && r.node_ids.iter().any(|id| move_ids.contains(id)))
            .map(|r| r.id)
            .collect();

        // original_positions aktualisieren, damit is_group_valid() true bleibt
        for seg_id in locked_segment_ids {
            state
                .group_registry
                .update_original_positions(seg_id, road_map_mut);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::group_registry::GroupRecord;
    use crate::core::{MapNode, NodeFlag, RoadMap};
    use glam::Vec2;
    use std::sync::Arc;

    fn make_seg_record(
        id: u64,
        node_ids: Vec<u64>,
        positions: Vec<Vec2>,
        locked: bool,
    ) -> GroupRecord {
        GroupRecord {
            id,
            node_ids,
            original_positions: positions,
            marker_node_ids: Vec::new(),
            locked,
            entry_node_id: None,
            exit_node_id: None,
        }
    }

    #[test]
    fn move_selected_nodes_moves_all_selected() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            1,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            2,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::Regular,
        ));

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state.selection.ids_mut().insert(1);
        state.selection.ids_mut().insert(2);

        move_selected_nodes(&mut state, glam::Vec2::new(2.0, 3.0));

        let road_map = state.road_map.as_ref().expect("map vorhanden");
        let node1 = road_map.node(1).expect("node 1 vorhanden");
        let node2 = road_map.node(2).expect("node 2 vorhanden");
        assert_eq!(node1.position, glam::Vec2::new(2.0, 3.0));
        assert_eq!(node2.position, glam::Vec2::new(12.0, 3.0));
    }

    #[test]
    fn move_selected_nodes_updates_spatial_index() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            1,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.ensure_spatial_index();

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state.selection.ids_mut().insert(1);

        // Node von (0,0) nach (100,100) verschieben
        move_selected_nodes(&mut state, glam::Vec2::new(100.0, 100.0));

        let road_map = state.road_map.as_ref().unwrap();
        // Spatial-Index muss die neue Position finden
        let hit = road_map.nearest_node(glam::Vec2::new(100.0, 100.0));
        assert!(
            hit.is_some(),
            "Spatial-Index muss Node an neuer Position finden"
        );
        let hit = hit.unwrap();
        assert_eq!(hit.node_id, 1);
        assert!(
            hit.distance < 1.0,
            "Node muss nahe (100,100) sein, war {}",
            hit.distance
        );

        // An der alten Position darf der Node nicht mehr nahe sein
        let old_hit = road_map.nearest_node(glam::Vec2::new(0.0, 0.0));
        assert!(old_hit.is_some());
        assert!(
            old_hit.unwrap().distance > 50.0,
            "Spatial-Index darf Node nicht mehr an alter Position (0,0) finden"
        );
    }

    #[test]
    fn locked_segment_bewegt_alle_nodes_mit() {
        // Node 1 und 2 sind in einem locked Segment. Nur Node 1 ist selektiert.
        // Erwartet: Beide Nodes werden verschoben.
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state.selection.ids_mut().insert(1); // nur Node 1 selektiert

        let record = make_seg_record(
            0,
            vec![1, 2],
            vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)],
            true, // locked
        );
        state.group_registry.register(record);

        move_selected_nodes(&mut state, Vec2::new(5.0, 0.0));

        let rm = state.road_map.as_ref().unwrap();
        assert_eq!(
            rm.node(1).expect("node 1 vorhanden").position,
            Vec2::new(5.0, 0.0),
            "Node 1 muss verschoben sein"
        );
        assert_eq!(
            rm.node(2).expect("node 2 vorhanden").position,
            Vec2::new(15.0, 0.0),
            "Node 2 muss mitbewegt werden"
        );
    }

    #[test]
    fn unlocked_segment_bewegt_nur_selektierten_node() {
        // Node 1 und 2 sind in einem UNlocked Segment. Nur Node 1 selektiert.
        // Erwartet: Nur Node 1 bewegt sich.
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state.selection.ids_mut().insert(1);

        let record = make_seg_record(
            0,
            vec![1, 2],
            vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)],
            false, // unlocked
        );
        state.group_registry.register(record);

        move_selected_nodes(&mut state, Vec2::new(5.0, 0.0));

        let rm = state.road_map.as_ref().unwrap();
        assert_eq!(
            rm.node(1).expect("node 1 vorhanden").position,
            Vec2::new(5.0, 0.0)
        );
        assert_eq!(
            rm.node(2).expect("node 2 vorhanden").position,
            Vec2::new(10.0, 0.0),
            "Node 2 darf nicht bewegt werden"
        );
    }

    #[test]
    fn locked_move_aktualisiert_original_positions() {
        // Nach einem Locked-Move müssen original_positions geupdated sein,
        // damit is_group_valid() weiterhin true zurückgibt.
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state.selection.ids_mut().insert(1);

        let record = make_seg_record(
            0,
            vec![1, 2],
            vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)],
            true,
        );
        state.group_registry.register(record);

        move_selected_nodes(&mut state, Vec2::new(7.0, 3.0));

        let rm = state.road_map.as_ref().unwrap();
        let seg = state.group_registry.get(0).unwrap();
        // original_positions müssen die neuen Positionen widerspiegeln
        assert_eq!(seg.original_positions[0], Vec2::new(7.0, 3.0));
        assert_eq!(seg.original_positions[1], Vec2::new(17.0, 3.0));
        // Segment muss noch gültig sein
        assert!(
            state.group_registry.is_group_valid(seg, rm),
            "Segment muss nach Locked-Move noch gueltig sein"
        );
    }
}
