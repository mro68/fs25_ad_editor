//! Handler fuer Node/Connection-Editing, Marker und Editor-Werkzeug.

#[path = "editing/clipboard_ops.rs"]
mod clipboard_ops;
#[path = "editing/connection_ops.rs"]
mod connection_ops;
#[path = "editing/group_ops.rs"]
mod group_ops;
#[path = "editing/marker_ops.rs"]
mod marker_ops;
#[path = "editing/node_ops.rs"]
mod node_ops;

pub use clipboard_ops::{
    cancel_paste_preview, confirm_paste, copy_selection, export_curseplay_file,
    import_curseplay_file, start_paste_preview, update_paste_preview,
};
pub use connection_ops::{
    add_connection, connect_selected, invert_all_between_selected, remove_all_between_selected,
    remove_connection_between, set_all_directions_between_selected,
    set_all_priorities_between_selected, set_connection_direction, set_connection_priority,
    set_default_direction, set_default_priority,
};
pub use group_ops::edit_group;
pub use marker_ops::{create_marker, open_marker_dialog, remove_marker, update_marker};
pub use node_ops::{
    add_node, connect_tool_pick, delete_selected, resample_path, set_editor_tool, set_node_flag,
    streckenteilung_aktivieren, trace_all_fields,
};
