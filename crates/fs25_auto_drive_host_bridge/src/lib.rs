//! Toolkit-freie Host-Bridge-Core-Crate fuer den FS25 AutoDrive Editor.

pub use fs25_auto_drive_engine::app::ui_contract::{HostUiSnapshot, ViewportOverlaySnapshot};

/// Wiederverwendbare Dispatch-Seam fuer Rust-Hosts ueber `AppController`.
pub mod dispatch;
/// Serialisierbare Session-, Action- und Dialog-DTOs der Host-Bridge.
pub mod dto;
/// Kanonische Session-Fassade ueber `AppController` und `AppState`.
pub mod session;

pub use dispatch::{
    apply_host_action, apply_host_action_with_viewport_input_state, apply_mapped_intent,
    apply_viewport_input_batch, build_host_chrome_snapshot, build_host_ui_snapshot,
    build_render_assets, build_render_frame, build_render_scene,
    build_route_tool_viewport_snapshot, build_viewport_geometry_snapshot,
    build_viewport_overlay_snapshot, map_host_action_to_intent, map_intent_to_host_action,
    take_host_dialog_requests, HostViewportInputState,
};
pub use dto::{
    EngineActiveTool, EngineChromeSnapshot, EngineDefaultConnectionDirection,
    EngineDefaultConnectionPriority, EngineDialogRequest, EngineDialogRequestKind,
    EngineDialogResult, EngineInputModifiers, EnginePointerButton, EngineRouteToolAction,
    EngineRouteToolDisabledReason, EngineRouteToolEntrySnapshot, EngineRouteToolGroup,
    EngineRouteToolIconKey, EngineRouteToolId, EngineRouteToolSelectionSnapshot,
    EngineRouteToolSurface, EngineRouteToolViewportSnapshot, EngineSelectionSnapshot,
    EngineSessionAction, EngineSessionSnapshot, EngineTangentMenuSnapshot,
    EngineTangentOptionSnapshot, EngineTangentSource, EngineTapKind,
    EngineViewportGeometrySnapshot, EngineViewportInputBatch, EngineViewportInputEvent,
    EngineViewportSnapshot, HostActiveTool, HostChromeSnapshot, HostDefaultConnectionDirection,
    HostDefaultConnectionPriority, HostDialogRequest, HostDialogRequestKind, HostDialogResult,
    HostInputModifiers, HostPointerButton, HostRouteToolAction, HostRouteToolDisabledReason,
    HostRouteToolEntrySnapshot, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId,
    HostRouteToolSelectionSnapshot, HostRouteToolSurface, HostRouteToolViewportSnapshot,
    HostSelectionSnapshot, HostSessionAction, HostSessionSnapshot, HostTangentMenuSnapshot,
    HostTangentOptionSnapshot, HostTangentSource, HostTapKind, HostViewportConnectionDirection,
    HostViewportConnectionPriority, HostViewportConnectionSnapshot, HostViewportGeometrySnapshot,
    HostViewportInputBatch, HostViewportInputEvent, HostViewportMarkerSnapshot,
    HostViewportNodeKind, HostViewportNodeSnapshot, HostViewportSnapshot,
};
pub use session::{
    EngineRenderFrameSnapshot, FlutterBridgeSession, HostBridgeSession, HostDialogUiState,
    HostLocalDialogState, HostPanelPropertiesState, HostRenderFrameSnapshot,
    HostViewportInputContext,
};

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::{AppController, AppState};

    use super::{
        apply_host_action, EngineSessionAction, FlutterBridgeSession, HostSessionAction,
        HostViewportInputState,
    };

    #[test]
    fn crate_root_reexports_keep_dispatch_and_session_surface_stable() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::ToggleCommandPalette,
        )
        .expect("crate-root apply_host_action muss ueber Re-Export verfuegbar bleiben");

        assert!(handled);
        assert!(state.ui.show_command_palette);

        let mut session = FlutterBridgeSession::new();
        session
            .apply_action(EngineSessionAction::ToggleCommandPalette)
            .expect("FlutterBridgeSession-Alias muss ueber den Crate-Root funktionieren");
        assert!(session.snapshot().show_command_palette);

        let _ = std::any::type_name::<HostViewportInputState>();
    }
}
