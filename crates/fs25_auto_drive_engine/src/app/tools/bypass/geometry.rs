//! Geometrie-Berechnungen fuer das Ausweichstrecken-Tool.
//!
//! Alle Funktionen sind pur (keine State-Mutation) und arbeiten nur mit `Vec2`-Koordinaten.
//!
//! # Geometrie
//!
//! ```text
//!  chain[0] ──S-Entry──▶ b0 ──── Hauptstrecke (versetzt) ────▶ bn ──S-Exit──▶ chain[n-1]
//! ```
//!
//! Die Hauptstrecke beginnt/endet jeweils `d_blend = |offset| × 1.5` entlang der Kette
//! versetzt, damit die S-Kurven ausreichend Laengsraum fuer tangentiale Uebergaenge haben.

use crate::app::tools::common::parallel_offset;
use crate::shared::spline_geometry::{
    catmull_rom_chain_with_tangents, polyline_length, resample_by_distance,
};
use glam::Vec2;

/// Berechnet die neuen Bypass-Knoten-Positionen.
///
/// # Parameter
/// - `chain_positions` — geordnete Kettenpositionen (mind. 2 Punkte)
/// - `offset` — seitlicher Versatz (positiv = links in Fahrtrichtung)
/// - `base_spacing` — maximaler Abstand zwischen Nodes auf der Hauptstrecke
///
/// # Rueckgabe
/// `Some((positions, d_blend))` — neue Knoten-Positionen **ohne** die Endpunkte der
/// Originalkette (diese existieren bereits in der RoadMap) sowie den verwendeten
/// d_blend-Wert.
///
/// `None` wenn der Kette zu kurz ist oder die Geometrie degeneriert.
pub fn compute_bypass_positions(
    chain_positions: &[Vec2],
    offset: f32,
    base_spacing: f32,
) -> Option<(Vec<Vec2>, f32)> {
    if chain_positions.len() < 2 {
        return None;
    }

    let base_spacing = base_spacing.max(0.5);
    let half_spacing = base_spacing * 0.5;

    // ── Catmull-Rom-Dichte-Approximation ─────────────────────────────────────
    const SAMPLES: usize = 20;
    let dense = catmull_rom_chain_with_tangents(chain_positions, SAMPLES, None, None);
    let total_len = polyline_length(&dense);
    if total_len < f32::EPSILON {
        return None;
    }

    // ── Uebergangslaenge ────────────────────────────────────────────────────────
    // Grenzen robust normalisieren, damit `min <= max` auch bei sehr kurzem Pfad gilt.
    let target_blend = offset.abs() * 1.5;
    let min_blend = offset.abs().max(0.1);
    let max_blend = total_len * 0.35;
    let d_blend = if min_blend <= max_blend {
        target_blend.clamp(min_blend, max_blend)
    } else {
        // Fallback fuer Grenzfall: kurzer Pfad + grosser Offset.
        // Der Mindestwert ist geometrisch nicht erfuellbar, daher maximal erlaubte Uebergangslaenge.
        max_blend
    };

    if d_blend * 2.0 >= total_len {
        return None;
    }

    // ── Blend-Punkte auf der Dichte-Spline ───────────────────────────────────
    let (i0, t0) = arc_split(&dense, d_blend)?;
    let (i_n, t_n) = arc_split(&dense, total_len - d_blend)?;

    let b0_chain = dense[i0].lerp(dense[i0 + 1], t0);
    let t_at_b0 = (dense[i0 + 1] - dense[i0]).normalize_or_zero();
    let perp_at_b0 = Vec2::new(-t_at_b0.y, t_at_b0.x);
    let b0 = b0_chain + perp_at_b0 * offset;

    let bn_chain = dense[i_n].lerp(dense[i_n + 1], t_n);
    let t_at_bn = (dense[i_n + 1] - dense[i_n]).normalize_or_zero();
    let perp_at_bn = Vec2::new(-t_at_bn.y, t_at_bn.x);
    let bn = bn_chain + perp_at_bn * offset;

    // ── Hauptstrecke (mittleres Teilstueck, parallel verschoben) ──────────────
    let mut main_chain: Vec<Vec2> = Vec::with_capacity(i_n - i0 + 3);
    main_chain.push(b0_chain);
    for point in dense.iter().take(i_n + 1).skip(i0 + 1) {
        main_chain.push(*point);
    }
    main_chain.push(bn_chain);

    let main_offset = parallel_offset(&main_chain, offset);
    let main_pts = resample_by_distance(&main_offset, base_spacing);

    // ── S-Kurven (kubische Bézier) ────────────────────────────────────────────
    let t_chain_start = (dense[1] - dense[0]).normalize_or_zero();
    let t_chain_end = (*dense
        .last()
        .expect("invariant: dense ist nach Catmull-Rom und polyline_length-Guard nicht-leer")
        - dense[dense.len() - 2])
        .normalize_or_zero();

    const CP: f32 = 0.45;
    let cp_dist = d_blend * CP;

    // S-Entry: chain[0] → b0
    let entry_pts = sample_bezier(
        chain_positions[0],
        chain_positions[0] + t_chain_start * cp_dist,
        b0 - t_at_b0 * cp_dist,
        b0,
        half_spacing,
    );

    // S-Exit: bn → chain[n-1]
    let exit_pts = sample_bezier(
        bn,
        bn + t_at_bn * cp_dist,
        *chain_positions
            .last()
            .expect("invariant: chain_positions ist nicht-leer – Load-Chain-Invariante")
            - t_chain_end * cp_dist,
        *chain_positions
            .last()
            .expect("invariant: chain_positions ist nicht-leer – Load-Chain-Invariante"),
        half_spacing,
    );

    // ── Ergebnis zusammenstellen ──────────────────────────────────────────────
    // entry_pts[0] = chain[0] → ueberspringen (existiert)
    // main_pts[0]  = b0       → ueberspringen (bereits in entry_pts)
    // exit_pts[0]  = bn       → ueberspringen (bereits in main_pts)
    // exit_pts[last] = chain[n-1] → ueberspringen (existiert)
    let entry_new: Vec<Vec2> = entry_pts.iter().skip(1).copied().collect();
    let main_new: Vec<Vec2> = main_pts.iter().skip(1).copied().collect();
    let exit_new: Vec<Vec2> = exit_pts
        .iter()
        .skip(1)
        .take(exit_pts.len().saturating_sub(2))
        .copied()
        .collect();

    let positions: Vec<Vec2> = entry_new
        .iter()
        .chain(main_new.iter())
        .chain(exit_new.iter())
        .copied()
        .collect();

    if positions.is_empty() {
        return None;
    }

    Some((positions, d_blend))
}

