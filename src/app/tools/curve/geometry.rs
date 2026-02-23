//! Geometrie-Berechnungen für Kurven-Tools (Arc-Length-Parametrisierung, Bézier, Catmull-Rom).

use super::state::CurveDegree;
use crate::app::tools::{common, ToolAnchor, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

/// Gleichmäßig verteilte Punkte entlang einer parametrischen Kurve (Arc-Length).
pub fn compute_curve_positions(eval: impl Fn(f32) -> Vec2, max_segment_length: f32) -> Vec<Vec2> {
    let start = eval(0.0);
    let total_length = approx_length(&eval, 128);
    if total_length < f32::EPSILON {
        return vec![start];
    }

    let segment_count = (total_length / max_segment_length).ceil().max(1.0) as usize;
    let target_spacing = total_length / segment_count as f32;

    // TODO(Performance): LUT bei mehreren Aufrufen mit identischer Kurve cachen
    // (z.B. in CurveTool::preview vs execute). Derzeit ~1 KB Allokation pro Mausbewegung.
    let lut_samples = 256;
    let mut arc_lengths = Vec::with_capacity(lut_samples + 1);
    let mut prev = start;
    let mut cumulative = 0.0f32;
    arc_lengths.push(0.0f32);
    for i in 1..=lut_samples {
        let t = i as f32 / lut_samples as f32;
        let p = eval(t);
        cumulative += prev.distance(p);
        arc_lengths.push(cumulative);
        prev = p;
    }

    let mut positions = Vec::with_capacity(segment_count + 1);
    positions.push(start);

    for seg in 1..segment_count {
        let target_length = seg as f32 * target_spacing;
        let idx = arc_lengths
            .partition_point(|&len| len < target_length)
            .min(lut_samples)
            .max(1);

        let len_before = arc_lengths[idx - 1];
        let len_after = arc_lengths[idx];
        let frac = if (len_after - len_before).abs() > f32::EPSILON {
            (target_length - len_before) / (len_after - len_before)
        } else {
            0.0
        };

        let t = ((idx - 1) as f32 + frac) / lut_samples as f32;
        positions.push(eval(t));
    }

    positions.push(eval(1.0));
    positions
}

/// Approximierte Kurvenlänge über Polylinien-Segmente.
pub fn approx_length(positions_fn: impl Fn(f32) -> Vec2, samples: usize) -> f32 {
    let mut length = 0.0;
    let mut prev = positions_fn(0.0);
    for i in 1..=samples {
        let t = i as f32 / samples as f32;
        let p = positions_fn(t);
        length += prev.distance(p);
        prev = p;
    }
    length
}

/// B(t) = (1-t)²·P0 + 2(1-t)t·P1 + t²·P2
pub fn quadratic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let inv = 1.0 - t;
    inv * inv * p0 + 2.0 * inv * t * p1 + t * t * p2
}

/// B(t) = (1-t)³·P0 + 3(1-t)²t·P1 + 3(1-t)t²·P2 + t³·P3
pub fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let inv = 1.0 - t;
    let inv2 = inv * inv;
    let t2 = t * t;
    inv2 * inv * p0 + 3.0 * inv2 * t * p1 + 3.0 * inv * t2 * p2 + t2 * t * p3
}

/// Berechnet die Position eines Kontrollpunkts basierend auf einer Tangente.
///
/// - `anchor_pos`: Position des Snap-Nodes (Start oder Ende der Kurve)
/// - `tangent_angle`: Winkel der gewählten Verbindung (Radiant)
/// - `other_anchor_pos`: Position des anderen Kurven-Endpunkts
/// - `is_start`: true = CP1 (Startseite), false = CP2 (Endseite)
///
/// Der CP wird im Abstand chord_length/3 entlang der Tangente platziert.
pub fn compute_tangent_cp(
    anchor_pos: Vec2,
    tangent_angle: f32,
    other_anchor_pos: Vec2,
    is_start: bool,
) -> Vec2 {
    let chord_length = anchor_pos.distance(other_anchor_pos);
    let cp_distance = chord_length / 3.0;
    let direction = if is_start {
        // CP1: In Fortsetzungsrichtung der Verbindung (weg vom Nachbar, zum Kurveninneren)
        Vec2::from_angle(tangent_angle + std::f32::consts::PI)
    } else {
        // CP2: In Gegenrichtung (Kurve soll tangential ankommen)
        Vec2::from_angle(tangent_angle)
    };
    anchor_pos + direction * cp_distance
}

/// Evaluiert die Kurvenposition für den aktuellen Grad.
pub fn eval_curve(
    degree: CurveDegree,
    start: Vec2,
    end: Vec2,
    cp1: Vec2,
    cp2: Option<Vec2>,
    t: f32,
) -> Vec2 {
    match degree {
        CurveDegree::Quadratic => quadratic_bezier(start, cp1, end, t),
        CurveDegree::Cubic => cubic_bezier(start, cp1, cp2.unwrap_or(cp1), end, t),
    }
}

/// Parameter-Bundle für build_tool_result (Clippy: max 7 Parameter).
#[derive(Clone, Copy)]
pub struct CurveParams {
    pub degree: CurveDegree,
    pub cp1: Vec2,
    pub cp2: Option<Vec2>,
    pub max_segment_length: f32,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
}

