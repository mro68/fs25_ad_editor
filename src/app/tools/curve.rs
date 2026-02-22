//! Kurven-Tool: Zeichnet Bézier-Kurven 2. oder 3. Grades.
//!
//! **Quadratisch (Grad 2):** Start → End → 1 Steuerpunkt (Klick) → Drag-Anpassung → Enter
//! **Kubisch (Grad 3):** Start → End → CP1 (Klick) → CP2 (Klick) → Drag-Anpassung → Enter
//!
//! Nach Platzierung aller Punkte können Start, End und Steuerpunkte
//! per Drag verschoben werden. Start/Ende rasten beim Loslassen auf
//! nahe existierende Nodes ein (Re-Snap).
//!
//! Grad wird über `render_config` umgeschaltet (UI-Dropdown).
//!
//! Aufgeteilt in:
//! - `state` — Structs, Enums, Konstruktor, Hilfsmethoden
//! - `lifecycle` — RouteTool-Implementierung (on_click, preview, execute, reset, …)
//! - `drag` — Drag-Logik (drag_targets, on_drag_start/update/end)
//! - `config_ui` — UI-Konfigurationspanel
//! - `geometry` — Bézier-Berechnungen und Arc-Length-Parametrisierung

mod config_ui;
pub(crate) mod drag;
pub(crate) mod geometry;
mod lifecycle;
mod state;

pub use state::{CurveDegree, CurveTool};

#[cfg(test)]
mod tests;
