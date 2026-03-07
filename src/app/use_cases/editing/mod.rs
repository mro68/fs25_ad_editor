//! Use-Case-Funktionen fuer Node/Connection-Editing.
//!
//! Aufgeteilt nach Operation:
//! - `add_node` — Neuen Node hinzufuegen (inkl. optionalem Connection-Split)
//! - `delete_nodes` — Selektierte Nodes loeschen (inkl. optionalem Reconnect)
//! - `connect` — Verbindungen erstellen
//! - `disconnect` — Verbindungen entfernen
//! - `direction` — Verbindungsrichtung aendern
//! - `priority` — Verbindungsprioritaet aendern
//! - `node_flag` — Node-Flag gezielt setzen
//! - `bulk_connections` — Bulk-Aenderungen an Verbindungen
//! - `markers` — Map-Marker-Operationen
//! - `resample_path` — Nodes-Kette per Catmull-Rom-Spline neu verteilen (Distanzen)
//! - `copy_paste` — Kopieren/Einfuegen von Nodes, Verbindungen und Markern

///
/// Aufgeteilt nach Operation:
/// - `add_node` — Neuen Node hinzufuegen
/// - `delete_nodes` — Selektierte Nodes loeschen
/// - `connect` — Verbindungen erstellen (inkl. Connect-Tool-Flow)
/// - `disconnect` — Verbindungen entfernen
/// - `direction` — Verbindungsrichtung aendern
mod add_node;
mod apply_tool_result;
mod bulk_connections;
mod connect;
mod copy_paste;
mod delete_nodes;
mod delete_nodes_by_ids;
mod direction;
mod disconnect;
mod markers;
mod node_flag;
mod priority;
mod resample_path;
mod trace_all_fields;

pub use add_node::add_node_at_position;
pub use add_node::AddNodeResult;
pub use apply_tool_result::apply_tool_result;
pub use apply_tool_result::apply_tool_result_no_snapshot;
pub use bulk_connections::{
    invert_all_connections_between_selected, remove_all_connections_between_selected,
    set_all_connections_direction_between_selected, set_all_connections_priority_between_selected,
};
pub use connect::{add_connection, connect_tool_pick_node};
pub use copy_paste::{
    cancel_paste_preview, confirm_paste, copy_selected_to_clipboard, start_paste_preview,
    update_paste_preview,
};
pub use delete_nodes::delete_selected_nodes;
pub use delete_nodes_by_ids::delete_nodes_by_ids;
pub use direction::set_connection_direction;
pub use disconnect::remove_connection_between;
pub use markers::{create_marker, open_marker_dialog, remove_marker, update_marker};
pub use node_flag::set_node_flag;
pub use priority::set_connection_priority;
pub use resample_path::resample_selected_path;
pub use trace_all_fields::trace_all_fields;
