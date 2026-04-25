//! Stage F — Polyline-Simplify, Junction-Trim und Resampling zu `PreparedSegment`s.

use super::bump_revision;
use crate::app::tools::color_path::editable::{EditableCenterlineId, EditableCenterlines};
use crate::app::tools::color_path::skeleton::{
    SkeletonGraphNodeKind, SkeletonGraphSegment, SkeletonNetwork,
};
use crate::app::tools::color_path::state::{
    ColorPathTool, PreparedSegment, PreparedSegmentsCacheKey,
};
use crate::core::simplify_polyline;
use crate::shared::spline_geometry::resample_by_distance;

impl ColorPathTool {
    /// Berechnet Stage F mit der aktuellen Preview-Konfiguration neu.
    pub(in crate::app::tools::color_path) fn rebuild_prepared_segments(&mut self) {
        let _ = self.ensure_prepared_segments();
    }

    /// Stellt sicher, dass Stage F (und bei Bedarf Stage D/E) aktuell ist.
    pub(super) fn ensure_prepared_segments(&mut self) -> bool {
        if self.preview_data.is_none() {
            if !self.ensure_preview_core() {
                return false;
            }
        } else if let Some(preview_core_key) = self.preview_core_cache_key()
            && self.cache.preview_core_key != Some(preview_core_key)
            && !self.ensure_preview_core()
        {
            return false;
        }

        let Some(key) = self.prepared_segments_cache_key() else {
            self.cache.prepared_segments_key = None;
            return false;
        };

        if self.cache.prepared_segments_key == Some(key) {
            return self
                .preview_data
                .as_ref()
                .is_some_and(|preview| !preview.prepared_segments.is_empty());
        }

        let Some(preview_data) = self.preview_data.as_mut() else {
            self.cache.prepared_segments_key = None;
            return false;
        };

        preview_data.prepared_segments = prepare_segments(
            &preview_data.network,
            self.editable.as_ref(),
            self.config.simplify_tolerance,
            self.config.node_spacing,
            self.config.junction_radius,
        );

        log::info!(
            "Netz-Vorschau aktualisiert: {} Knoten, {} Segmente, {} Preview-Segmente",
            preview_data.network.nodes.len(),
            preview_data.network.segments.len(),
            preview_data.prepared_segments.len()
        );

        let has_segments = !preview_data.prepared_segments.is_empty();

        self.cache.prepared_segments_key = Some(key);
        bump_revision(&mut self.cache.prepared_segments_revision);
        has_segments
    }

    /// Liefert den Cache-Schluessel fuer Stage F.
    pub(super) fn prepared_segments_cache_key(&self) -> Option<PreparedSegmentsCacheKey> {
        self.preview_data.as_ref()?;
        Some(PreparedSegmentsCacheKey {
            preview_core_revision: self.cache.preview_core_revision,
            editable_revision: self.editable.as_ref().map_or(0, |e| e.revision),
            simplify_tolerance_bits: self.config.simplify_tolerance.to_bits(),
            node_spacing_bits: self.config.node_spacing.to_bits(),
            junction_radius_bits: self.config.junction_radius.to_bits(),
        })
    }
}

pub(super) fn prepare_segments(
    network: &SkeletonNetwork,
    editable: Option<&EditableCenterlines>,
    simplify_tolerance: f32,
    node_spacing: f32,
    junction_radius: f32,
) -> Vec<PreparedSegment> {
    let mut prepared_segments = Vec::with_capacity(network.segments.len());

    for (segment_index, segment) in network.segments.iter().enumerate() {
        if segment.start_node == segment.end_node {
            continue;
        }

        let simplified = simplify_polyline(&segment.polyline, simplify_tolerance);
        // CP-07: Polyline-Start/-End an die aktuelle EditableJunction-Position ziehen,
        // damit Junction-Drag korrekt zieht. Faellt auf die unveraenderte Polyline
        // zurueck, wenn der Pull Start und Ende kollabieren liesse.
        let (pulled, did_pull) = pull_endpoints_to_editable(&simplified, editable, segment_index);
        let trimmed = trim_segment_near_junctions(network, segment, &pulled, junction_radius)
            .or_else(|| {
                // Degenerierter Fall: Falls der Pull+Trim kollabiert, versuchen wir
                // den Trim erneut ohne Endpunkt-Pull, damit die Original-Endpunkte
                // als Fallback erhalten bleiben.
                if did_pull {
                    trim_segment_near_junctions(network, segment, &simplified, junction_radius)
                } else {
                    None
                }
            });
        let Some(trimmed_nodes) = trimmed else {
            continue;
        };
        if trimmed_nodes.len() < 2 {
            continue;
        }
        let resampled_nodes = resample_by_distance(&trimmed_nodes, node_spacing);
        if resampled_nodes.len() < 2 {
            continue;
        }

        prepared_segments.push(PreparedSegment {
            start_node: segment.start_node,
            end_node: segment.end_node,
            resampled_nodes,
        });
    }

    prepared_segments
}

