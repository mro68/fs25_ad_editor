//! Bridge-interne Kontextmenue-Logik fuer host-neutrale Snapshots.

use crate::dispatch::build_route_tool_viewport_snapshot;
use crate::dto::{HostContextMenuAction, HostContextMenuSnapshot, HostContextMenuVariant};
use fs25_auto_drive_engine::app::{AppState, GroupRegistry, RoadMap};
use fs25_auto_drive_engine::shared::{t, I18nKey, Language};
use indexmap::IndexSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextMenuActionId {
    SetToolSelect,
    SetToolConnect,
    SetToolAddNode,
    SetToolRouteStraight,
    SetToolRouteSmoothCurve,
    SetToolRouteQuadratic,
    SetToolRouteCubic,
    CreateMarker,
    EditMarker,
    RemoveMarker,
    ConnectTwoNodes,
    RouteStraight,
    RouteSmoothCurve,
    RouteQuadratic,
    RouteCubic,
    DirectionRegular,
    DirectionDual,
    DirectionReverse,
    DirectionInvert,
    PriorityRegular,
    PrioritySub,
    RemoveAllConnections,
    Streckenteilung,
    InvertSelection,
    SelectAll,
    ClearSelection,
    DeleteSelected,
    RouteExecute,
    RouteRecreate,
    RouteCancel,
    CopySelection,
    PasteHere,
    EditGroup,
    GroupSelectionAsGroup,
    RemoveFromGroup,
    DissolveGroup,
    ZoomToFit,
    ZoomToSelection,
}

