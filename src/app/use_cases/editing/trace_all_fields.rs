//! Use-Case: Alle Farmland-Polygone als Wegpunkt-Ring nachzeichnen (Batch-Operation).
//!
//! Erzeugt fuer jedes erkannte Feldpolygon einen geschlossenen Wegpunkt-Ring
//! mit den Standard-Parametern des FieldBoundaryTool. Alle Polygone werden
//! in einem einzigen Undo-Schritt zusammengefasst. Pro Feld wird ein
//! `GroupRecord` in der `group_registry` angelegt, damit die erzeugten
//! Strecken nachtraeglich bearbeitet werden koennen.

use crate::app::compute_ring;
use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::AppState;
use crate::app::ToolAnchor;
use crate::core::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag};
use glam::Vec2;
use std::sync::Arc;

/// Berechnet die Flaeche eines Polygons mittels Shoelace-Formel.
///
/// Gibt 0 zurueck wenn das Polygon weniger als 3 Vertices hat.
fn polygon_area(vertices: &[Vec2]) -> f32 {
    let n = vertices.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += vertices[i].x * vertices[j].y;
        area -= vertices[j].x * vertices[i].y;
    }
    area.abs() / 2.0
}

/// Zeichnet alle erkannten Farmland-Polygone als Wegpunkt-Ring nach.
///
/// Alle erstellten Nodes und Verbindungen werden in einem einzigen Undo-Schritt
/// zusammengefasst. Der Spatial-Index wird nur einmal am Ende rebuildet.
/// Pro Feld wird ein `GroupRecord` in der `group_registry` angelegt,
/// damit die erzeugten Feldstrecken nachtraeglich als Segment angewaehlt
/// und bearbeitet werden koennen.
///
/// Gibt fruehzeitig zurueck wenn keine Polygone geladen oder keine RoadMap vorhanden.
///
/// # Parameter
/// * `spacing` – Abstand zwischen Wegpunkten in Welteinheiten (Metern)
/// * `offset` – Versatz vom Feldrand (positiv = nach innen)
/// * `tolerance` – Douglas-Peucker-Toleranz fuer Begradigung (0 = aus)
/// * `corner_angle` – Winkel-Schwellwert fuer Ecken-Erkennung in Grad (None = deaktiviert)
pub fn trace_all_fields(
    state: &mut AppState,
    spacing: f32,
    offset: f32,
    tolerance: f32,
    corner_angle: Option<f32>,
) {
    // Polygone vor dem Snapshot klonen (Arc, O(1))
    let polygons = match &state.farmland_polygons {
        Some(p) if !p.is_empty() => Arc::clone(p),
        _ => {
            log::info!("Keine Farmland-Polygone vorhanden — Abbruch");
            return;
        }
    };

    if state.road_map.is_none() {
        log::warn!("Keine RoadMap geladen — Alle Felder nachzeichnen abgebrochen");
        return;
    }

    // Standard-Parameter (entsprechen FieldBoundaryTool::new())
    let direction = ConnectionDirection::Dual;
    let priority = ConnectionPriority::Regular;

    // Undo-Snapshot VOR der Batch-Mutation (Arc-Clone, O(1))
    state.record_undo_snapshot();

    // (field_id, node_ids) — wird nach dem road_map-Block fuer Registry-Eintraege genutzt
    let (all_new_ids, field_segments) = {
        let road_map = Arc::make_mut(state.road_map.as_mut().expect("road_map vorhanden"));

        let mut all_new_ids: Vec<u64> = Vec::new();
        let mut field_segments: Vec<(u32, Vec<u64>)> = Vec::new();
        let mut field_count = 0usize;
        // Mindestflaeche: mindestens 4 Node-Abstands-Quadrate wert, sonst Noise-Patch
        let min_area = spacing * spacing * 4.0;

        for polygon in polygons.iter() {
            // Fix: Sehr kleine Polygone (Noise-Patches, 1–3 Pixel) ueberspringen
            let area = polygon_area(&polygon.vertices);
            if area < min_area {
                log::debug!(
                    "Feld {}: Flaeche {:.1} < Schwellwert {:.1} — uebersprungen",
                    polygon.id,
                    area,
                    min_area
                );
                continue;
            }

            let positions =
                compute_ring(&polygon.vertices, offset, tolerance, spacing, corner_angle);
            if positions.len() < 2 {
                log::debug!(
                    "Feld {}: zu wenige Punkte nach Ring-Berechnung — uebersprungen",
                    polygon.id
                );
                continue;
            }

            let n = positions.len();
            let mut poly_ids: Vec<u64> = Vec::with_capacity(n);

            // Nodes erstellen
            for pos in &positions {
                let id = road_map.next_node_id();
                road_map.add_node(MapNode::new(id, *pos, NodeFlag::Regular));
                poly_ids.push(id);
                all_new_ids.push(id);
            }

            // Verbindungen als geschlossener Ring erstellen
            for i in 0..n {
                let from_id = poly_ids[i];
                let to_id = poly_ids[(i + 1) % n];
                let from_pos = road_map.nodes[&from_id].position;
                let to_pos = road_map.nodes[&to_id].position;
                let conn = Connection::new(from_id, to_id, direction, priority, from_pos, to_pos);
                road_map.add_connection(conn);
            }

            field_segments.push((polygon.id, poly_ids));
            field_count += 1;
            log::debug!("Feld {}: {} Nodes erstellt", polygon.id, n);
        }

        if !all_new_ids.is_empty() {
            // Flag-Berechnung und Spatial-Index-Rebuild exakt 1x am Ende der Batch-Operation
            road_map.recalculate_node_flags(&all_new_ids);
            road_map.ensure_spatial_index();
            log::info!(
                "Alle Felder nachgezeichnet: {} Felder, {} Nodes erstellt",
                field_count,
                all_new_ids.len()
            );
        } else {
            log::info!("Alle Felder nachzeichnen: keine verwertbaren Polygone gefunden");
        }

        (all_new_ids, field_segments)
    };

    if all_new_ids.is_empty() {
        return;
    }

    // Pro Feld einen GroupRecord anlegen, damit die Strecken bearbeitbar bleiben
    {
        let road_map_ref = state.road_map.as_deref().expect("road_map vorhanden");
        for (field_id, node_ids) in &field_segments {
            let record_id = state.group_registry.next_id();
            let original_positions: Vec<Vec2> = node_ids
                .iter()
                .filter_map(|id| road_map_ref.nodes.get(id).map(|n| n.position))
                .collect();
            let record = GroupRecord {
                id: record_id,
                node_ids: node_ids.clone(),
                start_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
                end_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
                original_positions,
                marker_node_ids: Vec::new(),
                locked: true,
                entry_node_id: None,
                exit_node_id: None,
                kind: GroupKind::FieldBoundary {
                    field_id: *field_id,
                    node_spacing: spacing,
                    offset,
                    straighten_tolerance: tolerance,
                    corner_angle_threshold: corner_angle,
                    base: GroupBase {
                        direction,
                        priority,
                        max_segment_length: 0.0,
                    },
                },
            };
            state.group_registry.register(record);
        }
        log::debug!(
            "Segment-Registry: {} Feld-Segmente registriert",
            field_segments.len()
        );
    }

    // Selektion auf neu erstellte Nodes setzen
    state.selection.ids_mut().clear();
    for &id in &all_new_ids {
        state.selection.ids_mut().insert(id);
    }
    state.selection.selection_anchor_node_id = all_new_ids.last().copied();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Grundtest: Flaeche eines 2×2-Quadrats = 4.
    #[test]
    fn test_polygon_area_square() {
        let verts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        let area = polygon_area(&verts);
        assert!(
            (area - 4.0).abs() < 1e-4,
            "Erwartete Flaeche 4.0, bekam {}",
            area
        );
    }

    /// Ein Dreieck mit bekannter Flaeche.
    #[test]
    fn test_polygon_area_triangle() {
        let verts = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(0.0, 3.0),
        ];
        let area = polygon_area(&verts);
        assert!(
            (area - 6.0).abs() < 1e-4,
            "Erwartete Flaeche 6.0, bekam {}",
            area
        );
    }

    /// Weniger als 3 Vertices → Flaeche 0.
    #[test]
    fn test_polygon_area_too_few_vertices() {
        assert_eq!(polygon_area(&[]), 0.0);
        assert_eq!(polygon_area(&[Vec2::ZERO]), 0.0);
        assert_eq!(polygon_area(&[Vec2::ZERO, Vec2::ONE]), 0.0);
    }
}
