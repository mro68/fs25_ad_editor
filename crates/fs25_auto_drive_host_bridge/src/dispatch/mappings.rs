//! Konvertierungsfunktionen zwischen Engine-Typen und Host-Bridge-DTOs.

use fs25_auto_drive_engine::app::tool_contract::{RouteToolId, TangentSource};
use fs25_auto_drive_engine::app::tools::{
    resolve_route_tool_entries, RouteToolAvailabilityContext, RouteToolDisabledReason,
    RouteToolGroup, RouteToolIconKey, RouteToolSurface,
};
use fs25_auto_drive_engine::app::ui_contract::{
    DialogRequest, DialogRequestKind, DialogResult, TangentMenuData, TangentOptionData,
};
use fs25_auto_drive_engine::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority};
use fs25_auto_drive_engine::shared::{
    RenderConnectionDirection, RenderConnectionPriority, RenderNodeKind,
};
use glam::Vec2;

use crate::dto::{
    HostActiveTool, HostDefaultConnectionDirection, HostDefaultConnectionPriority,
    HostDialogRequest, HostDialogRequestKind, HostDialogResult, HostRouteToolAction,
    HostRouteToolDisabledReason, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId,
    HostRouteToolSurface, HostSessionAction, HostTangentMenuSnapshot, HostTangentOptionSnapshot,
    HostTangentSource, HostViewportConnectionDirection, HostViewportConnectionPriority,
    HostViewportNodeKind,
};
use fs25_auto_drive_engine::app::EditorTool;

// ──────────────────────────────── Tool / Editor ──────────────────────────────

pub(super) fn map_host_active_tool(tool: HostActiveTool) -> EditorTool {
    match tool {
        HostActiveTool::Select => EditorTool::Select,
        HostActiveTool::Connect => EditorTool::Connect,
        HostActiveTool::AddNode => EditorTool::AddNode,
        HostActiveTool::Route => EditorTool::Route,
    }
}

pub(super) fn map_editor_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
    }
}

// ────────────────────────────── Connection ───────────────────────────────────

pub(super) fn map_connection_direction(
    direction: ConnectionDirection,
) -> HostDefaultConnectionDirection {
    match direction {
        ConnectionDirection::Regular => HostDefaultConnectionDirection::Regular,
        ConnectionDirection::Dual => HostDefaultConnectionDirection::Dual,
        ConnectionDirection::Reverse => HostDefaultConnectionDirection::Reverse,
    }
}

pub(super) fn map_host_connection_direction(
    direction: HostDefaultConnectionDirection,
) -> ConnectionDirection {
    match direction {
        HostDefaultConnectionDirection::Regular => ConnectionDirection::Regular,
        HostDefaultConnectionDirection::Dual => ConnectionDirection::Dual,
        HostDefaultConnectionDirection::Reverse => ConnectionDirection::Reverse,
    }
}

pub(super) fn map_connection_priority(
    priority: ConnectionPriority,
) -> HostDefaultConnectionPriority {
    match priority {
        ConnectionPriority::Regular => HostDefaultConnectionPriority::Regular,
        ConnectionPriority::SubPriority => HostDefaultConnectionPriority::SubPriority,
    }
}

pub(super) fn map_host_connection_priority(
    priority: HostDefaultConnectionPriority,
) -> ConnectionPriority {
    match priority {
        HostDefaultConnectionPriority::Regular => ConnectionPriority::Regular,
        HostDefaultConnectionPriority::SubPriority => ConnectionPriority::SubPriority,
    }
}

// ───────────────────────────── Render types ──────────────────────────────────

pub(super) fn map_render_node_kind(kind: RenderNodeKind) -> HostViewportNodeKind {
    match kind {
        RenderNodeKind::Regular => HostViewportNodeKind::Regular,
        RenderNodeKind::SubPrio => HostViewportNodeKind::SubPrio,
        RenderNodeKind::Warning => HostViewportNodeKind::Warning,
    }
}