impl ContextMenuActionId {
    fn as_str(self) -> &'static str {
        match self {
            Self::SetToolSelect => "set_tool_select",
            Self::SetToolConnect => "set_tool_connect",
            Self::SetToolAddNode => "set_tool_add_node",
            Self::SetToolRouteStraight => "set_tool_route_straight",
            Self::SetToolRouteSmoothCurve => "set_tool_route_smooth_curve",
            Self::SetToolRouteQuadratic => "set_tool_route_quadratic",
            Self::SetToolRouteCubic => "set_tool_route_cubic",
            Self::CreateMarker => "create_marker",
            Self::EditMarker => "edit_marker",
            Self::RemoveMarker => "remove_marker",
            Self::ConnectTwoNodes => "connect_two_nodes",
            Self::RouteStraight => "route_straight",
            Self::RouteSmoothCurve => "route_smooth_curve",
            Self::RouteQuadratic => "route_quadratic",
            Self::RouteCubic => "route_cubic",
            Self::DirectionRegular => "direction_regular",
            Self::DirectionDual => "direction_dual",
            Self::DirectionReverse => "direction_reverse",
            Self::DirectionInvert => "direction_invert",
            Self::PriorityRegular => "priority_regular",
            Self::PrioritySub => "priority_sub",
            Self::RemoveAllConnections => "remove_all_connections",
            Self::Streckenteilung => "streckenteilung",
            Self::InvertSelection => "invert_selection",
            Self::SelectAll => "select_all",
            Self::ClearSelection => "clear_selection",
            Self::DeleteSelected => "delete_selected",
            Self::RouteExecute => "route_execute",
            Self::RouteRecreate => "route_recreate",
            Self::RouteCancel => "route_cancel",
            Self::CopySelection => "copy_selection",
            Self::PasteHere => "paste_here",
            Self::EditGroup => "edit_group",
            Self::GroupSelectionAsGroup => "group_selection_as_group",
            Self::RemoveFromGroup => "remove_from_group",
            Self::DissolveGroup => "dissolve_group",
            Self::ZoomToFit => "zoom_to_fit",
            Self::ZoomToSelection => "zoom_to_selection",
        }
    }

    fn label(self, lang: Language) -> String {
        match self {
            Self::SetToolSelect => t(lang, I18nKey::CtxToolSelect).to_string(),
            Self::SetToolConnect => t(lang, I18nKey::CtxToolConnect).to_string(),
            Self::SetToolAddNode => t(lang, I18nKey::CtxToolAddNode).to_string(),
            Self::SetToolRouteStraight => t(lang, I18nKey::CtxRouteStraight).to_string(),
            Self::SetToolRouteSmoothCurve => t(lang, I18nKey::CtxRouteSmoothCurve).to_string(),
            Self::SetToolRouteQuadratic => t(lang, I18nKey::CtxRouteQuadratic).to_string(),
            Self::SetToolRouteCubic => t(lang, I18nKey::CtxRouteCubic).to_string(),
            Self::CreateMarker => match lang {
                Language::De => "Marker erstellen".to_string(),
                Language::En => "Create marker".to_string(),
            },
            Self::EditMarker => match lang {
                Language::De => "Marker bearbeiten".to_string(),
                Language::En => "Edit marker".to_string(),
            },
            Self::RemoveMarker => match lang {
                Language::De => "Marker loeschen".to_string(),
                Language::En => "Remove marker".to_string(),
            },
            Self::ConnectTwoNodes => t(lang, I18nKey::CtxConnectNodes).to_string(),
            Self::RouteStraight => t(lang, I18nKey::CtxRouteStraight).to_string(),
            Self::RouteSmoothCurve => t(lang, I18nKey::CtxRouteSmoothCurve).to_string(),
            Self::RouteQuadratic => t(lang, I18nKey::CtxRouteQuadratic).to_string(),
            Self::RouteCubic => t(lang, I18nKey::CtxRouteCubic).to_string(),
            Self::DirectionRegular => t(lang, I18nKey::CtxDirectionRegular).to_string(),
            Self::DirectionDual => t(lang, I18nKey::CtxDirectionDual).to_string(),
            Self::DirectionReverse => t(lang, I18nKey::CtxDirectionReverse).to_string(),
            Self::DirectionInvert => t(lang, I18nKey::CtxDirectionInvert).to_string(),
            Self::PriorityRegular => t(lang, I18nKey::CtxPriorityMain).to_string(),
            Self::PrioritySub => t(lang, I18nKey::CtxPrioritySub).to_string(),
            Self::RemoveAllConnections => t(lang, I18nKey::CtxRemoveAllConnections).to_string(),
            Self::Streckenteilung => t(lang, I18nKey::CtxStreckenteilung).to_string(),
            Self::InvertSelection => t(lang, I18nKey::CtxSelectionInvert).to_string(),
            Self::SelectAll => t(lang, I18nKey::CtxSelectAll).to_string(),
            Self::ClearSelection => t(lang, I18nKey::CtxClearSelection).to_string(),
            Self::DeleteSelected => t(lang, I18nKey::CtxDeleteSelected).to_string(),
            Self::RouteExecute => match lang {
                Language::De => "Route ausfuehren".to_string(),
                Language::En => "Execute route".to_string(),
            },
            Self::RouteRecreate => match lang {
                Language::De => "Route neu berechnen".to_string(),
                Language::En => "Recreate route".to_string(),
            },
            Self::RouteCancel => match lang {
                Language::De => "Route abbrechen".to_string(),
                Language::En => "Cancel route".to_string(),
            },
            Self::CopySelection => t(lang, I18nKey::CtxCopy).to_string(),
            Self::PasteHere => t(lang, I18nKey::CtxPaste).to_string(),
            Self::EditGroup => t(lang, I18nKey::CtxEditGroup).to_string(),
            Self::GroupSelectionAsGroup => t(lang, I18nKey::CtxGroupAsSegment).to_string(),
            Self::RemoveFromGroup => t(lang, I18nKey::CtxRemoveFromGroup).to_string(),
            Self::DissolveGroup => t(lang, I18nKey::CtxDissolveGroup).to_string(),
            Self::ZoomToFit => t(lang, I18nKey::CtxZoomFullMap).to_string(),
            Self::ZoomToSelection => t(lang, I18nKey::CtxZoomSelection).to_string(),
        }
    }

    fn enabled(self, ctx: &ContextMenuContext<'_>) -> bool {
        match self {
            Self::SetToolSelect
            | Self::SetToolConnect
            | Self::SetToolAddNode
            | Self::SetToolRouteStraight
            | Self::SetToolRouteSmoothCurve
            | Self::SetToolRouteQuadratic
            | Self::SetToolRouteCubic
            | Self::InvertSelection
            | Self::SelectAll
            | Self::ClearSelection
            | Self::DeleteSelected
            | Self::RouteExecute
            | Self::RouteRecreate
            | Self::RouteCancel
            | Self::ZoomToFit => true,
            Self::CreateMarker => ctx.focus_node_id.is_some_and(|node_id| {
                ctx.road_map.contains_node(node_id) && !ctx.road_map.has_marker(node_id)
            }),
            Self::EditMarker | Self::RemoveMarker => ctx.focus_node_id.is_some_and(|node_id| {
                ctx.road_map.contains_node(node_id) && ctx.road_map.has_marker(node_id)
            }),
            Self::ConnectTwoNodes => is_two_selected_unconnected(ctx),
            Self::RouteStraight
            | Self::RouteSmoothCurve
            | Self::RouteQuadratic
            | Self::RouteCubic => ctx.selected_node_ids.len() == 2,
            Self::DirectionRegular
            | Self::DirectionDual
            | Self::DirectionReverse
            | Self::DirectionInvert
            | Self::PriorityRegular
            | Self::PrioritySub
            | Self::RemoveAllConnections => has_connections_between_selected(ctx),
            Self::Streckenteilung => {
                !ctx.distanzen_active && ctx.road_map.is_resampleable_chain(ctx.selected_node_ids)
            }
            Self::CopySelection => !ctx.selected_node_ids.is_empty(),
            Self::PasteHere => ctx.clipboard_has_data,
            Self::EditGroup => ctx.group_record_id.is_some(),
            Self::GroupSelectionAsGroup => {
                !ctx.group_editing_active
                    && ctx.road_map.is_connected_subgraph(ctx.selected_node_ids)
            }
            Self::RemoveFromGroup | Self::DissolveGroup => ctx.selection_has_group_member,
            Self::ZoomToSelection => ctx.selected_node_ids.len() >= 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextMenuVariant {
    EmptyArea,
    SelectionOnly,
    NodeFocused,
    RouteToolActive,
}

impl From<ContextMenuVariant> for HostContextMenuVariant {
    fn from(value: ContextMenuVariant) -> Self {
        match value {
            ContextMenuVariant::EmptyArea => Self::EmptyArea,
            ContextMenuVariant::SelectionOnly => Self::SelectionOnly,
            ContextMenuVariant::NodeFocused => Self::NodeFocused,
            ContextMenuVariant::RouteToolActive => Self::RouteToolActive,
        }
    }
}

struct ContextMenuContext<'a> {
    road_map: &'a RoadMap,
    selected_node_ids: &'a IndexSet<u64>,
    focus_node_id: Option<u64>,
    distanzen_active: bool,
    clipboard_has_data: bool,
    group_record_id: Option<u64>,
    group_editing_active: bool,
    selection_has_group_member: bool,
    language: Language,
}

struct ActionSpec {
    id: ContextMenuActionId,
    group: Option<&'static str>,
}

impl ActionSpec {
    fn new(id: ContextMenuActionId, group: Option<&'static str>) -> Self {
        Self { id, group }
    }

    fn build(self, ctx: &ContextMenuContext<'_>) -> HostContextMenuAction {
        HostContextMenuAction {
            id: self.id.as_str().to_string(),
            label: self.id.label(ctx.language),
            enabled: self.id.enabled(ctx),
            group: self.group.map(str::to_string),
        }
    }
}

pub(super) fn build_context_menu_snapshot(
    state: &AppState,
    focus_node_id: Option<u64>,
) -> HostContextMenuSnapshot {
    let route_tool_has_input = build_route_tool_viewport_snapshot(state).has_pending_input;
    let variant = determine_menu_variant(
        &state.selection.selected_node_ids,
        focus_node_id,
        route_tool_has_input,
    );

    let Some(road_map) = state.road_map.as_deref() else {
        return HostContextMenuSnapshot {
            variant: variant.into(),
            focus_node_id,
            available_actions: Vec::new(),
        };
    };

    let ctx = ContextMenuContext {
        road_map,
        selected_node_ids: &state.selection.selected_node_ids,
        focus_node_id,
        distanzen_active: state.ui.distanzen.active,
        clipboard_has_data: !state.clipboard.nodes.is_empty(),
        group_record_id: compute_group_record_id(
            &state.selection.selected_node_ids,
            &state.group_registry,
            road_map,
        ),
        group_editing_active: state.group_editing.is_some(),
        selection_has_group_member: has_group_member_in_selection(
            &state.selection.selected_node_ids,
            &state.group_registry,
        ),
        language: state.options.language,
    };

    let available_actions = action_specs_for_variant(variant)
        .into_iter()
        .map(|spec| spec.build(&ctx))
        .collect();

    HostContextMenuSnapshot {
        variant: variant.into(),
        focus_node_id,
        available_actions,
    }
}

fn determine_menu_variant(
    selected_node_ids: &IndexSet<u64>,
    focus_node_id: Option<u64>,
    route_tool_has_input: bool,
) -> ContextMenuVariant {
    if route_tool_has_input && focus_node_id.is_none() {
        return ContextMenuVariant::RouteToolActive;
    }
    if focus_node_id.is_some() {
        return ContextMenuVariant::NodeFocused;
    }
    if !selected_node_ids.is_empty() {
        return ContextMenuVariant::SelectionOnly;
    }
    ContextMenuVariant::EmptyArea
}

fn compute_group_record_id(
    selected_node_ids: &IndexSet<u64>,
    group_registry: &GroupRegistry,
    road_map: &RoadMap,
) -> Option<u64> {
    let records = group_registry.find_by_node_ids(selected_node_ids);
    if records.len() != 1 {
        return None;
    }

    let record = records[0];
    let all_belong = selected_node_ids
        .iter()
        .all(|node_id| record.node_ids.contains(node_id));
    if all_belong && group_registry.is_group_valid(record, road_map) {
        return Some(record.id);
    }
    None
}

fn has_group_member_in_selection(
    selected_node_ids: &IndexSet<u64>,
    group_registry: &GroupRegistry,
) -> bool {
    selected_node_ids
        .iter()
        .any(|&node_id| !group_registry.groups_for_node(node_id).is_empty())
}

fn is_two_selected_unconnected(ctx: &ContextMenuContext<'_>) -> bool {
    if ctx.selected_node_ids.len() != 2 {
        return false;
    }

    let ids: Vec<u64> = ctx.selected_node_ids.iter().copied().collect();
    let (a, b) = (ids[0], ids[1]);
    !ctx.road_map.has_connection(a, b) && !ctx.road_map.has_connection(b, a)
}

fn has_connections_between_selected(ctx: &ContextMenuContext<'_>) -> bool {
    ctx.road_map
        .connections_between_ids(ctx.selected_node_ids)
        .next()
        .is_some()
}

fn action_specs_for_variant(variant: ContextMenuVariant) -> Vec<ActionSpec> {
    match variant {
        ContextMenuVariant::EmptyArea => {
            let mut actions = tool_actions();
            actions.extend(zoom_actions());
            actions.extend(route_tool_selection_actions());
            actions
        }
        ContextMenuVariant::SelectionOnly => {
            let mut actions = tool_actions();
            actions.extend(zoom_actions());
            actions.extend(selection_actions());
            actions.push(ActionSpec::new(
                ContextMenuActionId::DeleteSelected,
                Some("selection"),
            ));
            actions.extend(clipboard_actions());
            actions
        }
        ContextMenuVariant::NodeFocused => {
            let mut actions = tool_actions();
            actions.extend(zoom_actions());
            actions.extend(marker_actions());
            actions.push(ActionSpec::new(
                ContextMenuActionId::DeleteSelected,
                Some("selection"),
            ));
            actions.extend(selection_actions());
            actions.extend(clipboard_actions());
            actions
        }
        ContextMenuVariant::RouteToolActive => route_tool_runtime_actions(),
    }
}

fn tool_actions() -> Vec<ActionSpec> {
    vec![
        ActionSpec::new(ContextMenuActionId::SetToolSelect, Some("tool")),
        ActionSpec::new(ContextMenuActionId::SetToolConnect, Some("tool")),
        ActionSpec::new(ContextMenuActionId::SetToolAddNode, Some("tool")),
    ]
}

fn zoom_actions() -> Vec<ActionSpec> {
    vec![
        ActionSpec::new(ContextMenuActionId::ZoomToFit, Some("zoom")),
        ActionSpec::new(ContextMenuActionId::ZoomToSelection, Some("zoom")),
    ]
}

fn route_tool_selection_actions() -> Vec<ActionSpec> {
    vec![
        ActionSpec::new(
            ContextMenuActionId::SetToolRouteSmoothCurve,
            Some("route_tool"),
        ),
        ActionSpec::new(
            ContextMenuActionId::SetToolRouteStraight,
            Some("route_tool"),
        ),
        ActionSpec::new(
            ContextMenuActionId::SetToolRouteQuadratic,
            Some("route_tool"),
        ),
        ActionSpec::new(ContextMenuActionId::SetToolRouteCubic, Some("route_tool")),
    ]
}

fn marker_actions() -> Vec<ActionSpec> {
    vec![
        ActionSpec::new(ContextMenuActionId::EditMarker, Some("marker")),
        ActionSpec::new(ContextMenuActionId::RemoveMarker, Some("marker")),
        ActionSpec::new(ContextMenuActionId::CreateMarker, Some("marker")),
    ]
}

fn selection_actions() -> Vec<ActionSpec> {
    vec![
        ActionSpec::new(ContextMenuActionId::EditGroup, Some("group")),
        ActionSpec::new(ContextMenuActionId::GroupSelectionAsGroup, Some("group")),
        ActionSpec::new(ContextMenuActionId::RemoveFromGroup, Some("group")),
        ActionSpec::new(ContextMenuActionId::DissolveGroup, Some("group")),
        ActionSpec::new(ContextMenuActionId::ConnectTwoNodes, Some("route")),
        ActionSpec::new(ContextMenuActionId::RouteSmoothCurve, Some("route")),
        ActionSpec::new(ContextMenuActionId::RouteStraight, Some("route")),
        ActionSpec::new(ContextMenuActionId::RouteQuadratic, Some("route")),
        ActionSpec::new(ContextMenuActionId::RouteCubic, Some("route")),
        ActionSpec::new(ContextMenuActionId::DirectionRegular, Some("direction")),
        ActionSpec::new(ContextMenuActionId::DirectionDual, Some("direction")),
        ActionSpec::new(ContextMenuActionId::DirectionReverse, Some("direction")),
        ActionSpec::new(ContextMenuActionId::DirectionInvert, Some("direction")),
        ActionSpec::new(ContextMenuActionId::PriorityRegular, Some("priority")),
        ActionSpec::new(ContextMenuActionId::PrioritySub, Some("priority")),
        ActionSpec::new(ContextMenuActionId::RemoveAllConnections, Some("priority")),
        ActionSpec::new(ContextMenuActionId::InvertSelection, Some("selection")),
        ActionSpec::new(ContextMenuActionId::SelectAll, Some("selection")),
        ActionSpec::new(ContextMenuActionId::ClearSelection, Some("selection")),
        ActionSpec::new(ContextMenuActionId::Streckenteilung, Some("resample")),
    ]
}

fn clipboard_actions() -> Vec<ActionSpec> {
    vec![
        ActionSpec::new(ContextMenuActionId::CopySelection, Some("clipboard")),
        ActionSpec::new(ContextMenuActionId::PasteHere, Some("clipboard")),
    ]
}

fn route_tool_runtime_actions() -> Vec<ActionSpec> {
    vec![
        ActionSpec::new(ContextMenuActionId::RouteExecute, Some("route_tool_active")),
        ActionSpec::new(
            ContextMenuActionId::RouteRecreate,
            Some("route_tool_active"),
        ),
        ActionSpec::new(ContextMenuActionId::RouteCancel, Some("route_tool_active")),
    ]
}

#[cfg(test)]
mod tests {
    use super::build_context_menu_snapshot;
    use crate::dto::HostContextMenuVariant;
    use crate::{HostBridgeSession, HostSessionAction};
    use fs25_auto_drive_engine::app::{MapMarker, MapNode, NodeFlag, RoadMap};
    use glam::Vec2;
    use std::sync::Arc;

    fn action_enabled(snapshot: &crate::dto::HostContextMenuSnapshot, id: &str) -> bool {
        snapshot
            .available_actions
            .iter()
            .find(|action| action.id == id)
            .map(|action| action.enabled)
            .unwrap_or(false)
    }

    fn selection_test_map() -> RoadMap {
        let mut road_map = RoadMap::new(4);
        road_map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
        road_map.ensure_spatial_index();
        road_map
    }

    #[test]
    fn selection_snapshot_marks_actions_enabled_and_disabled_from_state() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(selection_test_map()));
        session.state.selection.ids_mut().insert(1);
        session.state.selection.ids_mut().insert(2);

        let snapshot = build_context_menu_snapshot(&session.state, None);

        assert_eq!(snapshot.variant, HostContextMenuVariant::SelectionOnly);
        assert!(action_enabled(&snapshot, "connect_two_nodes"));
        assert!(action_enabled(&snapshot, "copy_selection"));
        assert!(!action_enabled(&snapshot, "paste_here"));
        assert!(!action_enabled(&snapshot, "remove_all_connections"));

        session
            .apply_action(HostSessionAction::ConnectSelectedNodes)
            .expect("Selektierte Nodes muessen verbunden werden koennen");

        let connected_snapshot = build_context_menu_snapshot(&session.state, None);
        assert!(!action_enabled(&connected_snapshot, "connect_two_nodes"));
        assert!(action_enabled(
            &connected_snapshot,
            "remove_all_connections"
        ));
    }

    #[test]
    fn node_focused_snapshot_reflects_marker_preconditions() {
        let mut session = HostBridgeSession::new();
        session.state.road_map = Some(Arc::new(selection_test_map()));

        let without_marker = session.context_menu_snapshot(Some(1));
        assert_eq!(without_marker.variant, HostContextMenuVariant::NodeFocused);
        assert!(action_enabled(&without_marker, "create_marker"));
        assert!(!action_enabled(&without_marker, "edit_marker"));

        Arc::make_mut(
            session
                .state
                .road_map
                .as_mut()
                .expect("RoadMap muss vorhanden sein"),
        )
        .add_map_marker(MapMarker::new(
            1,
            "Hof".to_string(),
            "All".to_string(),
            1,
            false,
        ));

        let with_marker = session.context_menu_snapshot(Some(1));
        assert!(!action_enabled(&with_marker, "create_marker"));
        assert!(action_enabled(&with_marker, "edit_marker"));
        assert!(action_enabled(&with_marker, "remove_marker"));
    }
}
