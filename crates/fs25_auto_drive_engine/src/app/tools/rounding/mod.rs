//! Verrundungs-Tool mit internem Modusrahmen fuer Arc- und Quadratic-Pfade.

mod config_ui;
mod geometry;
mod lifecycle;
mod state;

#[cfg(test)]
mod tests;

pub use state::{RoundingMode, RoundingTool};
