//! Ausweichstrecken-Tool — parallele Strecke zu einer selektierten Kette.
//!
//! Registriert als `RouteTool`. Wird mit der aktuellen Selektion initialisiert
//! wenn das Tool über den ToolManager aktiviert wird.

mod config_ui;
mod geometry;
mod lifecycle;
mod state;

pub use state::BypassTool;
