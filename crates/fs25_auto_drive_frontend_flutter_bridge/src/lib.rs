//! Flutter-Bridge-Seams fuer den FS25 AutoDrive Editor.

/// Serialisierbare DTOs fuer Host-/Frontend-Snapshots.
pub mod dto;
/// Session-Fassade ueber der Engine ohne Flutter-SDK-Kopplung.
pub mod session;

pub use dto::{
    EngineActiveTool, EngineSelectionSnapshot, EngineSessionAction, EngineSessionSnapshot,
    EngineViewportSnapshot,
};
pub use session::{EngineRenderFrameSnapshot, FlutterBridgeSession};
