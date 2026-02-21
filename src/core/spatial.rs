//! Spatial-Index (KD-Tree) für schnelle Node-Abfragen.

use std::collections::HashMap;

use glam::Vec2;
use kiddo::{KdTree, SquaredEuclidean};

use crate::core::MapNode;

/// Ergebnis einer Distanzabfrage gegen den Spatial-Index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialMatch {
    /// ID des gefundenen Nodes
    pub node_id: u64,
    /// Euklidische Distanz zum Suchpunkt
    pub distance: f32,
}

/// Read-only Spatial-Index über allen Nodes einer RoadMap.
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    tree: KdTree<f64, 2>,
    node_ids: Vec<u64>,
    positions: HashMap<u64, Vec2>,
}

impl SpatialIndex {
    /// Erstellt einen leeren Spatial-Index.
    pub fn empty() -> Self {
        Self {
            tree: (&Vec::<[f64; 2]>::new()).into(),
            node_ids: Vec::new(),
            positions: HashMap::new(),
        }
    }

    /// Baut einen neuen Index aus den übergebenen Nodes.
    pub fn from_nodes(nodes: &HashMap<u64, MapNode>) -> Self {
        let mut node_ids: Vec<u64> = nodes.keys().copied().collect();
        node_ids.sort_unstable();

        let entries: Vec<[f64; 2]> = node_ids
            .iter()
            .filter_map(|id| {
                nodes
                    .get(id)
                    .map(|node| [node.position.x as f64, node.position.y as f64])
            })
            .collect();

        let tree: KdTree<f64, 2> = (&entries).into();

        let positions = nodes
            .iter()
            .map(|(id, node)| (*id, node.position))
            .collect();

        Self {
            tree,
            node_ids,
            positions,
        }
    }

    /// Gibt die Anzahl indexierter Nodes zurück.
    pub fn len(&self) -> usize {
        self.node_ids.len()
    }

    /// Gibt `true` zurück, wenn keine Nodes im Index liegen.
    pub fn is_empty(&self) -> bool {
        self.node_ids.is_empty()
    }

    /// Findet den nächsten Node zur gegebenen Weltposition.
    pub fn nearest(&self, query: Vec2) -> Option<SpatialMatch> {
        if self.is_empty() {
            return None;
        }

        let result = self
            .tree
            .nearest_one::<SquaredEuclidean>(&[query.x as f64, query.y as f64]);
        let node_id = *self.node_ids.get(result.item as usize)?;

        Some(SpatialMatch {
            node_id,
            distance: (result.distance as f32).sqrt(),
        })
    }

    /// Findet alle Nodes innerhalb eines Radius um die Query-Position.
    pub fn within_radius(&self, query: Vec2, radius: f32) -> Vec<SpatialMatch> {
        if self.is_empty() || radius.is_sign_negative() {
            return Vec::new();
        }

        let mut results = self
            .tree
            .within::<SquaredEuclidean>(&[query.x as f64, query.y as f64], (radius * radius) as f64)
            .into_iter()
            .filter_map(|entry| {
                let node_id = *self.node_ids.get(entry.item as usize)?;
                Some(SpatialMatch {
                    node_id,
                    distance: (entry.distance as f32).sqrt(),
                })
            })
            .collect::<Vec<_>>();

        results.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        results
    }

    /// Findet alle Nodes innerhalb eines axis-aligned Rechtecks.
    ///
    /// Nutzt den KD-Tree mit einer umschließenden Kreisabfrage + Nachfilterung,
    /// statt O(n) über alle Positionen zu iterieren.
    pub fn within_rect(&self, min: Vec2, max: Vec2) -> Vec<u64> {
        if self.is_empty() {
            return Vec::new();
        }

        let center_x = (min.x + max.x) as f64 * 0.5;
        let center_y = (min.y + max.y) as f64 * 0.5;
        let half_w = (max.x - min.x) as f64 * 0.5;
        let half_h = (max.y - min.y) as f64 * 0.5;
        // Radius des umschließenden Kreises (Diagonale / 2)
        let radius_sq = half_w * half_w + half_h * half_h;

        self.tree
            .within::<SquaredEuclidean>(&[center_x, center_y], radius_sq)
            .into_iter()
            .filter_map(|entry| {
                let node_id = *self.node_ids.get(entry.item as usize)?;
                let pos = self.positions.get(&node_id)?;
                // Exakte Rechteck-Prüfung nach dem KD-Tree-Vorfilter
                if pos.x >= min.x && pos.x <= max.x && pos.y >= min.y && pos.y <= max.y {
                    Some(node_id)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::NodeFlag;

    fn sample_nodes() -> HashMap<u64, MapNode> {
        let mut nodes = HashMap::new();
        nodes.insert(1, MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        nodes.insert(2, MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        nodes.insert(3, MapNode::new(3, Vec2::new(4.0, 3.0), NodeFlag::Regular));
        nodes
    }

    #[test]
    fn nearest_returns_expected_node() {
        let index = SpatialIndex::from_nodes(&sample_nodes());
        let nearest = index
            .nearest(Vec2::new(3.9, 2.9))
            .expect("Treffer erwartet");

        assert_eq!(nearest.node_id, 3);
        assert!(nearest.distance < 0.2);
    }

    #[test]
    fn radius_query_returns_sorted_matches() {
        let index = SpatialIndex::from_nodes(&sample_nodes());
        let matches = index.within_radius(Vec2::new(0.0, 0.0), 6.0);

        let ids: Vec<u64> = matches.into_iter().map(|m| m.node_id).collect();
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn rect_query_returns_nodes_inside_bounds() {
        let index = SpatialIndex::from_nodes(&sample_nodes());
        let mut ids = index.within_rect(Vec2::new(-1.0, -1.0), Vec2::new(5.0, 3.5));
        ids.sort_unstable();

        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn empty_index_has_no_entries() {
        let index = SpatialIndex::empty();

        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert!(index.nearest(Vec2::new(0.0, 0.0)).is_none());
    }
}
