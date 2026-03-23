//! Feldgrenz-Erkennungs-Tool: erzeugt eine geschlossene Route entlang eines Feldumrisses.

pub mod geometry;
mod config_ui;
mod lifecycle;
mod state;

pub use geometry::RingNodeKind;
pub use lifecycle::compute_ring;
pub use state::FieldBoundaryTool;
