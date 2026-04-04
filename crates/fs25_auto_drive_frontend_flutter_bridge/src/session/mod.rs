//! Flutter-seitige Kompat-Surface ueber der kanonischen Host-Bridge.
//!
//! Dieses Modul enthaelt bewusst keine eigene Session-Logik mehr.
//! Bestehende Namen bleiben als Alias auf `fs25_auto_drive_host_bridge` erhalten.

pub use fs25_auto_drive_host_bridge::{
    HostBridgeSession as FlutterBridgeSession,
    HostRenderFrameSnapshot as EngineRenderFrameSnapshot,
};

#[cfg(test)]
mod tests {
    use crate::dto::EngineSessionAction;

    use super::FlutterBridgeSession;

    #[test]
    fn compatibility_alias_exposes_host_bridge_session_behavior() {
        let mut session = FlutterBridgeSession::new();

        session
            .apply_action(EngineSessionAction::ToggleCommandPalette)
            .expect("ToggleCommandPalette muss funktionieren");

        assert!(session.snapshot().show_command_palette);
    }
}
