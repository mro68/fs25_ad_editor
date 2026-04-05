//! Toolkit-freie Host-Bridge-Core-Crate fuer den FS25 AutoDrive Editor.

pub use fs25_auto_drive_engine::app::ui_contract::{HostUiSnapshot, ViewportOverlaySnapshot};

/// Wiederverwendbare Dispatch-Seam fuer Rust-Hosts ueber `AppController`.
pub mod dispatch;
/// Serialisierbare Session-, Action- und Dialog-DTOs der Host-Bridge.
pub mod dto;
/// Kanonische Session-Fassade ueber `AppController` und `AppState`.
pub mod session;

pub use dispatch::{
    apply_host_action, apply_mapped_intent, build_host_ui_snapshot, build_render_assets,
    build_render_scene, build_viewport_overlay_snapshot, map_host_action_to_intent,
    map_intent_to_host_action, take_host_dialog_requests,
};
pub use dto::{
    EngineActiveTool, EngineDialogRequest, EngineDialogRequestKind, EngineDialogResult,
    EngineSelectionSnapshot, EngineSessionAction, EngineSessionSnapshot, EngineViewportSnapshot,
    HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult,
    HostSelectionSnapshot, HostSessionAction, HostSessionSnapshot, HostViewportSnapshot,
};
pub use session::{
    EngineRenderFrameSnapshot, FlutterBridgeSession, HostBridgeSession, HostRenderFrameSnapshot,
};