/// Zieht Start- und Endpunkt der Polyline an die aktuelle Weltposition der
/// zugeordneten [`EditableJunction`]s (CP-07). Ohne `editable` oder ohne
/// passende Zuordnung bleibt die Polyline unveraendert.
///
/// Bei Degeneration (Pull laesst Start/Ende kollabieren, Polyline zu kurz)
/// wird die Original-Polyline zurueckgegeben, damit nachfolgende Stages einen
/// robusten Fallback auf die unveraenderten Endpunkte haben.
fn pull_endpoints_to_editable(
    polyline: &[glam::Vec2],
    editable: Option<&EditableCenterlines>,
    segment_index: usize,
) -> (Vec<glam::Vec2>, bool) {
    if polyline.len() < 2 {
        return (polyline.to_vec(), false);
    }
    let Some(editable) = editable else {
        return (polyline.to_vec(), false);
    };
    let centerline_id = EditableCenterlineId(segment_index as u32);
    let Some(centerline) = editable.centerlines.get(&centerline_id) else {
        return (polyline.to_vec(), false);
    };

    let mut pulled = polyline.to_vec();
    let last_idx = pulled.len() - 1;
    let mut changed = false;
    if let Some(start_id) = centerline.start_junction
        && let Some(junction) = editable.junctions.get(&start_id)
        && pulled[0] != junction.world_pos
    {
        pulled[0] = junction.world_pos;
        changed = true;
    }
    if let Some(end_id) = centerline.end_junction
        && let Some(junction) = editable.junctions.get(&end_id)
        && pulled[last_idx] != junction.world_pos
    {
        pulled[last_idx] = junction.world_pos;
        changed = true;
    }

    // Degenerierter Fall: Pull hat Start und Ende auf dieselbe Position gezogen
    // → Fallback auf Original-Polyline.
    if pulled[0].distance_squared(pulled[last_idx]) <= f32::EPSILON {
        return (polyline.to_vec(), false);
    }
    (pulled, changed)
}

pub(super) fn trim_segment_near_junctions(
    network: &SkeletonNetwork,
    segment: &SkeletonGraphSegment,
    nodes: &[glam::Vec2],
    junction_radius: f32,
) -> Option<Vec<glam::Vec2>> {
    if nodes.len() < 2 {
        return None;
    }

    let mut trimmed = nodes.to_vec();
    if junction_radius > 0.0 {
        if is_junction_node(network, segment.start_node) {
            trim_from_start(&mut trimmed, junction_radius);
        }
        if is_junction_node(network, segment.end_node) {
            trim_from_end(&mut trimmed, junction_radius);
        }
    }

    compact_consecutive_duplicates(&mut trimmed);
    let first = *trimmed.first()?;
    let last = *trimmed.last()?;
    if first.distance_squared(last) <= f32::EPSILON {
        return None;
    }

    Some(trimmed)
}

fn is_junction_node(network: &SkeletonNetwork, node_index: usize) -> bool {
    network
        .nodes
        .get(node_index)
        .is_some_and(|node| node.kind == SkeletonGraphNodeKind::Junction)
}

fn trim_from_start(nodes: &mut Vec<glam::Vec2>, radius: f32) {
    if nodes.len() < 3 {
        return;
    }

    let anchor = nodes[0];
    let first_outside = (1..nodes.len() - 1)
        .find(|&idx| nodes[idx].distance(anchor) > radius)
        .unwrap_or(nodes.len() - 1);

    if first_outside > 1 {
        nodes.drain(1..first_outside);
    }
}

fn trim_from_end(nodes: &mut Vec<glam::Vec2>, radius: f32) {
    if nodes.len() < 3 {
        return;
    }

    let anchor = *nodes.last().expect("Segmentende fehlt");
    let last_outside = (1..nodes.len() - 1)
        .rev()
        .find(|&idx| nodes[idx].distance(anchor) > radius)
        .unwrap_or(0);

    let keep_until = if last_outside == 0 {
        1
    } else {
        last_outside + 1
    };
    if keep_until < nodes.len() - 1 {
        nodes.drain(keep_until..nodes.len() - 1);
    }
}

fn compact_consecutive_duplicates(nodes: &mut Vec<glam::Vec2>) {
    nodes.dedup_by(|a, b| a.distance_squared(*b) <= f32::EPSILON);
}
