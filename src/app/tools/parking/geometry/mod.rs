//! Geometrie-Generatoren fuer Parkplatz-Layouts.
//!
//! Aufgeteilt in drei Submodule:
//! - `layout`: Basis-Parkplatz-Layout (`generate_parking_layout`)
//! - `blueprint`: Blueprint-Serien-Layout (`generate_blueprint_series_layout`)
//! - `conversion`: Konvertierung zu ToolResult/ToolPreview

mod blueprint;
mod conversion;
mod layout;

pub(super) use conversion::{build_parking_result, build_preview};
pub use layout::generate_parking_layout;

use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

use super::state::RampSide;

/// Richtungsvorzeichen der Y-Komponente fuer eine Rampen-Seite.
fn side_sign_y(side: RampSide) -> f32 {
    match side {
        RampSide::Right => -1.0,
        RampSide::Left => 1.0,
    }
}

/// Internes Ergebnis des Generators vor ToolResult-Konvertierung.
pub(super) struct ParkingLayout {
    /// Positionen aller Nodes in Weltkoordinaten.
    pub nodes: Vec<Vec2>,
    /// (from_idx, to_idx, direction, priority)
    pub connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)>,
    /// (node_idx, marker_name, marker_group)
    pub markers: Vec<(usize, String, String)>,
}
