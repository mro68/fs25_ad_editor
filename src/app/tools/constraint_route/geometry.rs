//! Solver-Logik für das Constraint-Route-Tool.
//!
//! Pipeline: Waypoint-Kette → Approach/Departure-Steerer → Subdivide → Winkelglättung → Resample
//! → Positionen an `common::assemble_tool_result()` übergeben.

use super::super::{common, ToolAnchor, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

/// Eingabe-Parameter für den Constraint-Route-Solver.
#[derive(Debug, Clone)]
pub struct ConstraintRouteInput {
    /// Startposition der Route
    pub start: Vec2,
    /// Endposition der Route
    pub end: Vec2,
    /// Vom User gesetzte Zwischen-Kontrollpunkte
    pub control_nodes: Vec<Vec2>,
    /// Maximaler Abstand zwischen aufeinanderfolgenden Nodes (Meter)
    pub max_segment_length_m: f32,
    /// Maximale Richtungsänderung pro Segment (Grad)
    pub max_direction_change_deg: f32,
    /// Richtungsvektoren bestehender Verbindungen am Startpunkt
    pub start_neighbor_directions: Vec<Vec2>,
    /// Richtungsvektoren bestehender Verbindungen am Endpunkt
    pub end_neighbor_directions: Vec<Vec2>,
}

/// Hauptfunktion des Constraint-Route-Solvers.
///
/// Erzeugt eine geglättete Waypoint-Kette die:
/// 1. Durch Start, Kontrollpunkte und End verläuft
/// 2. Winkel-Constraints einhält (max. Richtungsänderung pro Segment)
/// 3. Glatte Übergänge zu bestehenden Verbindungen sicherstellt (Steerer-Nodes)
/// 4. Gleichmäßig abgetastete Punkte mit ≤ max_segment_length Abstand liefert
pub fn solve_route(input: &ConstraintRouteInput) -> Vec<Vec2> {
    let max_angle_rad = input.max_direction_change_deg.to_radians();

    // Schritt 1: Waypoint-Kette aufbauen (Start → Steerer → Control-Nodes → Steerer → End)
    let mut waypoints = Vec::new();
    waypoints.push(input.start);

    // Vorwärts-Richtung für Approach-Steerer bestimmen
    let first_target = input.control_nodes.first().copied().unwrap_or(input.end);
    let forward_dir = (first_target - input.start).normalize_or_zero();

    // Approach-Steerer am Start
    if let Some(steerer) = compute_approach_steerer(
        input.start,
        forward_dir,
        &input.start_neighbor_directions,
        max_angle_rad,
        input.max_segment_length_m,
    ) {
        waypoints.push(steerer);
    }

    // User-Kontrollpunkte
    waypoints.extend_from_slice(&input.control_nodes);

    // Departure-Steerer am Ende
    let last_before_end = input.control_nodes.last().copied().unwrap_or(input.start);
    let backward_dir = (last_before_end - input.end).normalize_or_zero();
    if let Some(steerer) = compute_departure_steerer(
        input.end,
        backward_dir,
        &input.end_neighbor_directions,
        max_angle_rad,
        input.max_segment_length_m,
    ) {
        waypoints.push(steerer);
    }

    waypoints.push(input.end);

    // Schritt 2: Zwischen aufeinanderfolgenden Waypoints subdividen
    let subdivided = subdivide_polyline(&waypoints, input.max_segment_length_m);

    // Schritt 3: Iterativ Winkelglättung
    let smoothed = smooth_angles(&subdivided, max_angle_rad, 20);

    // Schritt 4: Resampling auf gleichmäßige Abstände
    resample(&smoothed, input.max_segment_length_m)
}

/// Berechnet einen Approach-Steerer-Node am Startpunkt.
///
/// Prüft ob die Forward-Richtung einen zu scharfen Winkel zu einer bestehenden
/// Verbindung am Startpunkt bildet. Falls ja, wird ein Steerer-Punkt eingefügt
/// der den Übergang glättet.
///
/// - `node_pos`: Position des Start-Nodes
/// - `forward_dir`: Normalisierte Richtung Start → erstes Ziel
/// - `neighbor_dirs`: Richtungsvektoren der bestehenden Verbindungen am Start
/// - `max_angle_rad`: Maximaler erlaubter Winkel (Radiant)
/// - `segment_length`: Abstand des Steerer-Nodes vom Start
fn compute_approach_steerer(
    node_pos: Vec2,
    forward_dir: Vec2,
    neighbor_dirs: &[Vec2],
    max_angle_rad: f32,
    segment_length: f32,
) -> Option<Vec2> {
    if neighbor_dirs.is_empty() || forward_dir.length_squared() < 0.01 {
        return None;
    }

    // Finde den Nachbar mit dem kritischsten (kleinsten) Winkel zur Forward-Richtung
    let mut worst_angle = f32::MAX;
    let mut worst_dir = Vec2::ZERO;

    for &ndir in neighbor_dirs {
        // Winkel zwischen Forward und der Gegenrichtung des Nachbarn
        // (Gegenrichtung, weil wir vom Node WEG wollen, nicht zum Nachbar HIN)
        let away_from_neighbor = -ndir;
        let angle = forward_dir.angle_to(away_from_neighbor).abs();
        if angle < worst_angle {
            worst_angle = angle;
            worst_dir = away_from_neighbor;
        }
    }

    // Nur Steerer einfügen wenn der Winkel das Limit überschreitet
    if worst_angle > max_angle_rad {
        // Steerer in die gemittelte Richtung (Bisektrix) platzieren
        let blended = (forward_dir + worst_dir).normalize_or_zero();
        let dir = if blended.length_squared() > 0.01 {
            blended
        } else {
            forward_dir
        };
        Some(node_pos + dir * segment_length)
    } else {
        None
    }
}

/// Berechnet einen Departure-Steerer-Node am Endpunkt.
///
/// Analog zu `compute_approach_steerer`, aber für die Ankunftsrichtung am Ende.
///
/// - `node_pos`: Position des End-Nodes
/// - `backward_dir`: Normalisierte Richtung End → letzter Punkt davor
/// - `neighbor_dirs`: Richtungsvektoren der bestehenden Verbindungen am Ende
/// - `max_angle_rad`: Maximaler erlaubter Winkel (Radiant)
/// - `segment_length`: Abstand des Steerer-Nodes vom Ende
fn compute_departure_steerer(
    node_pos: Vec2,
    backward_dir: Vec2,
    neighbor_dirs: &[Vec2],
    max_angle_rad: f32,
    segment_length: f32,
) -> Option<Vec2> {
    // Departure-Steerer: Route soll glatt am End-Knoten ankommen.
    // backward_dir zeigt vom Ende zum vorletzten Punkt.
    // Wir brauchen die Ankunftsrichtung = -backward_dir
    let arrival_dir = -backward_dir;
    compute_approach_steerer(
        node_pos,
        arrival_dir,
        neighbor_dirs,
        max_angle_rad,
        segment_length,
    )
}

/// Unterteilt eine Polyline so, dass kein Segment länger als `max_length` ist.
///
/// Zwischen jedem Paar aufeinanderfolgender Punkte werden gleichmäßig
/// Zwischen-Punkte eingefügt.
fn subdivide_polyline(points: &[Vec2], max_length: f32) -> Vec<Vec2> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    result.push(points[0]);

    for window in points.windows(2) {
        let a = window[0];
        let b = window[1];
        let dist = a.distance(b);
        let segments = (dist / max_length).ceil().max(1.0) as usize;
        for i in 1..=segments {
            let t = i as f32 / segments as f32;
            result.push(a.lerp(b, t));
        }
    }

    result
}

