//! Solver-Logik fuer das Constraint-Route-Tool.
//!
//! Pipeline: Waypoint-Kette → Approach/Departure-Steerer → Subdivide → Chaikin-Corner-Cutting
//! → Positionen an `common::assemble_tool_result()` uebergeben.
//!
//! Kein finales Resampling — die kruemmungsadaptive Verteilung des Corner-Cuttings
//! wird beibehalten (mehr Nodes in Kurven, weniger auf Geraden).

use super::super::{common, ToolAnchor, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

/// Eingabe-Parameter fuer den Geglättete-Kurve-Solver.
#[derive(Debug, Clone)]
pub struct SmoothCurveInput {
    /// Startposition der Route
    pub start: Vec2,
    /// Endposition der Route
    pub end: Vec2,
    /// Vom User gesetzte Zwischen-Kontrollpunkte (inkl. ggf. manuell verschobene Steuerpunkte)
    pub control_nodes: Vec<Vec2>,
    /// Maximaler Abstand zwischen aufeinanderfolgenden Nodes (Meter)
    pub max_segment_length_m: f32,
    /// Maximale Richtungsaenderung pro Segment (Grad)
    pub max_direction_change_deg: f32,
    /// Richtungsvektoren bestehender Verbindungen am Startpunkt
    pub start_neighbor_directions: Vec<Vec2>,
    /// Richtungsvektoren bestehender Verbindungen am Endpunkt
    pub end_neighbor_directions: Vec<Vec2>,
    /// Minimaldistanz: Nodes die naeher beieinander liegen werden gefiltert (Meter)
    pub min_distance: f32,
}

/// Ergebnis des Solvers: Positionen + berechnete Steuerpunkte.
#[derive(Debug, Clone)]
pub struct SolverResult {
    /// Berechnete Polyline (inkl. Start/Ende).
    pub positions: Vec<Vec2>,
    /// Automatischer Approach-Steuerpunkt am Start (wenn berechnet).
    pub approach_steerer: Option<Vec2>,
    /// Automatischer Departure-Steuerpunkt am Ende (wenn berechnet).
    pub departure_steerer: Option<Vec2>,
}

/// Hauptfunktion des Geglättete-Kurve-Solvers.
///
/// Erzeugt eine geglaettete Waypoint-Kette die:
/// 1. Durch Start, Kontrollpunkte und End verlaeuft
/// 2. Winkel-Constraints einhaelt (Chaikin-Corner-Cutting)
/// 3. Glatte Uebergaenge zu bestehenden Verbindungen sicherstellt (Steerer-Nodes)
/// 4. Gleichmaessig abgetastete Punkte mit ≤ max_segment_length Abstand liefert
pub fn solve_route(input: &SmoothCurveInput) -> SolverResult {
    let max_angle_rad = input.max_direction_change_deg.to_radians();
    let total_dist = input.start.distance(input.end);

    // Schritt 1: Steerer-Nodes berechnen
    let first_target = input.control_nodes.first().copied().unwrap_or(input.end);
    let forward_dir = (first_target - input.start).normalize_or_zero();

    let approach_steerer = compute_approach_steerer(
        input.start,
        forward_dir,
        &input.start_neighbor_directions,
        max_angle_rad,
        input.max_segment_length_m,
        total_dist,
    );

    let last_before_end = input.control_nodes.last().copied().unwrap_or(input.start);
    let forward_to_end = (input.end - last_before_end).normalize_or_zero();

    let departure_steerer = compute_departure_steerer(
        input.end,
        forward_to_end,
        &input.end_neighbor_directions,
        max_angle_rad,
        input.max_segment_length_m,
        total_dist,
    );

    // Schritt 2: Waypoint-Kette aufbauen
    let mut waypoints = Vec::with_capacity(input.control_nodes.len() + 4);
    waypoints.push(input.start);
    if let Some(s) = approach_steerer {
        waypoints.push(s);
    }
    waypoints.extend_from_slice(&input.control_nodes);
    if let Some(e) = departure_steerer {
        waypoints.push(e);
    }
    waypoints.push(input.end);

    // Schritt 3: Subdivide
    let mut positions = subdivide_polyline(&waypoints, input.max_segment_length_m);

    // Schritt 4: Chaikin-Corner-Cutting (iterativ scharfe Ecken glaetten)
    // Kein finales Resampling — die Punkte bleiben dort, wo das Corner-Cutting
    // sie natuerlich platziert. Das ergibt eine kruemmungsadaptive Verteilung:
    // mehr Nodes in Kurven, weniger auf geraden Strecken.
    for _ in 0..128 {
        if max_turn_angle(&positions) <= max_angle_rad {
            break;
        }
        if !smooth_first_sharp_corner(&mut positions, max_angle_rad) {
            break;
        }
        positions = subdivide_polyline(&positions, input.max_segment_length_m);
    }

    // Schritt 5: Minimaldistanz-Filter — zu nahe beieinanderliegende Nodes entfernen
    if input.min_distance > 0.0 {
        positions = filter_min_distance(&positions, input.min_distance);
    }

    SolverResult {
        positions,
        approach_steerer,
        departure_steerer,
    }
}

/// Berechnet den maximalen Eckwinkel einer Polyline.
fn max_turn_angle(points: &[Vec2]) -> f32 {
    let mut max_angle = 0.0_f32;
    for i in 1..points.len().saturating_sub(1) {
        let v1 = points[i] - points[i - 1];
        let v2 = points[i + 1] - points[i];
        let la = v1.length_squared();
        let lb = v2.length_squared();
        if la < f32::EPSILON || lb < f32::EPSILON {
            continue;
        }
        let dot = v1.normalize().dot(v2.normalize()).clamp(-1.0, 1.0);
        let angle = dot.acos();
        if angle > max_angle {
            max_angle = angle;
        }
    }
    max_angle
}

/// Chaikin-Corner-Cutting: Ersetzt die erste zu scharfe Ecke durch zwei Punkte.
///
/// Gibt `true` zurueck wenn eine Ecke geglaettet wurde.
fn smooth_first_sharp_corner(points: &mut Vec<Vec2>, max_turn_rad: f32) -> bool {
    if points.len() < 3 {
        return false;
    }

    for i in 1..points.len() - 1 {
        let prev = points[i - 1];
        let curr = points[i];
        let next = points[i + 1];

        let v1 = curr - prev;
        let v2 = next - curr;
        let la = v1.length_squared();
        let lb = v2.length_squared();
        if la < f32::EPSILON || lb < f32::EPSILON {
            continue;
        }

        let dot = v1.normalize().dot(v2.normalize()).clamp(-1.0, 1.0);
        let angle = dot.acos();
        if angle <= max_turn_rad {
            continue;
        }

        // Chaikin-Corner-Cutting: harte Ecke durch zwei weichere Punkte ersetzen
        let q = curr + (prev - curr) * 0.25;
        let r = curr + (next - curr) * 0.25;
        points.splice(i..=i, [q, r]);
        return true;
    }

    false
}

/// Berechnet einen Approach-Steerer-Node am Startpunkt.
///
/// Sucht die Nachbar-Richtung die am staerksten gegen `forward` zeigt
/// (= Ankunftsrichtung). Wenn der Winkel das Limit ueberschreitet,
/// wird ein Steuerpunkt in Anfahrtsrichtung platziert.
///
/// Reine Geometrie — kein RoadMap- oder AppState-Zugriff.
pub fn compute_approach_steerer(
    node_pos: Vec2,
    forward: Vec2,
    neighbor_directions: &[Vec2],
    max_angle_rad: f32,
    max_segment: f32,
    total_dist: f32,
) -> Option<Vec2> {
    if neighbor_directions.is_empty() || forward.length_squared() < 0.01 {
        return None;
    }

    // Nachbar-Richtung "hinter" dem Start: to_neighbor ≈ -forward → maximaler Score
    let best_dir = neighbor_directions
        .iter()
        .max_by(|a, b| {
            let sa = a.normalize_or_zero().dot(-forward);
            let sb = b.normalize_or_zero().dot(-forward);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()?;

    // Anfahrtsrichtung: umgekehrt zur Nachbar-Richtung (= Fahrtrichtung an diesem Punkt)
    let approach_dir = (-best_dir).normalize_or_zero();
    let angle = approach_dir.dot(forward).clamp(-1.0, 1.0).acos();

    if angle <= max_angle_rad {
        return None;
    }

    // Dynamischer Abstand: proportional zur Gesamtstrecke, aber begrenzt
    let step = max_segment.min(total_dist * 0.4).max(max_segment * 0.5);
    Some(node_pos + approach_dir * step)
}

/// Berechnet einen Departure-Steerer-Node am Endpunkt.
///
/// Sucht die Nachbar-Richtung die am staerksten in `forward`-Richtung zeigt.
/// Die Route soll tangential am End-Node ankommen.
///
/// Reine Geometrie — kein RoadMap- oder AppState-Zugriff.
pub fn compute_departure_steerer(
    node_pos: Vec2,
    forward: Vec2,
    neighbor_directions: &[Vec2],
    max_angle_rad: f32,
    max_segment: f32,
    total_dist: f32,
) -> Option<Vec2> {
    if neighbor_directions.is_empty() || forward.length_squared() < 0.01 {
        return None;
    }

    // Nachbar-Richtung "vor" dem Ende: to_neighbor ≈ forward → maximaler Score
    let best_dir = neighbor_directions
        .iter()
        .max_by(|a, b| {
            let sa = a.normalize_or_zero().dot(forward);
            let sb = b.normalize_or_zero().dot(forward);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()?;

    let depart_dir = best_dir.normalize_or_zero();
    let angle = depart_dir.dot(forward).clamp(-1.0, 1.0).acos();

    if angle <= max_angle_rad {
        return None;
    }

    // Steuerpunkt VOR dem Ende platzieren (Route kommt tangential an)
    let step = max_segment.min(total_dist * 0.4).max(max_segment * 0.5);
    Some(node_pos - depart_dir * step)
}

/// Filtert Nodes die naeher als `min_dist` beieinanderliegen.
///
/// Start- und Endpunkt werden immer beibehalten.
fn filter_min_distance(points: &[Vec2], min_dist: f32) -> Vec<Vec2> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let min_dist_sq = min_dist * min_dist;
    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]);

    let last = *points
        .last()
        .expect("invariant: points hat mehr als 2 Elemente nach len()<=2-Guard");
    for &p in &points[1..points.len() - 1] {
        let prev = *result
            .last()
            .expect("invariant: result ist nicht-leer – points[0] wurde gepusht");
        // Punkt beibehalten wenn weit genug vom letzten behaltenen entfernt
        // UND weit genug vom Endpunkt entfernt
        if prev.distance_squared(p) >= min_dist_sq && p.distance_squared(last) >= min_dist_sq {
            result.push(p);
        }
    }
    result.push(last);
    result
}

/// Unterteilt eine Polyline so, dass kein Segment laenger als `max_length` ist.
fn subdivide_polyline(points: &[Vec2], max_length: f32) -> Vec<Vec2> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(points.len());
    result.push(points[0]);

    for window in points.windows(2) {
        let a = window[0];
        let b = window[1];
        let dist = a.distance(b);
        if dist <= max_length {
            result.push(b);
            continue;
        }
        let n = (dist / max_length).ceil().max(1.0) as usize;
        for i in 1..=n {
            let t = i as f32 / n as f32;
            result.push(a.lerp(b, t));
        }
    }

    result
}

/// Parameter fuer `build_result` — vermeidet zu viele Funktionsargumente.
pub(crate) struct BuildResultParams<'a> {
    pub start: ToolAnchor,
    pub end: ToolAnchor,
    pub control_nodes: &'a [Vec2],
    pub max_segment_length: f32,
    pub max_angle_deg: f32,
    pub start_neighbor_dirs: &'a [Vec2],
    pub end_neighbor_dirs: &'a [Vec2],
    pub min_distance: f32,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
}

/// Gemeinsame build_result-Logik fuer `execute()` und `execute_from_anchors()`.
///
/// Berechnet Solver-Positionen und delegiert Node-/Verbindungs-Aufbau an `assemble_tool_result`.
pub(crate) fn build_result(p: &BuildResultParams, road_map: &RoadMap) -> Option<ToolResult> {
    let input = SmoothCurveInput {
        start: p.start.position(),
        end: p.end.position(),
        control_nodes: p.control_nodes.to_vec(),
        max_segment_length_m: p.max_segment_length,
        max_direction_change_deg: p.max_angle_deg,
        start_neighbor_directions: p.start_neighbor_dirs.to_vec(),
        end_neighbor_directions: p.end_neighbor_dirs.to_vec(),
        min_distance: p.min_distance,
    };

    let result = solve_route(&input);
    if result.positions.len() < 2 {
        return None;
    }

    Some(common::assemble_tool_result(
        &result.positions,
        &p.start,
        &p.end,
        p.direction,
        p.priority,
        road_map,
    ))
}
