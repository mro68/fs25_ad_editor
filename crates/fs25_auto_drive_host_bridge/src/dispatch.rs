//! Kanonische Dispatch-Seam und Snapshot-Builder der fs25_auto_drive_host_bridge.
//!
//! Intern in thematische Submodule aufgeteilt; die oeffentliche Schnittstelle ist unveraendert.

mod actions;
mod mappings;
mod snapshot;
mod viewport_input;

pub use actions::{
    apply_host_action, apply_host_action_with_viewport_input_state, apply_mapped_intent,
};
pub(crate) use mappings::map_engine_dialog_request;
pub use mappings::{
    map_host_action_to_intent, map_intent_to_host_action, take_host_dialog_requests,
};
pub use snapshot::{
    build_host_chrome_snapshot, build_host_ui_snapshot, build_render_assets, build_render_frame,
    build_render_scene, build_route_tool_viewport_snapshot, build_viewport_geometry_snapshot,
    build_viewport_overlay_snapshot,
};
pub use viewport_input::{apply_viewport_input_batch, HostViewportInputState};

#[cfg(test)]
mod tests;