pub(super) fn map_render_connection_direction(
    direction: RenderConnectionDirection,
) -> HostViewportConnectionDirection {
    match direction {
        RenderConnectionDirection::Regular => HostViewportConnectionDirection::Regular,
        RenderConnectionDirection::Dual => HostViewportConnectionDirection::Dual,
        RenderConnectionDirection::Reverse => HostViewportConnectionDirection::Reverse,
    }
}

pub(super) fn map_render_connection_priority(
    priority: RenderConnectionPriority,
) -> HostViewportConnectionPriority {
    match priority {
        RenderConnectionPriority::Regular => HostViewportConnectionPriority::Regular,
        RenderConnectionPriority::SubPriority => HostViewportConnectionPriority::SubPriority,
    }
}

// ─────────────────────────────── Route-Tool ──────────────────────────────────

pub(super) fn map_route_tool_id(tool_id: RouteToolId) -> HostRouteToolId {
    match tool_id {
        RouteToolId::Straight => HostRouteToolId::Straight,
        RouteToolId::CurveQuad => HostRouteToolId::CurveQuad,
        RouteToolId::CurveCubic => HostRouteToolId::CurveCubic,
        RouteToolId::Spline => HostRouteToolId::Spline,
        RouteToolId::Bypass => HostRouteToolId::Bypass,
        RouteToolId::SmoothCurve => HostRouteToolId::SmoothCurve,
        RouteToolId::Parking => HostRouteToolId::Parking,
        RouteToolId::FieldBoundary => HostRouteToolId::FieldBoundary,
        RouteToolId::FieldPath => HostRouteToolId::FieldPath,
        RouteToolId::RouteOffset => HostRouteToolId::RouteOffset,
        RouteToolId::ColorPath => HostRouteToolId::ColorPath,
    }
}

pub(super) fn map_host_route_tool_id(tool_id: HostRouteToolId) -> RouteToolId {
    match tool_id {
        HostRouteToolId::Straight => RouteToolId::Straight,
        HostRouteToolId::CurveQuad => RouteToolId::CurveQuad,
        HostRouteToolId::CurveCubic => RouteToolId::CurveCubic,
        HostRouteToolId::Spline => RouteToolId::Spline,
        HostRouteToolId::Bypass => RouteToolId::Bypass,
        HostRouteToolId::SmoothCurve => RouteToolId::SmoothCurve,
        HostRouteToolId::Parking => RouteToolId::Parking,
        HostRouteToolId::FieldBoundary => RouteToolId::FieldBoundary,
        HostRouteToolId::FieldPath => RouteToolId::FieldPath,
        HostRouteToolId::RouteOffset => RouteToolId::RouteOffset,
        HostRouteToolId::ColorPath => RouteToolId::ColorPath,
    }
}

pub(super) fn map_tangent_source(source: TangentSource) -> HostTangentSource {
    match source {
        TangentSource::None => HostTangentSource::None,
        TangentSource::Connection { neighbor_id, angle } => {
            HostTangentSource::Connection { neighbor_id, angle }
        }
    }
}

pub(super) fn map_host_tangent_source(source: HostTangentSource) -> TangentSource {
    match source {
        HostTangentSource::None => TangentSource::None,
        HostTangentSource::Connection { neighbor_id, angle } => {
            TangentSource::Connection { neighbor_id, angle }
        }
    }
}

