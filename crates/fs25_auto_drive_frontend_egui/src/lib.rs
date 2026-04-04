//! Egui-Frontend fuer den FS25 AutoDrive Editor.

pub use fs25_auto_drive_engine::{app, core, shared, xml};

/// GPU-Rendering mit wgpu fuer das egui-Frontend.
pub mod render;
