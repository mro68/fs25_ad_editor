use super::SelectionState;
use crate::core::RoadMap;
use std::sync::Arc;

/// Snapshot reduziert auf die für Undo/Redo relevanten Teile.
///
/// Nutzt Arc-Clone (Copy-on-Write): Das Erstellen eines Snapshots ist O(1) —
/// der teure RoadMap-Klon findet erst beim nächsten `Arc::make_mut()` in einem
/// Use-Case statt (COW-Semantik). Bei 100k+ Nodes macht das einen erheblichen
/// Unterschied gegenüber einem vollständigen Deep-Clone pro mutierender Operation.
#[derive(Clone)]
pub struct Snapshot {
    /// Optionale RoadMap (Arc-Klon für O(1)-Snapshot)
    pub road_map: Option<Arc<RoadMap>>,
    /// Selektionszustand zum Zeitpunkt des Snapshots
    pub selection: SelectionState,
}

impl Snapshot {
    /// Erstellt einen O(1)-Snapshot durch Arc-Clone statt Deep-Clone.
    pub fn from_state(state: &crate::app::AppState) -> Self {
        Self {
            road_map: state.road_map.clone(), // O(1): nur Arc-Ref-Count erhöhen
            selection: state.selection.clone(),
        }
    }

    /// Stellt den Snapshot wieder her (O(1) Arc-Zuweisung).
    pub fn apply_to(self, state: &mut crate::app::AppState) {
        state.road_map = self.road_map;
        state.selection = self.selection;
    }
}

/// Einfacher Undo/Redo-Manager mit Snapshotting.
#[derive(Default)]
pub struct EditHistory {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    max_depth: usize,
}

impl EditHistory {
    /// Erstellt einen neuen History-Manager mit maximaler Tiefe.
    pub fn new_with_capacity(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_depth),
            redo_stack: Vec::with_capacity(max_depth),
            max_depth,
        }
    }

    /// Record a pre-built snapshot. Accepting a Snapshot avoids simultaneous
    /// mutable/immutable borrows on the full `AppState`.
    pub fn record_snapshot(&mut self, snap: Snapshot) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(snap);
        self.redo_stack.clear();
    }

    /// Prüft ob Undo möglich ist.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Prüft ob Redo möglich ist.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Pop an undo entry and return the Snapshot to apply (caller applies it).
    /// Pop undo stack and push `current` onto redo stack; returns the snapshot to apply.
    pub fn pop_undo_with_current(&mut self, current: Snapshot) -> Option<Snapshot> {
        if let Some(prev) = self.undo_stack.pop() {
            if self.redo_stack.len() >= self.max_depth {
                self.redo_stack.remove(0);
            }
            self.redo_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    /// Pop redo stack and push `current` onto undo stack; returns the snapshot to apply.
    pub fn pop_redo_with_current(&mut self, current: Snapshot) -> Option<Snapshot> {
        if let Some(next) = self.redo_stack.pop() {
            if self.undo_stack.len() >= self.max_depth {
                self.undo_stack.remove(0);
            }
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;
    use crate::{MapNode, NodeFlag, RoadMap};
    use std::sync::Arc;

    fn make_snapshot_with_node_count(count: usize) -> Snapshot {
        let mut map = RoadMap::new(3);
        for i in 1..=count {
            let f = i as f32;
            map.add_node(MapNode::new(
                i as u64,
                glam::Vec2::new(f * 10.0, f * 7.0),
                NodeFlag::Regular,
            ));
        }
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        Snapshot::from_state(&state)
    }

    #[test]
    fn empty_history_cannot_undo_or_redo() {
        let history = EditHistory::new_with_capacity(10);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn record_enables_undo() {
        let mut history = EditHistory::new_with_capacity(10);
        history.record_snapshot(make_snapshot_with_node_count(1));
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn undo_restores_previous_snapshot() {
        let mut history = EditHistory::new_with_capacity(10);

        let snap_before = make_snapshot_with_node_count(2);
        history.record_snapshot(snap_before);

        let current = make_snapshot_with_node_count(5);
        let restored = history
            .pop_undo_with_current(current)
            .expect("undo vorhanden");

        assert_eq!(restored.road_map.as_deref().unwrap().node_count(), 2);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn redo_restores_undone_snapshot() {
        let mut history = EditHistory::new_with_capacity(10);

        let snap_before = make_snapshot_with_node_count(2);
        history.record_snapshot(snap_before);

        let current_at_undo = make_snapshot_with_node_count(5);
        let _restored = history.pop_undo_with_current(current_at_undo);

        let current_at_redo = make_snapshot_with_node_count(2);
        let redone = history
            .pop_redo_with_current(current_at_redo)
            .expect("redo vorhanden");

        assert_eq!(redone.road_map.as_deref().unwrap().node_count(), 5);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn new_record_clears_redo_stack() {
        let mut history = EditHistory::new_with_capacity(10);
        history.record_snapshot(make_snapshot_with_node_count(1));

        let current = make_snapshot_with_node_count(3);
        let _restored = history.pop_undo_with_current(current);
        assert!(history.can_redo());

        history.record_snapshot(make_snapshot_with_node_count(7));
        assert!(!history.can_redo());
    }

    #[test]
    fn respects_max_depth() {
        let mut history = EditHistory::new_with_capacity(3);

        for i in 1..=5 {
            history.record_snapshot(make_snapshot_with_node_count(i));
        }

        // Nur 3 Undo-Schritte sollten möglich sein
        let mut undo_count = 0;
        while history.can_undo() {
            let current = make_snapshot_with_node_count(99);
            history.pop_undo_with_current(current);
            undo_count += 1;
        }
        assert_eq!(undo_count, 3);
    }

    #[test]
    fn pop_undo_on_empty_returns_none() {
        let mut history = EditHistory::new_with_capacity(10);
        let current = make_snapshot_with_node_count(1);
        assert!(history.pop_undo_with_current(current).is_none());
    }

    #[test]
    fn pop_redo_on_empty_returns_none() {
        let mut history = EditHistory::new_with_capacity(10);
        let current = make_snapshot_with_node_count(1);
        assert!(history.pop_redo_with_current(current).is_none());
    }

    #[test]
    fn snapshot_apply_to_restores_state() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            42,
            glam::Vec2::new(1.0, 2.0),
            NodeFlag::SubPrio,
        ));

        let mut original_state = AppState::new();
        original_state.road_map = Some(Arc::new(map));
        original_state.selection.selected_node_ids.insert(42);

        let snap = Snapshot::from_state(&original_state);

        let mut target_state = AppState::new();
        snap.apply_to(&mut target_state);

        assert_eq!(target_state.road_map.as_ref().unwrap().node_count(), 1);
        assert!(target_state.selection.selected_node_ids.contains(&42));
    }
}
