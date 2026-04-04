//! Toolkit-freie Host-Bridge-Core-Crate fuer den FS25 AutoDrive Editor.

/// Serialisierbare Session-, Action- und Dialog-DTOs der Host-Bridge.
pub mod dto;
/// Kanonische Session-Fassade ueber `AppController` und `AppState`.
pub mod session;

pub use dto::{
	HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult,
	HostSelectionSnapshot, HostSessionAction, HostSessionSnapshot, HostViewportSnapshot,
};
pub use session::{HostBridgeSession, HostRenderFrameSnapshot};
