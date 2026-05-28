//! Zustandsbasierte Projektionsfunktionen fuer host-neutrale Snapshots.
//!
//! Diese freien Funktionen ersetzen die gleichnamigen Methoden auf `AppController`
//! und arbeiten direkt auf `&AppState`, ohne eine Controller-Instanz zu benoetigen.

use glam::Vec2;

use super::tool_editing::RouteToolEditPayload;
use super::ui_contract::{
    CommandPalettePanelState, HostUiSnapshot, OptionsPanelState, PanelState, RouteToolConfigState,
    ViewportOverlaySnapshot,
};
use super::{render_assets, render_scene, viewport_overlay, AppState};
use crate::shared::{RenderAssetsSnapshot, RenderScene};

const ROUNDING_RESTORE_EPSILON: f32 = 1e-3;
const ROUNDING_MIN_CORNER_ANGLE_RAD: f32 = 5.0_f32.to_radians();
const ROUNDING_MAX_CORNER_ANGLE_RAD: f32 = std::f32::consts::PI - 5.0_f32.to_radians();

fn restored_rounding_preview_node_count(state: &AppState, max_angle_deg: f32) -> Option<usize> {
    let road_map = state.road_map.as_deref()?;
    let session = state.active_tool_edit_session.as_ref()?;
    let RouteToolEditPayload::RoundingArc {
        first_anchor_id,
        second_anchor_id,
        corner_position,
        ..
    } = &session.tool_edit_backup.payload
    else {
        return None;
    };

    let first_position = road_map.node_position(*first_anchor_id)?;
    let second_position = road_map.node_position(*second_anchor_id)?;
    let first_vec = first_position - *corner_position;
    let second_vec = second_position - *corner_position;
    let first_len = first_vec.length();
    let second_len = second_vec.length();
    if first_len <= ROUNDING_RESTORE_EPSILON || second_len <= ROUNDING_RESTORE_EPSILON {
        return None;
    }

    let first_dir = first_vec / first_len;
    let second_dir = second_vec / second_len;
    let corner_angle = first_dir
        .perp_dot(second_dir)
        .atan2(first_dir.dot(second_dir))
        .abs();
    if !(ROUNDING_MIN_CORNER_ANGLE_RAD..=ROUNDING_MAX_CORNER_ANGLE_RAD).contains(&corner_angle) {
        return None;
    }

    let max_angle_rad = max_angle_deg.clamp(1.0, 45.0).to_radians();
    let segment_count = ((corner_angle / max_angle_rad).ceil() as usize).max(2);
    Some(segment_count + 1)
}

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

    if let Some(mut route_tool_panel) = state.editor.route_tool_panel_state() {
        if let Some(RouteToolConfigState::Rounding(rounding_state)) =
            route_tool_panel.config_state.as_mut()
            && rounding_state.is_adjusting
            && rounding_state.preview_node_count.is_none()
        {
            rounding_state.preview_node_count = restored_rounding_preview_node_count(
                state,
                rounding_state.max_angle_deg,
            )
            .or_else(|| {
                state
                    .road_map
                    .as_deref()
                    .and_then(|road_map| state.editor.route_tool_preview(Vec2::ZERO, road_map))
                    .map(|preview| preview.nodes.len().saturating_sub(2))
            });
        }

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
