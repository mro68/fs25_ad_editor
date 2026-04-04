//! Feldgrenz-Erkennungs-Tool: erzeugt eine geschlossene Route entlang eines Feldumrisses.

mod config_ui;
pub mod geometry;
mod lifecycle;
mod state;

pub use lifecycle::compute_ring;
pub use state::FieldBoundaryTool;
