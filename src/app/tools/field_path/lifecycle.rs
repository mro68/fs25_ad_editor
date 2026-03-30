//! RouteTool-Implementierung und Kernlogik des FieldPathTool.
//!
//! Enthaelt:
//! - `compute_centerline()` — Voronoi-BFS oder Grenzsegment-Rasterisierung
//! - `on_click()` / `preview()` / `execute()` / `reset()`
//! - Hilfsfunktionen fuer Feld- und Segmentauswahl

use std::sync::Arc;

use crate::app::tools::{ToolAction, ToolPreview, ToolResult};
use crate::core::{
    compute_polygon_centerline, compute_segment_centerline, find_polygon_at, simplify_polyline,
    FarmlandGrid, FieldPolygon, NodeFlag, RoadMap,
};
use crate::shared::spline_geometry::resample_by_distance;
use glam::Vec2;

use super::state::{FieldPathMode, FieldPathPhase, FieldPathTool};

/// Maximaler Abstand (Weltkoordinaten) fuer das Snap auf Grenz-Segmente.
const BOUNDARY_SNAP_THRESHOLD: f32 = 20.0;

// ---------------------------------------------------------------------------
// Interne Logik-Methoden
// ---------------------------------------------------------------------------

impl FieldPathTool {
    /// Berechnet die Mittellinie basierend auf den aktuell gewaehlten Seiten.
    ///
    /// Nutzt je nach Modus entweder polygon-basierte Kantenmittlung (Fields)
    /// oder segment-basierte Mittlung (Boundaries).
    /// Das Ergebnis wird in `self.centerline` und `self.resampled_nodes` gespeichert.
    /// Bei Erfolg wechselt die Phase auf `Preview`.
    pub(crate) fn compute_centerline(&mut self) {
        let raw = match self.mode {
            FieldPathMode::Fields => {
                if self.side1_field_ids.is_empty() || self.side2_field_ids.is_empty() {
                    log::warn!("Berechnung abgebrochen: Seite 1 oder Seite 2 ohne Felder");
                    return;
                }
                let Some(polygons) = &self.farmland_polygons else {
                    log::warn!("Keine Farmland-Polygone vorhanden — Berechnung nicht moeglich");
                    return;
                };

                // Polygon-Vertices fuer beide Seiten sammeln
                let side1_verts: Vec<&[Vec2]> = polygons
                    .iter()
                    .filter(|p| self.side1_field_ids.contains(&p.id))
                    .map(|p| p.vertices.as_slice())
                    .collect();
                let side2_verts: Vec<&[Vec2]> = polygons
                    .iter()
                    .filter(|p| self.side2_field_ids.contains(&p.id))
                    .map(|p| p.vertices.as_slice())
                    .collect();

                if side1_verts.is_empty() || side2_verts.is_empty() {
                    log::warn!("Polygon-Daten fuer gewaehlte Feld-IDs nicht gefunden");
                    return;
                }

                log::info!(
                    "Polygon-Centerline: {} Polygone Seite 1, {} Polygone Seite 2",
                    side1_verts.len(),
                    side2_verts.len()
                );
                compute_polygon_centerline(&side1_verts, &side2_verts, 2.0)
            }
            FieldPathMode::Boundaries => {
                if self.side1_segments.is_empty() || self.side2_segments.is_empty() {
                    log::warn!("Berechnung abgebrochen: Seite 1 oder Seite 2 ohne Grenzsegmente");
                    return;
                }
                compute_segment_centerline(&self.side1_segments, &self.side2_segments, 2.0)
            }
        };

        if raw.is_empty() {
            log::warn!(
                "Mittellinie ergab keine Punkte — kein Korridor zwischen den Seiten gefunden"
            );
            self.centerline.clear();
            self.resampled_nodes.clear();
            self.phase = FieldPathPhase::Preview;
            return;
        }

        let simplified = simplify_polyline(&raw, self.config.simplify_tolerance);
        let resampled = resample_by_distance(&simplified, self.config.node_spacing);

        log::info!(
            "Mittellinie: {} Rohpunkte -> {} vereinfacht -> {} Nodes (spacing={:.1}m)",
            raw.len(),
            simplified.len(),
            resampled.len(),
            self.config.node_spacing,
        );

        self.centerline = simplified;
        self.resampled_nodes = resampled;
        self.phase = FieldPathPhase::Preview;
    }

    /// Sucht das naechste Polygon-Grenz-Segment an der Klickposition.
    ///
    /// Iteriert ueber alle Kanten aller Polygone und gibt das naechste
    /// Segment zurueck, das innerhalb von `BOUNDARY_SNAP_THRESHOLD` liegt.
    fn find_nearest_boundary_segment(click: Vec2, polygons: &[FieldPolygon]) -> Option<Vec<Vec2>> {
        let mut best_dist = BOUNDARY_SNAP_THRESHOLD;
        let mut best_seg: Option<Vec<Vec2>> = None;

        for poly in polygons {
            let n = poly.vertices.len();
            if n < 2 {
                continue;
            }
            for i in 0..n {
                let a = poly.vertices[i];
                let b = poly.vertices[(i + 1) % n];
                let dist = point_to_segment_dist(click, a, b);
                if dist < best_dist {
                    best_dist = dist;
                    best_seg = Some(vec![a, b]);
                }
            }
        }

        best_seg
    }

