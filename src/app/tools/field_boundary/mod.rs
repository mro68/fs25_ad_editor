//! Feldgrenz-Erkennungs-Tool: erzeugt eine geschlossene Route entlang eines Feldumrisses.

mod config_ui;
mod lifecycle;
mod state;

pub(crate) use lifecycle::compute_ring;
pub use state::FieldBoundaryTool;
