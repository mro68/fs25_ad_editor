//! Kompatibilitaets-DTOs fuer bestehende Flutter-nahe Rust-Call-Sites.
//!
//! Die kanonischen Vertrage leben in `fs25_auto_drive_host_bridge`.
//! Dieses Modul bietet nur stabile Alias-Namen ohne eigene Logik.

pub use fs25_auto_drive_host_bridge::{
    HostActiveTool as EngineActiveTool, HostDialogRequest as EngineDialogRequest,
    HostDialogRequestKind as EngineDialogRequestKind, HostDialogResult as EngineDialogResult,
    HostSelectionSnapshot as EngineSelectionSnapshot, HostSessionAction as EngineSessionAction,
    HostSessionSnapshot as EngineSessionSnapshot, HostViewportSnapshot as EngineViewportSnapshot,
};