/// Iterative Winkelglättung: Verschiebt Punkte die das Winkellimit verletzen.
///
/// In jeder Iteration werden Punkte, bei denen der Richtungswechsel
/// `max_angle_rad` überschreitet, zur Mitte der Nachbarn verschoben.
/// Start und Ende bleiben fixiert.
fn smooth_angles(points: &[Vec2], max_angle_rad: f32, max_iterations: usize) -> Vec<Vec2> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut result = points.to_vec();

    for _ in 0..max_iterations {
        let mut any_violation = false;

        for i in 1..result.len() - 1 {
            let prev = result[i - 1];
            let curr = result[i];
            let next = result[i + 1];

            let dir_in = (curr - prev).normalize_or_zero();
            let dir_out = (next - curr).normalize_or_zero();

            if dir_in.length_squared() < 0.01 || dir_out.length_squared() < 0.01 {
                continue;
            }

            let angle = dir_in.angle_to(dir_out).abs();

            if angle > max_angle_rad {
                any_violation = true;
                // Punkt zur gewichteten Mitte der Nachbarn verschieben (Laplacian-Smoothing)
                let midpoint = (prev + next) * 0.5;
                // Blend: 70% Mitte, 30% Original (vorsichtiges Glätten)
                result[i] = curr.lerp(midpoint, 0.7);
            }
        }

        if !any_violation {
            break;
        }
    }

    result
}

