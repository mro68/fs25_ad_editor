//! Flutter-Adapter-/Kompat-Seams fuer den FS25 AutoDrive Editor.

/// Kompat-DTOs als Alias-Namen ueber der kanonischen Host-Bridge.
pub mod dto;
/// Kompat-Session-Alias ueber der kanonischen Host-Bridge.
pub mod session;

pub use dto::{
    EngineActiveTool, EngineDialogRequest, EngineDialogRequestKind, EngineDialogResult,
    EngineSelectionSnapshot, EngineSessionAction, EngineSessionSnapshot, EngineViewportSnapshot,
};
pub use session::{EngineRenderFrameSnapshot, FlutterBridgeSession};
