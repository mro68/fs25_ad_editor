//! Egui-Frontend fuer den FS25 AutoDrive Editor.

pub use fs25_auto_drive_engine::{app, core, shared, xml};

/// Eframe-/egui-Integrationsschale fuer den laufenden Editor.
pub mod editor_app;
/// Duenne egui-spezifische Adapter-Helfer fuer die gemeinsame Host-Bridge.
pub mod host_bridge_adapter;
/// GPU-Rendering mit wgpu fuer das egui-Frontend.
pub mod render;
mod runtime;
/// UI-Komponenten: Menue, Properties, Input-Handling und Dialoge.
pub mod ui;

pub use runtime::run_native;
