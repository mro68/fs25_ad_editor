//! Use-Case-Funktionen fuer Node-Selektion.
//!
//! Aufgeteilt nach Selektionsmodus:
//! - `pick` — Einzelklick-Selektion (Nearest-Node)
//! - `segment` — Doppelklick-Selektion (Korridor zwischen Kreuzungen)
//! - `rect` — Rechteck-Selektion (Shift + Drag)
//! - `lasso` — Lasso-Selektion (Alt + Drag)
//! - `move_nodes` — Verschieben selektierter Nodes
//! - `helpers` — Gemeinsame Hilfsfunktionen

mod group;
///
/// Aufgeteilt nach Selektionsmodus:
/// - `pick` — Einzelklick-Selektion (Nearest-Node)
/// - `segment` — Doppelklick-Selektion (Korridor zwischen Kreuzungen)
/// - `rect` — Rechteck-Selektion (Shift + Drag)
/// - `lasso` — Lasso-Selektion (Shift + Alt + Drag)
/// - `move_nodes` — Verschieben selektierter Nodes
/// - `rotate_nodes` — Rotation selektierter Nodes um Zentrum
/// - `helpers` — Gemeinsame Hilfsfunktionen
mod helpers;
mod lasso;
mod move_nodes;
mod pick;
mod rect;
mod rotate_nodes;
mod segment;

pub use group::select_group_by_nearest_node;
pub use helpers::clear_selection;
pub use lasso::select_nodes_in_lasso;
pub use move_nodes::move_selected_nodes;
pub use pick::select_nearest_node;
pub use rect::select_nodes_in_rect;
pub use rotate_nodes::rotate_selected_nodes;
pub use segment::select_segment_between_nearest_intersections;
