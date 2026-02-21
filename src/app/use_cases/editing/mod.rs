//! Use-Case-Funktionen für Node/Connection-Editing.
//!
//! Aufgeteilt nach Operation:
//! - `add_node` — Neuen Node hinzufügen
//! - `delete_nodes` — Selektierte Nodes löschen
//! - `connect` — Verbindungen erstellen
//! - `disconnect` — Verbindungen entfernen
//! - `direction` — Verbindungsrichtung ändern
//! - `priority` — Verbindungspriorität ändern
//! - `bulk_connections` — Bulk-Änderungen an Verbindungen
//! - `markers` — Map-Marker-Operationen

///
/// Aufgeteilt nach Operation:
/// - `add_node` — Neuen Node hinzufügen
/// - `delete_nodes` — Selektierte Nodes löschen
/// - `connect` — Verbindungen erstellen (inkl. Connect-Tool-Flow)
/// - `disconnect` — Verbindungen entfernen
/// - `direction` — Verbindungsrichtung ändern
mod add_node;
mod apply_tool_result;
mod bulk_connections;
mod connect;
mod delete_nodes;
mod delete_nodes_by_ids;
mod direction;
mod disconnect;
mod markers;
mod priority;

pub use add_node::add_node_at_position;
pub use apply_tool_result::apply_tool_result;
pub use apply_tool_result::apply_tool_result_no_snapshot;
pub use bulk_connections::{
    invert_all_connections_between_selected, remove_all_connections_between_selected,
    set_all_connections_direction_between_selected, set_all_connections_priority_between_selected,
};
pub use connect::{add_connection, connect_tool_pick_node};
pub use delete_nodes::delete_selected_nodes;
pub use delete_nodes_by_ids::delete_nodes_by_ids;
pub use direction::set_connection_direction;
pub use disconnect::remove_connection_between;
pub use markers::{create_marker, open_marker_dialog, remove_marker, update_marker};
pub use priority::set_connection_priority;
