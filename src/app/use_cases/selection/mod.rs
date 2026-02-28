//! Use-Case-Funktionen für Node-Selektion.
//!
//! Aufgeteilt nach Selektionsmodus:
//! - `pick` — Einzelklick-Selektion (Nearest-Node)
//! - `segment` — Doppelklick-Selektion (Korridor zwischen Kreuzungen)
//! - `rect` — Rechteck-Selektion (Shift + Drag)
//! - `lasso` — Lasso-Selektion (Alt + Drag)
//! - `move_nodes` — Verschieben selektierter Nodes
//! - `helpers` — Gemeinsame Hilfsfunktionen

///
/// Aufgeteilt nach Selektionsmodus:
/// - `pick` — Einzelklick-Selektion (Nearest-Node)
/// - `segment` — Doppelklick-Selektion (Korridor zwischen Kreuzungen)
/// - `rect` — Rechteck-Selektion (Shift + Drag)
/// - `lasso` — Lasso-Selektion (Shift + Alt + Drag)
/// - `move_nodes` — Verschieben selektierter Nodes
/// - `helpers` — Gemeinsame Hilfsfunktionen
mod helpers;
mod lasso;
mod move_nodes;
mod pick;
mod rect;
mod segment;

pub use helpers::clear_selection;
pub use lasso::select_nodes_in_lasso;
pub use move_nodes::move_selected_nodes;
pub use pick::context_menu_select;
pub use pick::select_nearest_node;
pub use rect::select_nodes_in_rect;
pub use segment::select_segment_between_nearest_intersections;
