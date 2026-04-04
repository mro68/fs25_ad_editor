use glam::Vec2;
use kiddo::{ImmutableKdTree, SquaredEuclidean};

/// Lokaler Suchindex fuer Sample-Punkte der Centerline-Berechnung.
#[derive(Debug, Clone)]
pub(super) struct SampleSearchIndex {
    tree: ImmutableKdTree<f64, 2>,
    points: Vec<Vec2>,
}

impl SampleSearchIndex {
    /// Baut einen immutable Suchindex aus den uebergebenen Sample-Punkten.
    pub(super) fn from_points(points: Vec<Vec2>) -> Self {
        if points.is_empty() {
            return Self::empty();
        }

        let entries: Vec<[f64; 2]> = points
            .iter()
            .map(|point| [point.x as f64, point.y as f64])
            .collect();
        let tree: ImmutableKdTree<f64, 2> = entries.as_slice().into();

        Self { tree, points }
    }

    /// Gibt `true` zurueck, wenn keine Punkte indexiert sind.
    pub(super) fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Findet den naechsten indexierten Punkt zur Query-Position.
    pub(super) fn nearest(&self, query: Vec2) -> Option<(Vec2, f32)> {
        if self.is_empty() {
            return None;
        }

        let result = self
            .tree
            .nearest_one::<SquaredEuclidean>(&[query.x as f64, query.y as f64]);
        let point = *self.points.get(result.item as usize)?;

        Some((point, (result.distance as f32).sqrt()))
    }

    fn empty() -> Self {
        let tree: ImmutableKdTree<f64, 2> = ImmutableKdTree::new_from_slice(&[]);
        Self {
            tree,
            points: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SampleSearchIndex;
    use glam::Vec2;

    fn brute_force_nearest(query: Vec2, set: &[Vec2]) -> (Vec2, f32) {
        let mut best = set[0];
        let mut best_distance_sq = query.distance_squared(best);

        for &point in &set[1..] {
            let distance_sq = query.distance_squared(point);
            if distance_sq < best_distance_sq {
                best = point;
                best_distance_sq = distance_sq;
            }
        }

        (best, best_distance_sq.sqrt())
    }

    #[test]
    fn nearest_matches_bruteforce_for_irregular_points() {
        let points = vec![
            Vec2::new(-10.0, 4.0),
            Vec2::new(2.5, -3.0),
            Vec2::new(17.0, 9.5),
            Vec2::new(-4.0, -11.0),
            Vec2::new(6.0, 13.0),
        ];
        let queries = [
            Vec2::new(-8.0, 3.0),
            Vec2::new(1.0, -2.0),
            Vec2::new(15.0, 11.0),
            Vec2::new(-2.0, -8.5),
        ];

        let index = SampleSearchIndex::from_points(points.clone());

        for query in queries {
            let indexed = index.nearest(query).expect("Index enthaelt Testpunkte");
            let brute_force = brute_force_nearest(query, &points);

            assert_eq!(indexed.0, brute_force.0);
            assert!((indexed.1 - brute_force.1).abs() < 1e-6);
        }
    }

    #[test]
    fn empty_index_returns_none() {
        let index = SampleSearchIndex::from_points(Vec::new());

        assert!(index.nearest(Vec2::new(0.0, 0.0)).is_none());
    }
}
