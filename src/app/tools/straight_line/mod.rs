//! Gerade-Strecke-Tool: Zeichnet eine Linie zwischen zwei Punkten
//! und füllt automatisch Zwischen-Nodes ein.
//!
//! Aufgeteilt in:
//! - `state`     — Struct, Konstruktor, Hilfsmethoden
//! - `lifecycle` — RouteTool-Implementierung
//! - `geometry`  — Geometrie-Berechnungen (compute_line_positions, build_result)
//! - `config_ui` — UI-Konfigurationspanel

mod config_ui;
pub(crate) mod geometry;
mod lifecycle;
mod state;

pub use state::StraightLineTool;

#[cfg(test)]
mod tests;
