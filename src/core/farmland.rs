//! Feldgrenz-Polygone in Weltkoordinaten.
//!
//! Enthalten geordnete Umriss-Vertices eines Farmland-Feldes, umgerechnet
//! aus den GRLE-Pixel-Koordinaten in das Weltkoordinatensystem des Editors.

use glam::Vec2;

/// Ein geordnetes Feldgrenz-Polygon in Weltkoordinaten (x/z-Ebene).
///
/// Die Vertices stammen aus dem GRLE-Farmland-Raster und wurden
/// per `world = pixel * (map_size / grle_width) - map_size / 2`
/// in Weltkoordinaten umgerechnet.
pub struct FieldPolygon {
    /// Farmland-ID (1–255, 0 = kein Feld)
    pub id: u32,
    /// Geordnete Rand-Vertices in Weltkoordinaten (x, z)
    pub vertices: Vec<Vec2>,
}
