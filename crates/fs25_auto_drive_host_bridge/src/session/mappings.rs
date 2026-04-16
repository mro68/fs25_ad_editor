use fs25_auto_drive_engine::app::tool_contract::RouteToolId;
use fs25_auto_drive_engine::app::{ConnectionDirection, ConnectionPriority, EditorTool};
use fs25_auto_drive_engine::shared::{OverviewFieldDetectionSource, OverviewLayerOptions};

use crate::dto::{
    HostActiveTool, HostDefaultConnectionDirection, HostDefaultConnectionPriority,
    HostFieldDetectionSource, HostOverviewLayersSnapshot, HostRouteToolId,
};

pub(super) fn map_active_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
    }
}

pub(super) fn map_connection_direction(
    direction: ConnectionDirection,
) -> HostDefaultConnectionDirection {
    match direction {
        ConnectionDirection::Regular => HostDefaultConnectionDirection::Regular,
        ConnectionDirection::Dual => HostDefaultConnectionDirection::Dual,
        ConnectionDirection::Reverse => HostDefaultConnectionDirection::Reverse,
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

pub(super) fn map_host_field_detection_source_to_engine(
    source: HostFieldDetectionSource,
) -> OverviewFieldDetectionSource {
    match source {
        HostFieldDetectionSource::FromZip => OverviewFieldDetectionSource::FromZip,
        HostFieldDetectionSource::ZipGroundGdm => OverviewFieldDetectionSource::ZipGroundGdm,
        HostFieldDetectionSource::FieldTypeGrle => OverviewFieldDetectionSource::FieldTypeGrle,
        HostFieldDetectionSource::GroundGdm => OverviewFieldDetectionSource::GroundGdm,
        HostFieldDetectionSource::FruitsGdm => OverviewFieldDetectionSource::FruitsGdm,
    }
}

pub(super) fn map_host_overview_layers_to_engine(
    layers: &HostOverviewLayersSnapshot,
) -> OverviewLayerOptions {
    OverviewLayerOptions {
        terrain: layers.terrain,
        hillshade: layers.hillshade,
        farmlands: layers.farmlands,
        farmland_ids: layers.farmland_ids,
        pois: layers.pois,
        legend: layers.legend,
    }
}

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
