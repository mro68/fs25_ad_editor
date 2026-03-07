//! Geometrie-Berechnungen fuer das Strecken-Versatz-Tool.
//!
//! Alle Funktionen sind pur (keine State-Mutation) und arbeiten nur mit `Vec2`-Koordinaten.
//!
//! # Geometrie
//!
//! ```text
//!  chain[0] ──▶ chain[1] ──▶ ... ──▶ chain[n-1]   (Original)
//!   offset[0] ──▶ offset[1] ──▶ ... ──▶ offset[m-1]  (Versatz, parallel verschoben)
//! ```
//!
//! Im Unterschied zum BypassTool gibt es keine S-Kurven-Uebergaenge.
//! Die Offset-Kette beginnt und endet direkt an den verschobenen Endpunkten.

use crate::app::tools::common::parallel_offset;
use crate::shared::spline_geometry::resample_by_distance;
use glam::Vec2;

/// Berechnet die Positionen einer parallel versetzten Kette.
///
/// Nutzt `parallel_offset()` aus `common/geometry.rs` und resampled das Ergebnis
/// mit `base_spacing` als maximalem Abstand zwischen Nodes.
///
/// # Parameter
/// - `chain_positions` — Originalkette (mind. 2 Punkte)
/// - `offset` — Versatz-Betrag (positiv = links in Fahrtrichtung, negativ = rechts)
/// - `base_spacing` — max. Abstand zwischen Offset-Nodes nach dem Resampling
///
/// # Rueckgabe
/// `Some(Vec<Vec2>)` — neue Knoten-Positionen einschliesslich der Endpunkte
/// `None` wenn die Kette zu kurz ist oder die Geometrie degeneriert
pub fn compute_offset_positions(
    chain_positions: &[Vec2],
    offset: f32,
    base_spacing: f32,
) -> Option<Vec<Vec2>> {
    if chain_positions.len() < 2 {
        return None;
    }
    let base_spacing = base_spacing.max(0.5);
    let offset_pts = parallel_offset(chain_positions, offset);
    let resampled = resample_by_distance(&offset_pts, base_spacing);
    if resampled.len() < 2 {
        return None;
    }
    Some(resampled)
}
