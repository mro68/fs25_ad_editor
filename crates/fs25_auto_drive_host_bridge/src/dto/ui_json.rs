//! JSON-Transport-Helfer fuer host-neutrale UI- und Overlay-Snapshots.

use fs25_auto_drive_engine::app::tools::ToolPreview;
use fs25_auto_drive_engine::app::ui_contract::{
    ClipboardOverlaySnapshot, ClipboardPreviewNode, ColorPathPanelPhase, ColorPathPanelState,
    ColorPathPreviewStats, CurvePanelState, CurveTangentsPanelState, FieldBoundaryPanelState,
    FieldPathPanelPhase, FieldPathPanelState, FieldPathPreviewStatus, FieldPathSelectionSummary,
    GroupBoundaryOverlaySnapshot, GroupLockOverlaySnapshot, HostUiSnapshot, PanelState,
    ParkingPanelState, PolylineOverlaySnapshot, RouteOffsetPanelState, RouteToolConfigState,
    RouteToolPanelState, SegmentConfigPanelState, SegmentLengthKind, SegmentPanelMode,
    SmoothCurvePanelState, SmoothCurveSteererState, SplinePanelState, TangentHelpHint,
    TangentNoneReason, TangentSelectionState, ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::{BoundaryDirection, ConnectionDirection, ConnectionPriority};
use fs25_auto_drive_engine::shared::I18nKey;
use glam::Vec2;
use serde_json::{json, Value};

/// Serialisiert einen host-neutralen UI-Snapshot als UTF-8-JSON.
pub fn host_ui_snapshot_json(snapshot: &HostUiSnapshot) -> serde_json::Result<String> {
    serde_json::to_string(&host_ui_snapshot_to_value(snapshot))
}

/// Serialisiert einen host-neutralen Viewport-Overlay-Snapshot als UTF-8-JSON.
pub fn viewport_overlay_snapshot_json(
    snapshot: &ViewportOverlaySnapshot,
) -> serde_json::Result<String> {
    serde_json::to_string(&viewport_overlay_snapshot_to_value(snapshot))
}

fn host_ui_snapshot_to_value(snapshot: &HostUiSnapshot) -> Value {
    json!({
        "panels": snapshot
            .panels
            .iter()
            .map(panel_state_to_value)
            .collect::<Vec<_>>()
    })
}

fn panel_state_to_value(panel: &PanelState) -> Value {
    match panel {
        PanelState::RouteTool(state) => json!({
            "kind": "route_tool",
            "state": route_tool_panel_state_to_value(state),
        }),
        PanelState::Options(state) => json!({
            "kind": "options",
            "state": {
                "visible": state.visible,
                "options": state.options.as_ref(),
            },
        }),
        PanelState::CommandPalette(state) => json!({
            "kind": "command_palette",
            "state": {
                "visible": state.visible,
            },
        }),
    }
}

fn route_tool_panel_state_to_value(state: &RouteToolPanelState) -> Value {
    json!({
        "active_tool_id": state.active_tool_id,
        "status_text": state.status_text,
        "has_pending_input": state.has_pending_input,
        "can_execute": state.can_execute,
        "config_state": state.config_state.as_ref().map(route_tool_config_state_to_value),
    })
}

fn route_tool_config_state_to_value(state: &RouteToolConfigState) -> Value {
    match state {
        RouteToolConfigState::Straight(state) => json!({
            "kind": "straight",
            "segment": segment_config_panel_state_to_value(&state.segment),
        }),
        RouteToolConfigState::Curve(state) => curve_panel_state_to_value(state),
        RouteToolConfigState::Spline(state) => spline_panel_state_to_value(state),
        RouteToolConfigState::SmoothCurve(state) => smooth_curve_panel_state_to_value(state),
        RouteToolConfigState::Bypass(state) => json!({
            "kind": "bypass",
            "has_chain": state.has_chain,
            "offset": state.offset,
            "base_spacing": state.base_spacing,
            "new_node_count": state.new_node_count,
            "chain_node_count": state.chain_node_count,
            "transition_length_m": state.transition_length_m,
        }),
        RouteToolConfigState::Parking(state) => parking_panel_state_to_value(state),
        RouteToolConfigState::FieldBoundary(state) => field_boundary_panel_state_to_value(state),
        RouteToolConfigState::FieldPath(state) => field_path_panel_state_to_value(state),
        RouteToolConfigState::RouteOffset(state) => route_offset_panel_state_to_value(state),
        RouteToolConfigState::ColorPath(state) => color_path_panel_state_to_value(state),
    }
}

fn curve_panel_state_to_value(state: &CurvePanelState) -> Value {
    json!({
        "kind": "curve",
        "degree": state.degree,
        "tangents": state.tangents.as_ref().map(curve_tangents_panel_state_to_value),
        "segment": segment_config_panel_state_to_value(&state.segment),
    })
}

fn curve_tangents_panel_state_to_value(state: &CurveTangentsPanelState) -> Value {
    json!({
        "help_hint": state.help_hint.map(tangent_help_hint_to_str),
        "start": tangent_selection_state_to_value(&state.start),
        "end": tangent_selection_state_to_value(&state.end),
    })
}

fn spline_panel_state_to_value(state: &SplinePanelState) -> Value {
    json!({
        "kind": "spline",
        "control_point_count": state.control_point_count,
        "start_tangent": state.start_tangent.as_ref().map(tangent_selection_state_to_value),
        "end_tangent": state.end_tangent.as_ref().map(tangent_selection_state_to_value),
        "segment": segment_config_panel_state_to_value(&state.segment),
    })
}

fn smooth_curve_panel_state_to_value(state: &SmoothCurvePanelState) -> Value {
    json!({
        "kind": "smooth_curve",
        "max_angle_deg": state.max_angle_deg,
        "segment": segment_config_panel_state_to_value(&state.segment),
        "min_distance": state.min_distance,
        "approach_steerer": state
            .approach_steerer
            .as_ref()
            .map(smooth_curve_steerer_state_to_value),
        "departure_steerer": state
            .departure_steerer
            .as_ref()
            .map(smooth_curve_steerer_state_to_value),
        "control_nodes": state
            .control_nodes
            .iter()
            .copied()
            .map(vec2_to_array)
            .collect::<Vec<_>>(),
        "preview_node_count": state.preview_node_count,
    })
}

fn smooth_curve_steerer_state_to_value(state: &SmoothCurveSteererState) -> Value {
    json!({
        "position": vec2_to_array(state.position),
        "is_manual": state.is_manual,
    })
}

fn parking_panel_state_to_value(state: &ParkingPanelState) -> Value {
    json!({
        "kind": "parking",
        "num_rows": state.num_rows,
        "row_spacing": state.row_spacing,
        "bay_length": state.bay_length,
        "max_node_distance": state.max_node_distance,
        "entry_t": state.entry_t,
        "exit_t": state.exit_t,
        "ramp_length": state.ramp_length,
        "entry_side": state.entry_side,
        "exit_side": state.exit_side,
        "marker_group": state.marker_group,
        "rotation_step_deg": state.rotation_step_deg,
        "angle_deg": state.angle_deg,
        "hint_text": state.hint_text.map(i18n_key_to_string),
    })
}

fn field_boundary_panel_state_to_value(state: &FieldBoundaryPanelState) -> Value {
    json!({
        "kind": "field_boundary",
        "selected_field_id": state.selected_field_id,
        "node_spacing": state.node_spacing,
        "offset": state.offset,
        "straighten_tolerance": state.straighten_tolerance,
        "corner_detection_enabled": state.corner_detection_enabled,
        "corner_angle_threshold_deg": state.corner_angle_threshold_deg,
        "corner_rounding_enabled": state.corner_rounding_enabled,
        "corner_rounding_radius": state.corner_rounding_radius,
        "corner_rounding_max_angle_deg": state.corner_rounding_max_angle_deg,
        "direction": connection_direction_to_str(state.direction),
        "priority": connection_priority_to_str(state.priority),
        "show_select_hint": state.show_select_hint,
    })
}

fn field_path_panel_state_to_value(state: &FieldPathPanelState) -> Value {
    json!({
        "kind": "field_path",
        "mode": state.mode,
        "phase": field_path_panel_phase_to_str(state.phase),
        "side1": field_path_selection_summary_to_value(&state.side1),
        "side2": state.side2.as_ref().map(field_path_selection_summary_to_value),
        "can_advance_to_side2": state.can_advance_to_side2,
        "can_compute": state.can_compute,
        "preview_status": state.preview_status.map(field_path_preview_status_to_value),
        "node_spacing": state.node_spacing,
        "simplify_tolerance": state.simplify_tolerance,
        "connect_to_existing": state.connect_to_existing,
    })
}

fn field_path_selection_summary_to_value(state: &FieldPathSelectionSummary) -> Value {
    json!({
        "title": i18n_key_to_string(state.title),
        "text": state.text,
        "empty_hint": state.empty_hint.map(i18n_key_to_string),
        "is_empty": state.is_empty,
    })
}

fn route_offset_panel_state_to_value(state: &RouteOffsetPanelState) -> Value {
    json!({
        "kind": "route_offset",
        "has_chain": state.has_chain,
        "left_enabled": state.left_enabled,
        "left_distance": state.left_distance,
        "right_enabled": state.right_enabled,
        "right_distance": state.right_distance,
        "base_spacing": state.base_spacing,
        "keep_original": state.keep_original,
        "chain_node_count": state.chain_node_count,
    })
}

fn color_path_panel_state_to_value(state: &ColorPathPanelState) -> Value {
    // CP-05: Die Legacy-Flags can_next/can_back sind `#[deprecated]`, werden
    // aber bis CP-06 von der Engine noch befuellt; der DTO-Snapshot reicht sie
    // unveraendert an Hosts weiter, damit aeltere Adapter nicht brechen.
    #[allow(deprecated)]
    let (can_next, can_back) = (state.can_next, state.can_back);
    json!({
        "kind": "color_path",
        "phase": color_path_panel_phase_to_str(state.phase),
        "sample_count": state.sample_count,
        "avg_color": state.avg_color,
        "palette_colors": state.palette_colors,
        "can_compute": state.can_compute,
        "can_next": can_next,
        "can_back": can_back,
        "can_accept": state.can_accept,
        "preview_stats": state.preview_stats.map(color_path_preview_stats_to_value),
        "exact_color_match": state.exact_color_match,
        "color_tolerance": state.color_tolerance,
        "node_spacing": state.node_spacing,
        "simplify_tolerance": state.simplify_tolerance,
        "junction_radius": state.junction_radius,
        "noise_filter": state.noise_filter,
        "existing_connection_mode": state.existing_connection_mode,
    })
}

fn color_path_preview_stats_to_value(state: ColorPathPreviewStats) -> Value {
    json!({
        "junction_count": state.junction_count,
        "open_end_count": state.open_end_count,
        "segment_count": state.segment_count,
        "node_count": state.node_count,
        "can_accept": state.can_accept,
    })
}

fn segment_config_panel_state_to_value(state: &SegmentConfigPanelState) -> Value {
    json!({
        "mode": segment_panel_mode_to_str(state.mode),
        "length_kind": segment_length_kind_to_str(state.length_kind),
        "length_m": state.length_m,
        "max_segment_length": state.max_segment_length,
        "max_segment_length_min": state.max_segment_length_min,
        "max_segment_length_max": state.max_segment_length_max,
        "node_count": state.node_count,
        "node_count_min": state.node_count_min,
        "node_count_max": state.node_count_max,
    })
}

fn tangent_selection_state_to_value(state: &TangentSelectionState) -> Value {
    json!({
        "none_reason": tangent_none_reason_to_str(state.none_reason),
        "current": state.current,
        "options": state
            .options
            .iter()
            .map(|option| {
                json!({
                    "source": option.source,
                    "label": option.label,
                })
            })
            .collect::<Vec<_>>(),
        "enabled": state.enabled,
    })
}

fn viewport_overlay_snapshot_to_value(snapshot: &ViewportOverlaySnapshot) -> Value {
    json!({
        "route_tool_preview": snapshot
            .route_tool_preview
            .as_ref()
            .map(tool_preview_to_value),
        "clipboard_preview": snapshot
            .clipboard_preview
            .as_ref()
            .map(clipboard_overlay_snapshot_to_value),
        "distance_preview": snapshot
            .distance_preview
            .as_ref()
            .map(polyline_overlay_snapshot_to_value),
        "group_locks": snapshot
            .group_locks
            .iter()
            .copied()
            .map(group_lock_overlay_snapshot_to_value)
            .collect::<Vec<_>>(),
        "group_boundaries": snapshot
            .group_boundaries
            .iter()
            .copied()
            .map(group_boundary_overlay_snapshot_to_value)
            .collect::<Vec<_>>(),
        "show_no_file_hint": snapshot.show_no_file_hint,
    })
}

fn tool_preview_to_value(preview: &ToolPreview) -> Value {
    json!({
        "nodes": preview
            .nodes
            .iter()
            .copied()
            .map(vec2_to_array)
            .collect::<Vec<_>>(),
        "connections": preview
            .connections
            .iter()
            .enumerate()
            .map(|(index, (start_index, end_index))| {
                let (direction, priority) = preview
                    .connection_styles
                    .get(index)
                    .copied()
                    .unwrap_or((ConnectionDirection::Regular, ConnectionPriority::Regular));
                json!({
                    "start_index": start_index,
                    "end_index": end_index,
                    "direction": connection_direction_to_str(direction),
                    "priority": connection_priority_to_str(priority),
                })
            })
            .collect::<Vec<_>>(),
        "labels": preview
            .labels
            .iter()
            .map(|(node_index, text)| {
                json!({
                    "node_index": node_index,
                    "text": text,
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn clipboard_overlay_snapshot_to_value(snapshot: &ClipboardOverlaySnapshot) -> Value {
    json!({
        "nodes": snapshot
            .nodes
            .iter()
            .copied()
            .map(clipboard_preview_node_to_value)
            .collect::<Vec<_>>(),
        "connections": snapshot
            .connections
            .iter()
            .map(|(start_index, end_index)| {
                json!({
                    "start_index": start_index,
                    "end_index": end_index,
                })
            })
            .collect::<Vec<_>>(),
        "opacity": snapshot.opacity,
    })
}

fn clipboard_preview_node_to_value(node: ClipboardPreviewNode) -> Value {
    json!({
        "world_pos": vec2_to_array(node.world_pos),
        "has_marker": node.has_marker,
    })
}

fn polyline_overlay_snapshot_to_value(snapshot: &PolylineOverlaySnapshot) -> Value {
    json!({
        "points": snapshot
            .points
            .iter()
            .copied()
            .map(vec2_to_array)
            .collect::<Vec<_>>(),
    })
}

fn group_lock_overlay_snapshot_to_value(snapshot: GroupLockOverlaySnapshot) -> Value {
    json!({
        "segment_id": snapshot.segment_id,
        "world_pos": vec2_to_array(snapshot.world_pos),
        "locked": snapshot.locked,
    })
}

fn group_boundary_overlay_snapshot_to_value(snapshot: GroupBoundaryOverlaySnapshot) -> Value {
    json!({
        "segment_id": snapshot.segment_id,
        "node_id": snapshot.node_id,
        "world_pos": vec2_to_array(snapshot.world_pos),
        "direction": boundary_direction_to_str(snapshot.direction),
    })
}

fn vec2_to_array(value: Vec2) -> [f32; 2] {
    [value.x, value.y]
}

fn tangent_help_hint_to_str(value: TangentHelpHint) -> &'static str {
    match value {
        TangentHelpHint::SetStartEnd => "set_start_end",
    }
}

fn tangent_none_reason_to_str(value: TangentNoneReason) -> &'static str {
    match value {
        TangentNoneReason::NoConnection => "no_connection",
        TangentNoneReason::NoTangent => "no_tangent",
        TangentNoneReason::UseDefault => "use_default",
    }
}

fn segment_panel_mode_to_str(value: SegmentPanelMode) -> &'static str {
    match value {
        SegmentPanelMode::Default => "default",
        SegmentPanelMode::Ready => "ready",
        SegmentPanelMode::Adjusting => "adjusting",
    }
}

fn segment_length_kind_to_str(value: SegmentLengthKind) -> &'static str {
    match value {
        SegmentLengthKind::StraightLine => "straight_line",
        SegmentLengthKind::Curve => "curve",
        SegmentLengthKind::CatmullRomSpline => "catmull_rom_spline",
        SegmentLengthKind::SmoothRoute => "smooth_route",
    }
}

fn field_path_panel_phase_to_str(value: FieldPathPanelPhase) -> &'static str {
    match value {
        FieldPathPanelPhase::Idle => "idle",
        FieldPathPanelPhase::SelectingSide1 => "selecting_side1",
        FieldPathPanelPhase::SelectingSide2 => "selecting_side2",
        FieldPathPanelPhase::Preview => "preview",
    }
}

fn field_path_preview_status_to_value(value: FieldPathPreviewStatus) -> Value {
    match value {
        FieldPathPreviewStatus::NoMiddleLine => json!({
            "kind": "no_middle_line",
        }),
        FieldPathPreviewStatus::Generated { node_count } => json!({
            "kind": "generated",
            "node_count": node_count,
        }),
    }
}

fn color_path_panel_phase_to_str(value: ColorPathPanelPhase) -> &'static str {
    // CP-05 (Single-Step-Wizard): Die Engine emittiert kanonisch nur noch
    // `"idle"`/`"sampling"`/`"editing"`. Die Legacy-Wizard-Phasen
    // `Preview`/`CenterlinePreview`/`JunctionEdit`/`Finalize` werden auf
    // `"editing"` gefaltet, damit bestehende Hosts/Snapshots weiter
    // konsistent gelesen werden koennen. CP-11 entfernt die Legacy-Varianten.
    #[allow(deprecated)]
    match value {
        ColorPathPanelPhase::Idle => "idle",
        ColorPathPanelPhase::Sampling => "sampling",
        ColorPathPanelPhase::Editing
        | ColorPathPanelPhase::CenterlinePreview
        | ColorPathPanelPhase::JunctionEdit
        | ColorPathPanelPhase::Finalize
        | ColorPathPanelPhase::Preview => "editing",
    }
}

/// Liefert die kanonische [`ColorPathPanelPhase`] fuer einen DTO-Phase-String.
///
/// Akzeptiert kanonisch `"idle"`, `"sampling"`, `"editing"` sowie zusaetzlich
/// die Legacy-Strings `"preview"`, `"centerline_preview"`, `"junction_edit"`
/// und `"finalize"` aus dem alten Wizard-Modell und mappt sie auf
/// [`ColorPathPanelPhase::Editing`].
#[allow(dead_code)] // CP-06 nutzt den Helfer fuer Host-Eingangs-Deserialisierung.
pub(crate) fn color_path_panel_phase_from_str(value: &str) -> Option<ColorPathPanelPhase> {
    match value {
        "idle" => Some(ColorPathPanelPhase::Idle),
        "sampling" => Some(ColorPathPanelPhase::Sampling),
        "editing" | "preview" | "centerline_preview" | "junction_edit" | "finalize" => {
            Some(ColorPathPanelPhase::Editing)
        }
        _ => None,
    }
}

fn boundary_direction_to_str(value: BoundaryDirection) -> &'static str {
    match value {
        BoundaryDirection::Entry => "entry",
        BoundaryDirection::Exit => "exit",
        BoundaryDirection::Bidirectional => "bidirectional",
    }
}

fn connection_direction_to_str(value: ConnectionDirection) -> &'static str {
    match value {
        ConnectionDirection::Regular => "regular",
        ConnectionDirection::Dual => "dual",
        ConnectionDirection::Reverse => "reverse",
    }
}

