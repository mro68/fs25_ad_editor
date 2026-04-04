//! Geglättete-Kurve-Tool: Erzeugt Routen mit automatischer Winkelglaettung
//! und tangentialen Uebergaengen zu bestehenden Verbindungen.
//!
//! Aufgeteilt in:
//! - `state`     — Struct, Phase-Enum, Konstruktor, Hilfsmethoden
//! - `lifecycle` — RouteTool-Implementierung (Klick-Phasen, Preview, Execute)
//! - `geometry`  — Solver-Logik (Steerer, Subdivision, Winkelglaettung, Resampling)
//! - `config_ui` — UI-Konfigurationspanel (Max-Winkel, Segment-Laenge, Kontrollpunkte)
//! - `drag`      — Drag-Logik fuer Kontrollpunkte und Endpunkte

mod config_ui;
mod drag;
pub(crate) mod geometry;
mod lifecycle;
mod state;

pub use geometry::{solve_route, SmoothCurveInput};
pub use state::SmoothCurveTool;

#[cfg(test)]
mod tests;