/// Gemeinsame Logik für execute() und execute_from_anchors().
pub fn build_tool_result(
    start: &ToolAnchor,
    end: &ToolAnchor,
    params: &CurveParams,
    road_map: &RoadMap,
) -> Option<ToolResult> {
    let CurveParams {
        degree,
        cp1,
        cp2,
        max_segment_length,
        direction,
        priority,
    } = *params;
    let start_pos = start.position();
    let end_pos = end.position();

    let positions = compute_curve_positions(
        |t| eval_curve(degree, start_pos, end_pos, cp1, cp2, t),
        max_segment_length,
    );

    Some(common::assemble_tool_result(
        &positions, start, end, direction, priority, road_map,
    ))
}

/// Berechnet CP1 und CP2 symmetrisch aus einem gewünschten Scheitelpunkt.
///
/// Der Apex entspricht B(0.5) der resultierenden Kurve.
/// Beide CPs erhalten die gleiche laterale Abweichung von der Sehne.
pub fn cps_from_apex_symmetric(p0: Vec2, p3: Vec2, apex: Vec2) -> (Vec2, Vec2) {
    let chord = p3 - p0;
    let chord_len = chord.length();
    if chord_len < f32::EPSILON {
        return (p0, p3);
    }
    let chord_dir = chord / chord_len;
    let perp = Vec2::new(-chord_dir.y, chord_dir.x);
    // Lateraler Anteil des Apex: dot(apex - midpoint, perp)
    let midpoint = (p0 + p3) * 0.5;
    let lateral = (apex - midpoint).dot(perp) * (4.0 / 3.0);
    let cp1 = p0 + chord_dir * (chord_len / 3.0) + perp * lateral;
    let cp2 = p3 - chord_dir * (chord_len / 3.0) + perp * lateral;
    (cp1, cp2)
}

/// Berechnet CP2 so, dass B(0.5) = `apex`, bei fixiertem CP1.
///
/// Aus der Formel B(0.5) = (P0 + 3·CP1 + 3·CP2 + P3) / 8 gelöst nach CP2:
/// `CP2 = (8·apex − P0 − 3·CP1 − P3) / 3`
pub fn cp2_from_apex(p0: Vec2, cp1: Vec2, apex: Vec2, p3: Vec2) -> Vec2 {
    (8.0 * apex - p0 - 3.0 * cp1 - p3) / 3.0
}

/// Berechnet CP1 so, dass B(0.5) = `apex`, bei fixiertem CP2.
///
/// Aus der Formel B(0.5) = (P0 + 3·CP1 + 3·CP2 + P3) / 8 gelöst nach CP1:
/// `CP1 = (8·apex − P0 − 3·CP2 − P3) / 3`
pub fn cp1_from_apex(p0: Vec2, apex: Vec2, cp2: Vec2, p3: Vec2) -> Vec2 {
    (8.0 * apex - p0 - 3.0 * cp2 - p3) / 3.0
}

/// Projiziert `cursor` auf die Tangenten-Linie durch `anchor`.
///
/// Der erlaubte Freiheitsgrad für CP1 (is_start=true) bzw. CP2 (is_start=false)
/// ist die Gerade in tangentialer Richtung. Negative Parameterwerte sind zulässig
/// (ermöglicht S-Kurven hinter dem Anker).
///
/// Die Richtungskonvention ist identisch mit `compute_tangent_cp`:
/// - `is_start = true` (CP1): Richtung `angle + π` (weg vom Nachbar)
/// - `is_start = false` (CP2): Richtung `angle` (zum Nachbar)
pub fn project_onto_tangent_line(anchor: Vec2, angle: f32, cursor: Vec2, is_start: bool) -> Vec2 {
    let dir = if is_start {
        Vec2::from_angle(angle + std::f32::consts::PI)
    } else {
        Vec2::from_angle(angle)
    };
    let t = (cursor - anchor).dot(dir);
    anchor + dir * t
}

/// Löst das 2×2-System für beide Tangenten-Abstände bei gegebenem Apex B(0.5).
///
/// Gegeben CP1 = p0 + t1·dir1 und CP2 = p3 + t2·dir2 sowie B(0.5) = apex,
/// wird aus der kubischen Bézier-Formel das lineare System gelöst:
///
/// ```text
/// t1·dir1 + t2·dir2 = (8·apex − 4·(p0 + p3)) / 3
/// ```
///
/// Gibt `None` wenn die Tangenten parallel (|det| < ε) sind.
pub fn solve_cps_from_apex_both_tangents(
    p0: Vec2,
    p3: Vec2,
    dir1: Vec2,
    dir2: Vec2,
    apex: Vec2,
) -> Option<(Vec2, Vec2)> {
    let det = dir1.x * dir2.y - dir1.y * dir2.x;
    if det.abs() < 1e-6 {
        return None;
    }
    let r = (8.0 * apex - 4.0 * (p0 + p3)) / 3.0;
    let t1 = (r.x * dir2.y - r.y * dir2.x) / det;
    let t2 = (dir1.x * r.y - dir1.y * r.x) / det;
    Some((p0 + t1 * dir1, p3 + t2 * dir2))
}
