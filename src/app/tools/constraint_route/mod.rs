//! Constraint-Route-Tool: Erzeugt Routen mit automatischer Winkelglättung
//! und tangentialen Übergängen zu bestehenden Verbindungen.
//!
//! Aufgeteilt in:
//! - `state`     — Struct, Phase-Enum, Konstruktor, Hilfsmethoden
//! - `lifecycle` — RouteTool-Implementierung (Klick-Phasen, Preview, Execute)
//! - `geometry`  — Solver-Logik (Steerer, Subdivision, Winkelglättung, Resampling)
//! - `config_ui` — UI-Konfigurationspanel (Max-Winkel, Segment-Länge, Kontrollpunkte)
//! - `drag`      — Drag-Logik für Kontrollpunkte und Endpunkte

mod config_ui;
mod drag;
pub(crate) mod geometry;
mod lifecycle;
mod state;

pub use state::ConstraintRouteTool;

#[cfg(test)]
mod tests;
