//! Gemeinsame Hilfsfunktionen fuer Route-Tools.
//!
//! Aufgeteilt in:
//! - `geometry`  — rein-mathematische Funktionen ohne egui
//! - `tangent`   — TangentSource, TangentState, render_tangent_combo
//! - `lifecycle` — ToolLifecycleState, SegmentConfig, LastEdited
//! - `builder`   — assemble_tool_result

mod builder;
mod geometry;
mod input_helpers;
mod lifecycle;
mod tangent;

pub(crate) use builder::assemble_tool_result;
pub(crate) use input_helpers::wheel_dir;
// angle_to_compass und local_perp werden in Teilmodulen/Tests verwendet
#[allow(unused_imports)]
pub(crate) use geometry::angle_to_compass;
#[allow(unused_imports)]
pub(crate) use geometry::local_perp;
pub(crate) use geometry::{
    linear_connections, parallel_offset, populate_neighbors, tangent_options,
};
pub(crate) use lifecycle::{render_segment_config_3modes, SegmentConfig, ToolLifecycleState};
pub use tangent::TangentSource;
pub(crate) use tangent::{render_tangent_combo, TangentMenuData, TangentState};