pub(super) fn map_route_tool_action_to_intent(action: HostRouteToolAction) -> AppIntent {
    match action {
        HostRouteToolAction::SelectTool { tool } => AppIntent::SelectRouteToolRequested {
            tool_id: map_host_route_tool_id(tool),
        },
        HostRouteToolAction::SelectToolWithAnchors {
            tool,
            start_node_id,
            end_node_id,
        } => AppIntent::RouteToolWithAnchorsRequested {
            tool_id: map_host_route_tool_id(tool),
            start_node_id,
            end_node_id,
        },
        HostRouteToolAction::PanelAction { action } => {
            AppIntent::RouteToolPanelActionRequested { action }
        }
        HostRouteToolAction::Execute => AppIntent::RouteToolExecuteRequested,
        HostRouteToolAction::Cancel => AppIntent::RouteToolCancelled,
        HostRouteToolAction::Recreate => AppIntent::RouteToolRecreateRequested,
        HostRouteToolAction::ApplyTangent { start, end } => AppIntent::RouteToolTangentSelected {
            start: map_host_tangent_source(start),
            end: map_host_tangent_source(end),
        },
        HostRouteToolAction::Click { world_pos, ctrl } => AppIntent::RouteToolClicked {
            world_pos: Vec2::new(world_pos[0], world_pos[1]),
            ctrl,
        },
        HostRouteToolAction::LassoCompleted { polygon } => AppIntent::RouteToolLassoCompleted {
            polygon: polygon
                .into_iter()
                .map(|point| Vec2::new(point[0], point[1]))
                .collect(),
        },
        HostRouteToolAction::DragStart { world_pos } => AppIntent::RouteToolDragStarted {
            world_pos: Vec2::new(world_pos[0], world_pos[1]),
        },
        HostRouteToolAction::DragUpdate { world_pos } => AppIntent::RouteToolDragUpdated {
            world_pos: Vec2::new(world_pos[0], world_pos[1]),
        },
        HostRouteToolAction::DragEnd => AppIntent::RouteToolDragEnded,
        HostRouteToolAction::ScrollRotate { delta } => AppIntent::RouteToolScrollRotated { delta },
        HostRouteToolAction::IncreaseNodeCount => AppIntent::IncreaseRouteToolNodeCount,
        HostRouteToolAction::DecreaseNodeCount => AppIntent::DecreaseRouteToolNodeCount,
        HostRouteToolAction::IncreaseSegmentLength => AppIntent::IncreaseRouteToolSegmentLength,
        HostRouteToolAction::DecreaseSegmentLength => AppIntent::DecreaseRouteToolSegmentLength,
    }
}

pub(super) fn map_route_tool_group(group: RouteToolGroup) -> HostRouteToolGroup {
    match group {
        RouteToolGroup::Basics => HostRouteToolGroup::Basics,
        RouteToolGroup::Section => HostRouteToolGroup::Section,
        RouteToolGroup::Analysis => HostRouteToolGroup::Analysis,
    }
}

pub(super) fn map_route_tool_surface(surface: RouteToolSurface) -> HostRouteToolSurface {
    match surface {
        RouteToolSurface::FloatingMenu => HostRouteToolSurface::FloatingMenu,
        RouteToolSurface::DefaultsPanel => HostRouteToolSurface::DefaultsPanel,
        RouteToolSurface::MainMenu => HostRouteToolSurface::MainMenu,
        RouteToolSurface::CommandPalette => HostRouteToolSurface::CommandPalette,
    }
}

pub(super) fn map_route_tool_icon_key(icon_key: RouteToolIconKey) -> HostRouteToolIconKey {
    match icon_key {
        RouteToolIconKey::Straight => HostRouteToolIconKey::Straight,
        RouteToolIconKey::CurveQuad => HostRouteToolIconKey::CurveQuad,
        RouteToolIconKey::CurveCubic => HostRouteToolIconKey::CurveCubic,
        RouteToolIconKey::Spline => HostRouteToolIconKey::Spline,
        RouteToolIconKey::Bypass => HostRouteToolIconKey::Bypass,
        RouteToolIconKey::SmoothCurve => HostRouteToolIconKey::SmoothCurve,
        RouteToolIconKey::Parking => HostRouteToolIconKey::Parking,
        RouteToolIconKey::FieldBoundary => HostRouteToolIconKey::FieldBoundary,
        RouteToolIconKey::FieldPath => HostRouteToolIconKey::FieldPath,
        RouteToolIconKey::RouteOffset => HostRouteToolIconKey::RouteOffset,
        RouteToolIconKey::ColorPath => HostRouteToolIconKey::ColorPath,
    }
}