    /// Toggelt ein Grenzsegment in der Liste (hinzufuegen oder entfernen).
    ///
    /// Vergleich ist ungerichtet: [a,b] und [b,a] gelten als identisch.
    fn toggle_segment(list: &mut Vec<Vec<Vec2>>, seg: Vec<Vec2>) {
        let existing_idx = list.iter().position(|s| {
            s.len() == 2
                && seg.len() == 2
                && ((s[0] == seg[0] && s[1] == seg[1]) || (s[0] == seg[1] && s[1] == seg[0]))
        });
        if let Some(idx) = existing_idx {
            list.remove(idx);
        } else {
            list.push(seg);
        }
    }

    /// Verarbeitet einen Auswahlklick fuer Seite 1 oder Seite 2.
    fn handle_selection_click(&mut self, pos: Vec2, is_side1: bool) {
        match self.mode {
            FieldPathMode::Fields => {
                let Some(polygons) = self.farmland_polygons.clone() else {
                    log::warn!("Keine Farmland-Polygone geladen — Feld-Auswahl nicht moeglich");
                    return;
                };
                if let Some(poly) = find_polygon_at(pos, &polygons) {
                    let id = poly.id;
                    if is_side1 {
                        toggle_u32(&mut self.side1_field_ids, id);
                    } else {
                        if self.side1_field_ids.contains(&id) {
                            log::info!(
                                "Feld #{} ist bereits in Seite 1 — Zuweisung zu Seite 2 ignoriert",
                                id
                            );
                            return;
                        }
                        toggle_u32(&mut self.side2_field_ids, id);
                    }
                }
            }
            FieldPathMode::Boundaries => {
                let Some(polygons) = self.farmland_polygons.clone() else {
                    log::warn!("Keine Farmland-Polygone geladen — Grenz-Auswahl nicht moeglich");
                    return;
                };
                if let Some(seg) = Self::find_nearest_boundary_segment(pos, &polygons) {
                    if is_side1 {
                        Self::toggle_segment(&mut self.side1_segments, seg);
                    } else {
                        Self::toggle_segment(&mut self.side2_segments, seg);
                    }
                }
            }
        }
    }

    /// Baut die Vorschau fuer die Auswahlphasen (Polygon-Umrisse der gewaehlten Felder/Segmente).
    fn build_selection_preview(&self) -> ToolPreview {
        let Some(polygons) = &self.farmland_polygons else {
            return ToolPreview::default();
        };

        let mut nodes: Vec<Vec2> = Vec::new();
        let mut connections: Vec<(usize, usize)> = Vec::new();

        // Seite-1-Felder: Polygon-Umrisse
        for &id in &self.side1_field_ids {
            if let Some(poly) = polygons.iter().find(|p| p.id == id) {
                let start = nodes.len();
                nodes.extend_from_slice(&poly.vertices);
                let n = poly.vertices.len();
                for i in 0..n {
                    connections.push((start + i, start + (i + 1) % n));
                }
            }
        }

        // Seite-2-Felder: Polygon-Umrisse
        for &id in &self.side2_field_ids {
            if let Some(poly) = polygons.iter().find(|p| p.id == id) {
                let start = nodes.len();
                nodes.extend_from_slice(&poly.vertices);
                let n = poly.vertices.len();
                for i in 0..n {
                    connections.push((start + i, start + (i + 1) % n));
                }
            }
        }

        // Seite-1-Grenzsegmente
        for seg in &self.side1_segments {
            if seg.len() >= 2 {
                let start = nodes.len();
                nodes.extend_from_slice(seg);
                for i in 0..seg.len() - 1 {
                    connections.push((start + i, start + i + 1));
                }
            }
        }

        // Seite-2-Grenzsegmente
        for seg in &self.side2_segments {
            if seg.len() >= 2 {
                let start = nodes.len();
                nodes.extend_from_slice(seg);
                for i in 0..seg.len() - 1 {
                    connections.push((start + i, start + i + 1));
                }
            }
        }

        let cn = connections.len();
        ToolPreview {
            nodes,
            connections,
            connection_styles: vec![(self.direction, self.priority); cn],
            labels: vec![],
        }
    }