/// Resampled eine Polyline auf gleichmäßige Abstände ≤ `max_segment_length`.
///
/// Behält Start und Ende exakt bei. Zwischen-Punkte werden entlang der
/// Polyline in gleichmäßigen Abständen interpoliert.
fn resample(points: &[Vec2], max_segment_length: f32) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }

    // Gesamtlänge bestimmen
    let total_length: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
    if total_length < f32::EPSILON {
        return vec![points[0]];
    }

    let segment_count = (total_length / max_segment_length).ceil().max(1.0) as usize;
    let step = total_length / segment_count as f32;

    let mut result = Vec::with_capacity(segment_count + 1);
    result.push(points[0]);

    let mut current_dist = step;
    let mut seg_idx = 0;
    let mut seg_start_dist = 0.0_f32;

    for target in 1..segment_count {
        let target_dist = target as f32 * step;

        // Vorspulen zum richtigen Segment
        while seg_idx < points.len() - 1 {
            let seg_len = points[seg_idx].distance(points[seg_idx + 1]);
            if seg_start_dist + seg_len >= target_dist {
                break;
            }
            seg_start_dist += seg_len;
            seg_idx += 1;
        }

        if seg_idx >= points.len() - 1 {
            break;
        }

        let seg_len = points[seg_idx].distance(points[seg_idx + 1]);
        if seg_len < f32::EPSILON {
            result.push(points[seg_idx]);
            continue;
        }

        let local_t = (target_dist - seg_start_dist) / seg_len;
        result.push(points[seg_idx].lerp(points[seg_idx + 1], local_t.clamp(0.0, 1.0)));
        current_dist = target_dist + step;
    }
    let _ = current_dist; // Suppress unused variable warning

    // Ende exakt übernehmen
    result.push(*points.last().unwrap());

    result
}

/// Gemeinsame build_result-Logik für `execute()` und `execute_from_anchors()`.
///
/// Berechnet Solver-Positionen und delegiert Node-/Verbindungs-Aufbau an `assemble_tool_result`.
pub(crate) fn build_result(
    start: ToolAnchor,
    end: ToolAnchor,
    control_nodes: &[Vec2],
    max_segment_length: f32,
    max_angle_deg: f32,
    start_neighbor_dirs: &[Vec2],
    end_neighbor_dirs: &[Vec2],
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    road_map: &RoadMap,
) -> Option<ToolResult> {
    let input = ConstraintRouteInput {
        start: start.position(),
        end: end.position(),
        control_nodes: control_nodes.to_vec(),
        max_segment_length_m: max_segment_length,
        max_direction_change_deg: max_angle_deg,
        start_neighbor_directions: start_neighbor_dirs.to_vec(),
        end_neighbor_directions: end_neighbor_dirs.to_vec(),
    };

    let positions = solve_route(&input);
    if positions.len() < 2 {
        return None;
    }

    Some(common::assemble_tool_result(
        &positions, &start, &end, direction, priority, road_map,
    ))
}
