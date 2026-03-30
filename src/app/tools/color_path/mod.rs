//! Farb-Pfad-Tool: erkennt Wege anhand der Farbe im Hintergrundbild.

mod config_ui;
mod lifecycle;
pub(crate) mod sampling;
mod state;

pub use state::ColorPathTool;