    /// Baut die Vorschau fuer die Preview-Phase (Mittellinie als Node-Kette).
    fn build_path_preview(&self) -> ToolPreview {
        if self.resampled_nodes.is_empty() {
            return ToolPreview::default();
        }
        let n = self.resampled_nodes.len();
        let connections: Vec<(usize, usize)> =
            (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect();
        ToolPreview {
            nodes: self.resampled_nodes.clone(),
            connections: connections.clone(),
            connection_styles: vec![(self.direction, self.priority); connections.len()],
            labels: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// RouteTool-Implementierung
// ---------------------------------------------------------------------------

impl crate::app::tools::RouteTool for FieldPathTool {
    fn name(&self) -> &str {
        "Feldweg"
    }

    fn icon(&self) -> &str {
        "\u{1f6e4}" // 🛤
    }

    fn description(&self) -> &str {
        "Berechnet Mittellinien zwischen Farmland-Grenzen"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            FieldPathPhase::Idle => "Tool aktiv — Seite 1 waehlen oder Starten klicken",
            FieldPathPhase::SelectingSide1 => match self.mode {
                FieldPathMode::Fields => {
                    "Felder fuer Seite 1 klicken (nochmal klicken = entfernen)"
                }
                FieldPathMode::Boundaries => {
                    "Feldgrenzen fuer Seite 1 klicken (nochmal klicken = entfernen)"
                }
            },
            FieldPathPhase::SelectingSide2 => match self.mode {
                FieldPathMode::Fields => "Felder fuer Seite 2 klicken — dann Berechnen",
                FieldPathMode::Boundaries => "Feldgrenzen fuer Seite 2 klicken — dann Berechnen",
            },
            FieldPathPhase::Preview => "Vorschau — Uebernehmen oder Seiten anpassen",
        }
    }

    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            FieldPathPhase::Idle => {
                self.phase = FieldPathPhase::SelectingSide1;
                self.handle_selection_click(pos, true);
                ToolAction::Continue
            }
            FieldPathPhase::SelectingSide1 => {
                self.handle_selection_click(pos, true);
                ToolAction::Continue
            }
            FieldPathPhase::SelectingSide2 => {
                self.handle_selection_click(pos, false);
                ToolAction::Continue
            }
            FieldPathPhase::Preview => ToolAction::Continue,
        }
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        match self.phase {
            FieldPathPhase::Idle => ToolPreview::default(),
            FieldPathPhase::SelectingSide1 | FieldPathPhase::SelectingSide2 => {
                self.build_selection_preview()
            }
            FieldPathPhase::Preview => self.build_path_preview(),
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, distance_wheel_step_m)
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if self.phase != FieldPathPhase::Preview || self.resampled_nodes.is_empty() {
            return None;
        }

        let n = self.resampled_nodes.len();
        let new_nodes: Vec<(Vec2, NodeFlag)> = self
            .resampled_nodes
            .iter()
            .map(|&pos| (pos, NodeFlag::Regular))
            .collect();

        // Kette: 0→1, 1→2, ...
        let internal_connections = (0..n.saturating_sub(1))
            .map(|i| (i, i + 1, self.direction, self.priority))
            .collect();

        // Externe Verbindungen an Start und Ende
        let mut external_connections = Vec::new();
        if self.config.connect_to_existing {
            if let Some(&start_pos) = self.resampled_nodes.first() {
                if let Some(hit) = road_map.nearest_node(start_pos) {
                    // Verbindung vom existierenden Node zum ersten neuen Node
                    external_connections.push((
                        0,
                        hit.node_id,
                        true,
                        self.direction,
                        self.priority,
                    ));
                }
            }
            if n >= 2 {
                if let Some(&end_pos) = self.resampled_nodes.last() {
                    if let Some(hit) = road_map.nearest_node(end_pos) {
                        // Verbindung vom letzten neuen Node zum existierenden Node
                        external_connections.push((
                            n - 1,
                            hit.node_id,
                            false,
                            self.direction,
                            self.priority,
                        ));
                    }
                }
            }
        }

        Some(ToolResult {
            new_nodes,
            internal_connections,
            external_connections,
            markers: Vec::new(),
            nodes_to_remove: Vec::new(),
        })
    }

    fn reset(&mut self) {
        self.phase = FieldPathPhase::Idle;
        self.side1_field_ids.clear();
        self.side2_field_ids.clear();
        self.side1_segments.clear();
        self.side2_segments.clear();
        self.centerline.clear();
        self.resampled_nodes.clear();
    }

    fn is_ready(&self) -> bool {
        self.phase == FieldPathPhase::Preview && !self.resampled_nodes.is_empty()
    }

    fn has_pending_input(&self) -> bool {
        !matches!(self.phase, FieldPathPhase::Idle)
    }

    fn set_farmland_data(&mut self, data: Option<Arc<Vec<FieldPolygon>>>) {
        self.farmland_polygons = data;
    }

    fn set_farmland_grid(&mut self, grid: Option<Arc<FarmlandGrid>>) {
        self.farmland_grid = grid;
        // Voronoi-Cache ungueltig machen bei Grid-Wechsel
        self.voronoi_cache = None;
    }

    fn set_background_map_image(&mut self, image: Option<std::sync::Arc<image::DynamicImage>>) {
        self.background_image = image;
    }

    crate::impl_lifecycle_delegation_no_seg!();
}

// ---------------------------------------------------------------------------
// Private Hilfsfunktionen
// ---------------------------------------------------------------------------

/// Euklidischer Punkt-zu-Segment-Abstand.
fn point_to_segment_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Toggelt eine u32-ID in einem Vec (hinzufuegen oder entfernen).
fn toggle_u32(list: &mut Vec<u32>, id: u32) {
    if let Some(pos) = list.iter().position(|&x| x == id) {
        list.remove(pos);
    } else {
        list.push(id);
    }
}
