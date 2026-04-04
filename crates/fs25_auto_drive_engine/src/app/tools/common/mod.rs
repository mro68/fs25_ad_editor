//! Gemeinsame Hilfsfunktionen fuer Route-Tools.
//!
//! Aufgeteilt in:
//! - `geometry`  — rein-mathematische Funktionen ohne egui
//! - `tangent`   — TangentState
//! - `lifecycle` — ToolLifecycleState, SegmentConfig, LastEdited
//! - `builder`   — assemble_tool_result
//! - `result`    — kanonische ToolResult-Defaults fuer einfache Faelle

mod builder;
mod geometry;
mod lifecycle;
mod result;
mod tangent;

pub(crate) use builder::assemble_tool_result;
// angle_to_compass und local_perp werden in Teilmodulen/Tests verwendet
#[allow(unused_imports)]
pub(crate) use geometry::angle_to_compass;
#[allow(unused_imports)]
pub(crate) use geometry::local_perp;
pub(crate) use geometry::{
    linear_connections, parallel_offset, populate_neighbors, tangent_options,
};
pub(crate) use lifecycle::{
    record_applied_tool_state, sync_tool_host, SegmentConfig, ToolLifecycleState,
};
pub(crate) use result::ToolResultBuilder;
pub(crate) use tangent::TangentState;
