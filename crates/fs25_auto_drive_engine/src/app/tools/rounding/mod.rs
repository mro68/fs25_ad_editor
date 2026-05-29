//! Arc-only-Verrundungs-Tool mit `max_angle_deg`-basierter Segmentierung fuer lokale Corner-Replace-Pfade.

mod config_ui;
mod geometry;
mod lifecycle;
mod state;

#[cfg(test)]
mod tests;

pub use state::RoundingTool;