pub(super) fn map_route_tool_disabled_reason(
    reason: RouteToolDisabledReason,
) -> HostRouteToolDisabledReason {
    match reason {
        RouteToolDisabledReason::MissingFarmland => HostRouteToolDisabledReason::MissingFarmland,
        RouteToolDisabledReason::MissingBackground => {
            HostRouteToolDisabledReason::MissingBackground
        }
        RouteToolDisabledReason::MissingOrderedChain => {
            HostRouteToolDisabledReason::MissingOrderedChain
        }
    }
}

pub(super) fn route_tool_availability_context(state: &AppState) -> RouteToolAvailabilityContext {
    let has_farmland = state
        .farmland_polygons_arc()
        .is_some_and(|polygons| !polygons.is_empty());
    let has_background = state.has_background_image();
    let has_ordered_chain = state.road_map.as_deref().is_some_and(|road_map| {
        road_map
            .ordered_chain_nodes(&state.selection.selected_node_ids)
            .is_some()
    });

    RouteToolAvailabilityContext {
        has_farmland,
        has_background,
        has_ordered_chain,
    }
}

pub(super) fn map_tangent_option_data(option: TangentOptionData) -> HostTangentOptionSnapshot {
    HostTangentOptionSnapshot {
        source: map_tangent_source(option.source),
        label: option.label,
    }
}

pub(super) fn map_tangent_menu_data(menu: TangentMenuData) -> HostTangentMenuSnapshot {
    HostTangentMenuSnapshot {
        start_options: menu
            .start_options
            .into_iter()
            .map(map_tangent_option_data)
            .collect(),
        end_options: menu
            .end_options
            .into_iter()
            .map(map_tangent_option_data)
            .collect(),
        current_start: map_tangent_source(menu.current_start),
        current_end: map_tangent_source(menu.current_end),
    }
}

// ─────────────────────────────── Dialog ──────────────────────────────────────

pub(super) fn map_engine_dialog_request_kind(kind: DialogRequestKind) -> HostDialogRequestKind {
    match kind {
        DialogRequestKind::OpenFile => HostDialogRequestKind::OpenFile,
        DialogRequestKind::SaveFile => HostDialogRequestKind::SaveFile,
        DialogRequestKind::Heightmap => HostDialogRequestKind::Heightmap,
        DialogRequestKind::BackgroundMap => HostDialogRequestKind::BackgroundMap,
        DialogRequestKind::OverviewZip => HostDialogRequestKind::OverviewZip,
        DialogRequestKind::CurseplayImport => HostDialogRequestKind::CurseplayImport,
        DialogRequestKind::CurseplayExport => HostDialogRequestKind::CurseplayExport,
    }
}

pub(crate) fn map_engine_dialog_request(request: DialogRequest) -> HostDialogRequest {
    let DialogRequest::PickPath {
        kind,
        suggested_file_name,
    } = request
    else {
        unreachable!("map_engine_dialog_request darf nur fuer PickPath-Anfragen aufgerufen werden")
    };
    HostDialogRequest {
        kind: map_engine_dialog_request_kind(kind),
        suggested_file_name,
    }
}

pub(super) fn map_host_dialog_request_kind(kind: HostDialogRequestKind) -> DialogRequestKind {
    match kind {
        HostDialogRequestKind::OpenFile => DialogRequestKind::OpenFile,
        HostDialogRequestKind::SaveFile => DialogRequestKind::SaveFile,
        HostDialogRequestKind::Heightmap => DialogRequestKind::Heightmap,
        HostDialogRequestKind::BackgroundMap => DialogRequestKind::BackgroundMap,
        HostDialogRequestKind::OverviewZip => DialogRequestKind::OverviewZip,
        HostDialogRequestKind::CurseplayImport => DialogRequestKind::CurseplayImport,
        HostDialogRequestKind::CurseplayExport => DialogRequestKind::CurseplayExport,
    }
}