fn connection_priority_to_str(value: ConnectionPriority) -> &'static str {
    match value {
        ConnectionPriority::Regular => "regular",
        ConnectionPriority::SubPriority => "sub_priority",
    }
}

fn i18n_key_to_string(value: I18nKey) -> String {
    debug_name_to_snake_case(&format!("{value:?}"))
}

fn debug_name_to_snake_case(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut result = String::with_capacity(value.len() + 8);

    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_uppercase() {
            let prev = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
            let next = chars.get(index + 1).copied();
            let needs_separator = index > 0
                && (prev.is_some_and(char::is_lowercase) || next.is_some_and(char::is_lowercase));
            if needs_separator {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fs25_auto_drive_engine::app::tool_contract::RouteToolId;
    use fs25_auto_drive_engine::app::tools::ToolPreview;
    use fs25_auto_drive_engine::app::ui_contract::{
        ClipboardOverlaySnapshot, ClipboardPreviewNode, CommandPalettePanelState,
        GroupBoundaryOverlaySnapshot, GroupLockOverlaySnapshot, HostUiSnapshot, OptionsPanelState,
        PanelState, PolylineOverlaySnapshot, RouteToolConfigState, RouteToolPanelState,
        SegmentConfigPanelState, SegmentLengthKind, SegmentPanelMode, StraightPanelState,
        ViewportOverlaySnapshot,
    };
    use fs25_auto_drive_engine::app::BoundaryDirection;
    use fs25_auto_drive_engine::core::{ConnectionDirection, ConnectionPriority};
    use fs25_auto_drive_engine::shared::EditorOptions;
    use glam::Vec2;
    use serde_json::Value;

    use super::{host_ui_snapshot_json, viewport_overlay_snapshot_json};

    #[test]
    fn host_ui_snapshot_json_serializes_route_tool_panels() {
        let snapshot = HostUiSnapshot {
            panels: vec![
                PanelState::RouteTool(Box::new(RouteToolPanelState {
                    active_tool_id: Some(RouteToolId::Straight),
                    status_text: Some("Bereit".to_string()),
                    has_pending_input: true,
                    can_execute: false,
                    config_state: Some(RouteToolConfigState::Straight(StraightPanelState {
                        segment: SegmentConfigPanelState {
                            mode: SegmentPanelMode::Default,
                            length_kind: SegmentLengthKind::StraightLine,
                            length_m: None,
                            max_segment_length: 5.0,
                            max_segment_length_min: 1.0,
                            max_segment_length_max: 10.0,
                            node_count: None,
                            node_count_min: None,
                            node_count_max: None,
                        },
                    })),
                })),
                PanelState::Options(OptionsPanelState {
                    visible: true,
                    options: Arc::new(EditorOptions::default()),
                }),
                PanelState::CommandPalette(CommandPalettePanelState { visible: false }),
            ],
        };

        let payload = host_ui_snapshot_json(&snapshot)
            .expect("HostUiSnapshot muss als JSON serialisierbar sein");
        let value: Value =
            serde_json::from_str(&payload).expect("HostUiSnapshot-JSON muss parsebar sein");

        assert_eq!(value["panels"][0]["kind"], "route_tool");
        assert_eq!(value["panels"][0]["state"]["active_tool_id"], "straight");
        assert_eq!(
            value["panels"][0]["state"]["config_state"]["kind"],
            "straight"
        );
        assert_eq!(value["panels"][1]["kind"], "options");
        assert_eq!(value["panels"][2]["kind"], "command_palette");
    }

    #[test]
    fn viewport_overlay_snapshot_json_serializes_preview_and_boundaries() {
        let snapshot = ViewportOverlaySnapshot {
            route_tool_preview: Some(ToolPreview {
                nodes: vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)],
                connections: vec![(0, 1)],
                connection_styles: vec![(
                    ConnectionDirection::Dual,
                    ConnectionPriority::SubPriority,
                )],
                labels: vec![(0, "A".to_string())],
            }),
            clipboard_preview: Some(ClipboardOverlaySnapshot {
                nodes: vec![ClipboardPreviewNode {
                    world_pos: Vec2::new(5.0, 6.0),
                    has_marker: true,
                }],
                connections: vec![(0, 0)],
                opacity: 0.35,
            }),
            distance_preview: Some(PolylineOverlaySnapshot {
                points: vec![Vec2::new(7.0, 8.0), Vec2::new(9.0, 10.0)],
            }),
            group_locks: vec![GroupLockOverlaySnapshot {
                segment_id: 21,
                world_pos: Vec2::new(11.0, 12.0),
                locked: true,
            }],
            group_boundaries: vec![GroupBoundaryOverlaySnapshot {
                segment_id: 22,
                node_id: 23,
                world_pos: Vec2::new(13.0, 14.0),
                direction: BoundaryDirection::Exit,
            }],
            show_no_file_hint: true,
        };

        let payload = viewport_overlay_snapshot_json(&snapshot)
            .expect("ViewportOverlaySnapshot muss als JSON serialisierbar sein");
        let value: Value = serde_json::from_str(&payload)
            .expect("ViewportOverlaySnapshot-JSON muss parsebar sein");

        assert_eq!(
            value["route_tool_preview"]["connections"][0]["direction"],
            "dual"
        );
        assert_eq!(
            value["route_tool_preview"]["connections"][0]["priority"],
            "sub_priority"
        );
        assert_eq!(value["group_boundaries"][0]["direction"], "exit");
        assert_eq!(value["show_no_file_hint"], true);
    }

    // ------------------------------------------------------------------
    // CP-05 — DTO-Tests fuer kanonisches Single-Step-Editing.
    // ------------------------------------------------------------------

    use fs25_auto_drive_engine::app::ui_contract::{
        ColorPathPanelAction, ColorPathPanelPhase, ColorPathPanelState,
        ExistingConnectionModeChoice,
    };

    use super::color_path_panel_phase_from_str;

    fn make_color_path_state(phase: ColorPathPanelPhase) -> ColorPathPanelState {
        #[allow(deprecated)] // can_next/can_back sind Legacy-Felder bis CP-06.
        ColorPathPanelState {
            phase,
            sample_count: 0,
            avg_color: None,
            palette_colors: Vec::new(),
            can_compute: false,
            can_next: false,
            can_back: false,
            can_accept: false,
            preview_stats: None,
            exact_color_match: false,
            color_tolerance: 1.0,
            node_spacing: 1.0,
            simplify_tolerance: 0.0,
            junction_radius: 0.0,
            noise_filter: false,
            existing_connection_mode: ExistingConnectionModeChoice::Never,
        }
    }

    #[test]
    fn color_path_panel_phase_emits_editing_for_canonical_phase() {
        let value = super::color_path_panel_state_to_value(&make_color_path_state(
            ColorPathPanelPhase::Editing,
        ));
        assert_eq!(value["phase"], "editing");
    }

    #[test]
    fn color_path_panel_phase_emits_editing_for_legacy_wizard_phases() {
        // Bis CP-06 emittiert die Engine noch `Finalize`/`CenterlinePreview`/...
        // Der DTO-Layer muss alle Editing-Stufen auf den kanonischen
        // `"editing"`-String falten.
        #[allow(deprecated)]
        let legacy = [
            ColorPathPanelPhase::Preview,
            ColorPathPanelPhase::CenterlinePreview,
            ColorPathPanelPhase::JunctionEdit,
            ColorPathPanelPhase::Finalize,
        ];
        for phase in legacy {
            let value = super::color_path_panel_state_to_value(&make_color_path_state(phase));
            assert_eq!(
                value["phase"], "editing",
                "Legacy-Phase {phase:?} muss auf 'editing' gefaltet werden",
            );
        }
    }

    #[test]
    fn color_path_panel_phase_emits_idle_and_sampling_unchanged() {
        for (phase, expected) in [
            (ColorPathPanelPhase::Idle, "idle"),
            (ColorPathPanelPhase::Sampling, "sampling"),
        ] {
            let value = super::color_path_panel_state_to_value(&make_color_path_state(phase));
            assert_eq!(value["phase"], expected);
        }
    }

    #[test]
    fn color_path_panel_phase_from_str_accepts_canonical_and_legacy() {
        assert_eq!(
            color_path_panel_phase_from_str("idle"),
            Some(ColorPathPanelPhase::Idle)
        );
        assert_eq!(
            color_path_panel_phase_from_str("sampling"),
            Some(ColorPathPanelPhase::Sampling)
        );
        assert_eq!(
            color_path_panel_phase_from_str("editing"),
            Some(ColorPathPanelPhase::Editing)
        );
        for legacy in ["preview", "centerline_preview", "junction_edit", "finalize"] {
            assert_eq!(
                color_path_panel_phase_from_str(legacy),
                Some(ColorPathPanelPhase::Editing),
                "Legacy-String {legacy:?} muss auf Editing gemappt werden",
            );
        }
        assert_eq!(color_path_panel_phase_from_str("garbage"), None);
    }

    #[test]
    fn color_path_panel_action_compute_roundtrips_canonical_tag() {
        let json = serde_json::to_string(&ColorPathPanelAction::Compute)
            .expect("Compute muss serialisierbar sein");
        let value: Value = serde_json::from_str(&json).expect("JSON muss parsebar sein");
        assert_eq!(value["kind"], "compute");

        let parsed: ColorPathPanelAction =
            serde_json::from_str(&json).expect("Roundtrip-Deserialisierung muss gelingen");
        assert_eq!(parsed, ColorPathPanelAction::Compute);
    }

    #[test]
    fn color_path_panel_action_legacy_tags_still_deserialize() {
        // CP-05: Legacy-Tags bleiben deserialisierbar. Engine-Mapping auf
        // Compute/Reset erfolgt in CP-06; hier wird nur der Wire-Kontrakt
        // verifiziert.
        #[allow(deprecated)]
        let cases: &[(&str, ColorPathPanelAction)] = &[
            ("compute_preview", ColorPathPanelAction::ComputePreview),
            ("back_to_sampling", ColorPathPanelAction::BackToSampling),
            ("next_phase", ColorPathPanelAction::NextPhase),
            ("prev_phase", ColorPathPanelAction::PrevPhase),
        ];
        for (tag, expected) in cases {
            let json = format!(r#"{{"kind":"{tag}"}}"#);
            let parsed: ColorPathPanelAction = serde_json::from_str(&json)
                .unwrap_or_else(|err| panic!("Legacy-Tag {tag:?} muss parsebar bleiben: {err}"));
            assert_eq!(parsed, *expected);
        }
    }
}
