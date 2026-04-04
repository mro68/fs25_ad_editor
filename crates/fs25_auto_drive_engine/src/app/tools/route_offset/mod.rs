//! Strecken-Versatz-Tool — paralleler Versatz einer selektierten Kette nach links und/oder rechts.
//!
//! Registriert als `RouteTool`. Wird mit der aktuellen Selektion initialisiert
//! wenn das Tool ueber den ToolManager aktiviert wird.

mod config_ui;
mod geometry;
mod lifecycle;
mod state;
#[cfg(test)]
mod tests;

pub use geometry::compute_offset_positions;
pub use state::RouteOffsetTool;