pub(super) fn map_dialog_result(result: HostDialogResult) -> DialogResult {
    match result {
        HostDialogResult::Cancelled { kind } => DialogResult::Cancelled {
            kind: map_host_dialog_request_kind(kind),
        },
        HostDialogResult::PathSelected { kind, path } => DialogResult::PathSelected {
            kind: map_host_dialog_request_kind(kind),
            path,
        },
    }
}

// ───────────────────────────── Public mappings ───────────────────────────────

/// Mappt einen stabilen Engine-Intent auf eine explizite Host-Action.
///
/// Rueckgabewert `None` bedeutet, dass der Intent nicht zur stabilen,
/// niederfrequenten Host-Action-Surface gehoert.
pub fn map_intent_to_host_action(intent: &AppIntent) -> Option<HostSessionAction> {
    match intent {
        AppIntent::OpenFileRequested => Some(HostSessionAction::OpenFile),
        AppIntent::SaveRequested => Some(HostSessionAction::Save),
        AppIntent::SaveAsRequested => Some(HostSessionAction::SaveAs),
        AppIntent::HeightmapSelectionRequested => {
            Some(HostSessionAction::RequestHeightmapSelection)
        }
        AppIntent::BackgroundMapSelectionRequested => {
            Some(HostSessionAction::RequestBackgroundMapSelection)
        }
        AppIntent::GenerateOverviewRequested => Some(HostSessionAction::GenerateOverview),
        AppIntent::CurseplayImportRequested => Some(HostSessionAction::CurseplayImport),
        AppIntent::CurseplayExportRequested => Some(HostSessionAction::CurseplayExport),
        AppIntent::ResetCameraRequested => Some(HostSessionAction::ResetCamera),
        AppIntent::ZoomToFitRequested => Some(HostSessionAction::ZoomToFit),
        AppIntent::ZoomToSelectionBoundsRequested => Some(HostSessionAction::ZoomToSelectionBounds),
        AppIntent::ExitRequested => Some(HostSessionAction::Exit),
        AppIntent::CommandPaletteToggled => Some(HostSessionAction::ToggleCommandPalette),
        AppIntent::SetEditorToolRequested { tool } => Some(HostSessionAction::SetEditorTool {
            tool: map_editor_tool(*tool),
        }),
        AppIntent::SetDefaultDirectionRequested { direction } => {
            Some(HostSessionAction::SetDefaultDirection {
                direction: map_connection_direction(*direction),
            })
        }
        AppIntent::SetDefaultPriorityRequested { priority } => {
            Some(HostSessionAction::SetDefaultPriority {
                priority: map_connection_priority(*priority),
            })
        }
        AppIntent::OptionsChanged { options } => Some(HostSessionAction::ApplyOptions {
            options: options.clone(),
        }),
        AppIntent::ResetOptionsRequested => Some(HostSessionAction::ResetOptions),
        AppIntent::SelectRouteToolRequested { tool_id } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::SelectTool {
                tool: map_route_tool_id(*tool_id),
            },
        }),
        AppIntent::RouteToolWithAnchorsRequested {
            tool_id,
            start_node_id,
            end_node_id,
        } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::SelectToolWithAnchors {
                tool: map_route_tool_id(*tool_id),
                start_node_id: *start_node_id,
                end_node_id: *end_node_id,
            },
        }),
        AppIntent::RouteToolPanelActionRequested { action } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::PanelAction {
                action: action.clone(),
            },
        }),
        AppIntent::RouteToolExecuteRequested => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::Execute,
        }),
        AppIntent::RouteToolCancelled => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::Cancel,
        }),
        AppIntent::RouteToolConfigChanged | AppIntent::RouteToolRecreateRequested => {
            Some(HostSessionAction::RouteTool {
                action: HostRouteToolAction::Recreate,
            })
        }
        AppIntent::RouteToolTangentSelected { start, end } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::ApplyTangent {
                start: map_tangent_source(*start),
                end: map_tangent_source(*end),
            },
        }),
        AppIntent::RouteToolClicked { world_pos, ctrl } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::Click {
                world_pos: [world_pos.x, world_pos.y],
                ctrl: *ctrl,
            },
        }),
        AppIntent::RouteToolLassoCompleted { polygon } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::LassoCompleted {
                polygon: polygon.iter().map(|point| [point.x, point.y]).collect(),
            },
        }),
        AppIntent::RouteToolDragStarted { world_pos } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragStart {
                world_pos: [world_pos.x, world_pos.y],
            },
        }),
        AppIntent::RouteToolDragUpdated { world_pos } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragUpdate {
                world_pos: [world_pos.x, world_pos.y],
            },
        }),
        AppIntent::RouteToolDragEnded => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragEnd,
        }),
        AppIntent::RouteToolScrollRotated { delta } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::ScrollRotate { delta: *delta },
        }),
        AppIntent::IncreaseRouteToolNodeCount => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::IncreaseNodeCount,
        }),
        AppIntent::DecreaseRouteToolNodeCount => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DecreaseNodeCount,
        }),
        AppIntent::IncreaseRouteToolSegmentLength => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::IncreaseSegmentLength,
        }),
        AppIntent::DecreaseRouteToolSegmentLength => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DecreaseSegmentLength,
        }),
        AppIntent::OpenOptionsDialogRequested => Some(HostSessionAction::OpenOptionsDialog),
        AppIntent::CloseOptionsDialogRequested => Some(HostSessionAction::CloseOptionsDialog),
        AppIntent::UndoRequested => Some(HostSessionAction::Undo),
        AppIntent::RedoRequested => Some(HostSessionAction::Redo),
        _ => None,
    }
}

