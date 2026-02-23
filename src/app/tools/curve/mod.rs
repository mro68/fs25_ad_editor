//! Kurven-Tool: Zeichnet Bézier-Kurven 2. oder 3. Grades.
//!
//! **Quadratisch (Grad 2):** Start → End → 1 Steuerpunkt (Klick) → Drag-Anpassung → Enter
//! **Kubisch (Grad 3):** Start → End → automatische Tangenten-Vorschlag, CP1/CP2 per
//! Drag verschieben, Apex B(0.5) draggbar — Enter bestätigt.
//!
//! **Tangenten-Auswahl:** Rechtsklick im Viewport öffnet Kontextmenü mit
//! Start-/End-Tangenten-Auswahl (als NW/SO-Kompass-Richtung). Nach Auswahl
//! wird CP1/CP2 automatisch auf chord/3-Abstand entlang der Tangente gesetzt.
//!
//! **CP-Constraint:** Wenn eine Tangente fixiert ist, gleitet das jeweilige CP-Handle
//! nur entlang der Tangenten-Linie (ermöglicht auch S-Kurven hinter dem Anker).
//!
//! **Asymmetrisches Biegen:** Wenn beide Tangenten fixiert sind, löst der Apex-Drag
//! ein 2×2-Gleichungssystem — die Kurve biegt sich asymmetrisch unter Einhaltung
//! beider Tangenten-Richtungen.
//!
//! Grad wird über das Dropdown in der Toolbar umgeschaltet.
//!
//! Aufgeteilt in:
//! - `state`      — Structs, Enums, Konstruktor, Hilfsmethoden
//! - `lifecycle`  — RouteTool-Implementierung (on_click, preview, execute, reset, …)
//! - `drag`       — Drag-Logik (drag_targets, on_drag_start/update/end)
//! - `config_ui`  — Properties-Panel + Tangenten-Kontextmenü
//! - `geometry`   — Bézier-Berechnungen, Arc-Length-Parametrisierung, Tangenten-Solver

mod config_ui;
pub(crate) mod drag;
pub(crate) mod geometry;
mod lifecycle;
mod state;

pub use state::{CurveDegree, CurveTool};

#[cfg(test)]
mod tests;
