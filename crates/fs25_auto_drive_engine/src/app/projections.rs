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
pub fn build_host_ui_snapshot(state: &AppState) -> HostUiSnapshot {
    let mut panels = Vec::new();

    panels.push(PanelState::CommandPalette(CommandPalettePanelState {
        visible: state.ui.show_command_palette,
    }));

    panels.push(PanelState::Options(OptionsPanelState {
        visible: state.ui.show_options_dialog,
        options: state.options.clone(),
    }));

    if let Some(route_tool_panel) = state.editor.route_tool_panel_state() {
        panels.push(PanelState::RouteTool(route_tool_panel));
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
