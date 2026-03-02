//! Query-Helfer auf `RoadMap` fuer Connection- und Spatial-Abfragen.

use super::Connection;
use super::RoadMap;
use crate::core::SpatialMatch;
use glam::Vec2;
use std::collections::HashSet;

impl RoadMap {
    /// Gibt alle Connections zurück, deren Start- und End-Ids in der gegebenen Menge liegen.
    ///
    /// Verwendet zum Filtern von Connections zwischen selektierten Nodes.
    /// O(n) über alle Connections, aber nur bei Use-Cases aufgerufen (nicht per-Frame).
    pub fn connections_between_ids<'a>(
        &'a self,
        ids: &'a HashSet<u64>,
    ) -> Box<dyn Iterator<Item = &'a Connection> + 'a> {
        Box::new(
            self.connections
                .values()
                .filter(move |c| ids.contains(&c.start_id) && ids.contains(&c.end_id)),
        )
    }

    /// Findet den nächstgelegenen Node zur Weltposition.
    pub fn nearest_node(&self, query: Vec2) -> Option<SpatialMatch> {
        debug_assert!(
            !self.spatial_dirty,
            "Spatial-Index ist veraltet — ensure_spatial_index() fehlt"
        );
        self.spatial_index.nearest(query)
    }

    /// Findet alle Nodes innerhalb eines Radius.
    pub fn nodes_within_radius(&self, query: Vec2, radius: f32) -> Vec<SpatialMatch> {
        debug_assert!(
            !self.spatial_dirty,
            "Spatial-Index ist veraltet — ensure_spatial_index() fehlt"
        );
        self.spatial_index.within_radius(query, radius)
    }

    /// Findet alle Nodes innerhalb eines Rechtecks.
    pub fn nodes_within_rect(&self, min: Vec2, max: Vec2) -> Vec<u64> {
        debug_assert!(
            !self.spatial_dirty,
            "Spatial-Index ist veraltet — ensure_spatial_index() fehlt"
        );
        self.spatial_index.within_rect(min, max)
    }
}
