//! Toolkit-freie Host-Bridge-Core-Crate fuer den FS25 AutoDrive Editor.

/// Wiederverwendbare Dispatch-Seam fuer Rust-Hosts ueber `AppController`.
pub mod dispatch;
/// Serialisierbare Session-, Action- und Dialog-DTOs der Host-Bridge.
pub mod dto;
/// Kanonische Session-Fassade ueber `AppController` und `AppState`.
pub mod session;

pub use dispatch::{apply_host_action, map_host_action_to_intent, take_host_dialog_requests};
pub use dto::{
    HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult,
    HostSelectionSnapshot, HostSessionAction, HostSessionSnapshot, HostViewportSnapshot,
};
pub use session::{HostBridgeSession, HostRenderFrameSnapshot};
