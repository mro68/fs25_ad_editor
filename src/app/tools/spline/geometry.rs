//! Catmull-Rom-Geometrie-Funktionen für das Spline-Tool.
//!
//! Die eigentlichen Algorithmen liegen in `crate::shared::spline_geometry`.
//! Dieses Modul re-exportiert sie für die bequeme Nutzung im Tool-Layer.

pub use crate::shared::spline_geometry::{
    catmull_rom_chain_with_tangents, polyline_length, resample_by_distance,
};

/// Kompatibilität: Standard-Catmull-Rom-Chain ohne Tangent-Override (nur für Tests).
#[cfg(test)]
pub fn catmull_rom_chain(points: &[glam::Vec2], samples_per_segment: usize) -> Vec<glam::Vec2> {
    catmull_rom_chain_with_tangents(points, samples_per_segment, None, None)
}
