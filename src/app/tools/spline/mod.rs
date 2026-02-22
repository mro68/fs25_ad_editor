//! Catmull-Rom-Spline-Tool — interpolierende Kurve durch alle geklickten Punkte.
//!
//! Aufgeteilt in:
//! - `state`     — Struct, Konstruktor, geometrische Hilfsmethoden
//! - `lifecycle` — RouteTool-Implementierung (on_click, preview, execute, reset, …)
//! - `geometry`  — Catmull-Rom-Geometrie-Funktionen
//! - `config_ui` — UI-Konfigurationspanel

mod config_ui;
pub(crate) mod geometry;
mod lifecycle;
mod state;

pub use state::SplineTool;

#[cfg(test)]
mod tests;