// ─── private Hilfsfunktionen ──────────────────────────────────────────────────

/// Findet `(index, t)` sodass `poly[index].lerp(poly[index+1], t)` bei Arc-Distanz
/// `target` liegt. `None` wenn `target` die Gesamtlaenge uebersteigt.
fn arc_split(poly: &[Vec2], target: f32) -> Option<(usize, f32)> {
    let mut acc = 0.0_f32;
    for i in 0..poly.len().saturating_sub(1) {
        let seg = poly[i].distance(poly[i + 1]);
        if acc + seg >= target {
            let t = if seg > f32::EPSILON {
                (target - acc) / seg
            } else {
                0.0
            };
            return Some((i, t.clamp(0.0, 1.0)));
        }
        acc += seg;
    }
    None
}

/// Gibt einen Punkt auf einer kubischen Bézier-Kurve bei Parameter `t ∈ [0,1]` zurueck.
fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

/// Berechnet Punkte auf einer kubischen Bézier-Kurve mit gegebenem maximalen Abstand.
fn sample_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, step: f32) -> Vec<Vec2> {
    const DENSE: usize = 64;
    let dense: Vec<Vec2> = (0..=DENSE)
        .map(|i| cubic_bezier(p0, p1, p2, p3, i as f32 / DENSE as f32))
        .collect();
    resample_by_distance(&dense, step)
}
