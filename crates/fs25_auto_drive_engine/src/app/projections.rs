//! Zustandsbasierte Projektionsfunktionen fuer host-neutrale Snapshots.
//!
//! Diese freien Funktionen ersetzen die gleichnamigen Methoden auf `AppController`
//! und arbeiten direkt auf `&AppState`, ohne eine Controller-Instanz zu benoetigen.

use glam::Vec2;

use super::ui_contract::{
    CommandPalettePanelState, HostUiSnapshot, OptionsPanelState, PanelState,
    ViewportOverlaySnapshot,
};
use super::{render_assets, render_scene, viewport_overlay, AppState};
use crate::shared::{RenderAssetsSnapshot, RenderScene};

/// Baut die Render-Szene aus dem aktuellen AppState.
pub fn build_render_scene(state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
    render_scene::build(state, viewport_size)
}

/// Baut den host-neutralen Render-Asset-Snapshot aus dem aktuellen AppState.
pub fn build_render_assets(state: &AppState) -> RenderAssetsSnapshot {
    render_assets::build(state)
}

/// Baut den host-neutralen UI-Snapshot fuer sichtbare Panels.
///
/// Datei- und Pfaddialoge sind bewusst nicht Teil dieses Snapshots und
/// laufen separat ueber `take_dialog_requests()`.
///
/// **Hinweis:** Die Sichtbarkeit von `CommandPalette` und `Options` ist hier
/// immer `false`. Die tatsaechliche Sichtbarkeit stammt aus dem
/// `HostLocalDialogState` (chrome_state) in der `HostBridgeSession` und wird
/// beim Bau des Snapshots auf Bridge-Ebene eingefuegt.
pub fn build_host_ui_snapshot(state: &AppState) -> HostUiSnapshot {
    let mut panels = Vec::new();

    panels.push(PanelState::CommandPalette(CommandPalettePanelState {
        visible: false,
    }));

    panels.push(PanelState::Options(OptionsPanelState {
        visible: false,
        options: state.options_arc(),
    }));

    if let Some(route_tool_panel) = state.editor.route_tool_panel_state() {
        panels.push(PanelState::RouteTool(Box::new(route_tool_panel)));
    }

    HostUiSnapshot { panels }
}

/// Baut den host-neutralen Overlay-Snapshot fuer den Viewport.
///
/// Die mutable State-Referenz bleibt noetig, weil beim Aufbau Caches im
/// `AppState` aufgewaermt werden koennen.
pub fn build_viewport_overlay_snapshot(
    state: &mut AppState,
    cursor_world: Option<Vec2>,
) -> ViewportOverlaySnapshot {
    viewport_overlay::build(state, cursor_world)
}

#[cfg(test)]
mod tests {
    use super::build_host_ui_snapshot;
    use crate::app::AppState;

    /// Prüft, dass build_host_ui_snapshot immer das CommandPalette-Panel enthält.
    ///
    /// Die tatsaechliche Sichtbarkeit wird auf Bridge-Ebene aus `chrome_state` eingesetzt.
    #[test]
    fn build_host_ui_snapshot_contains_command_palette_panel() {
        let state = AppState::new();

        let snapshot = build_host_ui_snapshot(&state);

        let cp_state = snapshot
            .command_palette_state()
            .expect("CommandPalette-Panel muss im Snapshot enthalten sein");
        assert!(
            !cp_state.visible,
            "engine-seitig ist visible immer false (Bridge setzt chrome_state)"
        );
    }

    /// Prüft, dass build_host_ui_snapshot die aktuellen EditorOptions korrekt
    /// in den OptionsPanelState überträgt.
    #[test]
    fn build_host_ui_snapshot_options_panel_carries_current_options() {
        let mut state = AppState::new();
        state.options.node_size_world = 77.5;
        state.options.snap_scale_percent = 33.0;
        // options_arc wird nur durch set_options()/refresh_options_arc() aktualisiert.
        state.refresh_options_arc();

        let snapshot = build_host_ui_snapshot(&state);

        let opts_panel = snapshot
            .options_panel_state()
            .expect("OptionsPanelState muss im Snapshot vorhanden sein");
        assert!(
            !opts_panel.visible,
            "engine-seitig ist visible immer false (Bridge setzt chrome_state)"
        );
        assert!(
            (opts_panel.options.node_size_world - 77.5).abs() < f32::EPSILON,
            "node_size_world muss korrekt übertragen werden"
        );
        assert!(
            (opts_panel.options.snap_scale_percent - 33.0).abs() < f32::EPSILON,
            "snap_scale_percent muss korrekt übertragen werden"
        );
    }
}