/// Uebersetzt eine explizite Host-Action in einen stabilen Engine-Intent.
///
/// Gibt `None` zurueck, wenn die Action keinen direkten Intent erzeugt
/// (z. B. ein abgebrochenes Dialog-Ergebnis).
pub fn map_host_action_to_intent(action: HostSessionAction) -> Option<AppIntent> {
    use fs25_auto_drive_engine::app::ui_contract::dialog_result_to_intent;

    match action {
        HostSessionAction::OpenFile => Some(AppIntent::OpenFileRequested),
        HostSessionAction::Save => Some(AppIntent::SaveRequested),
        HostSessionAction::SaveAs => Some(AppIntent::SaveAsRequested),
        HostSessionAction::RequestHeightmapSelection => {
            Some(AppIntent::HeightmapSelectionRequested)
        }
        HostSessionAction::RequestBackgroundMapSelection => {
            Some(AppIntent::BackgroundMapSelectionRequested)
        }
        HostSessionAction::GenerateOverview => Some(AppIntent::GenerateOverviewRequested),
        HostSessionAction::CurseplayImport => Some(AppIntent::CurseplayImportRequested),
        HostSessionAction::CurseplayExport => Some(AppIntent::CurseplayExportRequested),
        HostSessionAction::ResetCamera => Some(AppIntent::ResetCameraRequested),
        HostSessionAction::ZoomToFit => Some(AppIntent::ZoomToFitRequested),
        HostSessionAction::ZoomToSelectionBounds => Some(AppIntent::ZoomToSelectionBoundsRequested),
        HostSessionAction::Exit => Some(AppIntent::ExitRequested),
        HostSessionAction::ToggleCommandPalette => Some(AppIntent::CommandPaletteToggled),
        HostSessionAction::SetEditorTool { tool } => Some(AppIntent::SetEditorToolRequested {
            tool: map_host_active_tool(tool),
        }),
        HostSessionAction::RouteTool { action } => Some(map_route_tool_action_to_intent(action)),
        HostSessionAction::SetDefaultDirection { direction } => {
            Some(AppIntent::SetDefaultDirectionRequested {
                direction: map_host_connection_direction(direction),
            })
        }
        HostSessionAction::SetDefaultPriority { priority } => {
            Some(AppIntent::SetDefaultPriorityRequested {
                priority: map_host_connection_priority(priority),
            })
        }
        HostSessionAction::ApplyOptions { options } => Some(AppIntent::OptionsChanged { options }),
        HostSessionAction::ResetOptions => Some(AppIntent::ResetOptionsRequested),
        HostSessionAction::OpenOptionsDialog => Some(AppIntent::OpenOptionsDialogRequested),
        HostSessionAction::CloseOptionsDialog => Some(AppIntent::CloseOptionsDialogRequested),
        HostSessionAction::Undo => Some(AppIntent::UndoRequested),
        HostSessionAction::Redo => Some(AppIntent::RedoRequested),
        HostSessionAction::SubmitViewportInput { .. } => None,
        HostSessionAction::SubmitDialogResult { result } => {
            dialog_result_to_intent(map_dialog_result(result))
        }
    }
}

