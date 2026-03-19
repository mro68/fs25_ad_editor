//! Handler fuer Segment-Operationen (Lock-Toggle).

use crate::app::segment_registry::{SegmentBase, SegmentKind, SegmentRecord};
use crate::app::tools::ToolAnchor;
use crate::app::AppState;

/// Schaltet den Lock-Zustand eines Segments um.
///
/// Ist das Segment gesperrt, werden beim Verschieben eines zugehoerigen Nodes
/// alle Segment-Nodes mitbewegt. Unbekannte IDs werden ignoriert.
pub fn toggle_lock(state: &mut AppState, segment_id: u64) {
    state.segment_registry.toggle_lock(segment_id);
}

/// Loest ein Segment auf, indem nur der Segment-Record entfernt wird.
///
/// Die zugehoerigen Nodes und Verbindungen in der RoadMap bleiben unveraendert.
/// Unbekannte IDs werden ignoriert.
pub fn dissolve(state: &mut AppState, segment_id: u64) {
    state.segment_registry.remove(segment_id);
}

/// Gruppiert die selektierten zusammenhaengenden Nodes als neues Segment.
///
/// Die Nodes werden in Ketten-Reihenfolge gespeichert. Ist die Selektion
/// keine gueltige lineare Kette, wird der Aufruf ignoriert.
pub fn group_selection(state: &mut AppState) {
    let road_map = match state.road_map.as_deref() {
        Some(rm) => rm,
        None => return,
    };
    let selected_ids = &state.selection.selected_node_ids;
    let ordered_ids = match road_map.ordered_chain_nodes(selected_ids) {
        Some(ids) => ids,
        None => return,
    };

    let start_id = *ordered_ids.first().unwrap();
    let end_id = *ordered_ids.last().unwrap();
    let start_pos = match road_map.nodes.get(&start_id) {
        Some(n) => n.position,
        None => return,
    };
    let end_pos = match road_map.nodes.get(&end_id) {
        Some(n) => n.position,
        None => return,
    };

    let original_positions: Vec<_> = ordered_ids
        .iter()
        .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
        .collect();

    let record_id = state.segment_registry.next_id();
    let record = SegmentRecord {
        id: record_id,
        node_ids: ordered_ids,
        start_anchor: ToolAnchor::ExistingNode(start_id, start_pos),
        end_anchor: ToolAnchor::ExistingNode(end_id, end_pos),
        kind: SegmentKind::Straight {
            base: SegmentBase {
                direction: state.editor.default_direction,
                priority: state.editor.default_priority,
                max_segment_length: state.options.mouse_wheel_distance_step_m.max(1.0),
            },
        },
        original_positions,
        marker_node_ids: vec![],
        locked: false,
    };
    state.segment_registry.register(record);
}