/// Entnimmt ausstehende Dialog-Anforderungen als Host-Bridge-DTOs.
///
/// Diese Funktion ist fuer Host-Adapter gedacht, die weiterhin einen eigenen
/// `AppController`/`AppState` besitzen, den Dialog-Lifecycle aber bereits ueber
/// die kanonischen `HostDialogRequest`-DTOs konsolidieren wollen.
///
/// Chrome-Sichtbarkeits-Requests (`ToggleCommandPalette` etc.) werden hier
/// herausgefiltert und NICHT zurueckgegeben — sie sind fuer die
/// `HostBridgeSession::drain_engine_requests()`-Seam bestimmt.
pub fn take_host_dialog_requests(
    controller: &fs25_auto_drive_engine::app::AppController,
    state: &mut AppState,
) -> Vec<HostDialogRequest> {
    controller
        .take_dialog_requests(state)
        .into_iter()
        .filter(|r| !r.is_chrome_request())
        .map(map_engine_dialog_request)
        .collect()
}

/// Baut den host-neutralen Route-Tool-Eintrag-Snapshot auf.
pub(super) fn build_route_tool_entries_snapshot(
    state: &AppState,
) -> Vec<crate::dto::HostRouteToolEntrySnapshot> {
    let availability = route_tool_availability_context(state);
    let mut entries = Vec::new();

    for surface in [
        RouteToolSurface::MainMenu,
        RouteToolSurface::DefaultsPanel,
        RouteToolSurface::FloatingMenu,
        RouteToolSurface::CommandPalette,
    ] {
        for group in [
            RouteToolGroup::Basics,
            RouteToolGroup::Section,
            RouteToolGroup::Analysis,
        ] {
            for entry in resolve_route_tool_entries(surface, group, availability) {
                entries.push(crate::dto::HostRouteToolEntrySnapshot {
                    surface: map_route_tool_surface(surface),
                    group: map_route_tool_group(group),
                    tool: map_route_tool_id(entry.descriptor.id),
                    slot: entry.slot,
                    icon_key: map_route_tool_icon_key(entry.descriptor.icon_key),
                    enabled: entry.enabled,
                    disabled_reason: entry.disabled_reason.map(map_route_tool_disabled_reason),
                });
            }
        }
    }

    entries
}

/// Baut den host-neutralen Route-Tool-Auswahl-Snapshot auf.
pub(super) fn build_route_tool_selection_snapshot(
    state: &AppState,
) -> crate::dto::HostRouteToolSelectionSnapshot {
    crate::dto::HostRouteToolSelectionSnapshot {
        basics: map_route_tool_id(state.editor.route_tool_memory.basics),
        section: map_route_tool_id(state.editor.route_tool_memory.section),
        analysis: map_route_tool_id(state.editor.route_tool_memory.analysis),
    }
}
