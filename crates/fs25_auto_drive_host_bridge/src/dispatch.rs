//! Kanonische Dispatch-Seam und Snapshot-Builder der fs25_auto_drive_host_bridge.
//!
//! Intern in thematische Submodule aufgeteilt; die oeffentliche Schnittstelle ist unveraendert.

mod actions;
mod mappings;
mod snapshot;
mod viewport_input;

pub use actions::{
    apply_host_action, apply_host_action_with_viewport_input_state, apply_mapped_intent,
};
pub use mappings::{
    map_host_action_to_intent, map_intent_to_host_action, take_host_dialog_requests,
};
pub(crate) use mappings::map_engine_dialog_request;
pub use snapshot::{
    build_host_chrome_snapshot, build_host_ui_snapshot, build_render_assets, build_render_frame,
    build_render_scene, build_route_tool_viewport_snapshot, build_viewport_geometry_snapshot,
    build_viewport_overlay_snapshot,
};
pub use viewport_input::{apply_viewport_input_batch, HostViewportInputState};

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::ui_contract::{BypassPanelAction, RouteToolPanelAction};
    use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};
    use fs25_auto_drive_engine::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;
    use std::sync::Arc;

    use crate::dto::{
        HostActiveTool, HostDefaultConnectionDirection, HostDefaultConnectionPriority,
        HostDialogRequestKind, HostDialogResult, HostRouteToolAction, HostRouteToolDisabledReason,
        HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId, HostRouteToolSurface,
        HostSessionAction, HostTangentSource, HostViewportConnectionDirection,
        HostViewportConnectionPriority, HostViewportNodeKind,
    };

    use super::{
        apply_host_action, apply_mapped_intent, build_host_chrome_snapshot, build_host_ui_snapshot,
        build_render_assets, build_render_frame, build_render_scene,
        build_route_tool_viewport_snapshot, build_viewport_geometry_snapshot,
        build_viewport_overlay_snapshot, map_host_action_to_intent, map_intent_to_host_action,
        take_host_dialog_requests,
    };

    fn geometry_test_map() -> RoadMap {
        let mut map = RoadMap::new(2);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(20.0, 10.0), NodeFlag::SubPrio));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Dual,
            ConnectionPriority::SubPriority,
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 10.0),
        ));
        map.add_map_marker(MapMarker::new(
            1,
            "Hof".to_string(),
            "Farmen".to_string(),
            1,
            false,
        ));
        map.ensure_spatial_index();
        map
    }

    #[test]
    fn take_host_dialog_requests_maps_and_drains_engine_queue() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        controller
            .handle_intent(&mut state, AppIntent::OpenFileRequested)
            .expect("OpenFileRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::HeightmapSelectionRequested)
            .expect("HeightmapSelectionRequested muss Dialog-Anforderung erzeugen");

        let requests = take_host_dialog_requests(&controller, &mut state);
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].kind, HostDialogRequestKind::OpenFile);
        assert_eq!(requests[1].kind, HostDialogRequestKind::Heightmap);

        let drained = take_host_dialog_requests(&controller, &mut state);
        assert!(drained.is_empty());
    }

    #[test]
    fn take_host_dialog_requests_covers_save_background_and_curseplay_export_requests() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        controller
            .handle_intent(&mut state, AppIntent::SaveAsRequested)
            .expect("SaveAsRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::BackgroundMapSelectionRequested)
            .expect("BackgroundMapSelectionRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::CurseplayExportRequested)
            .expect("CurseplayExportRequested muss Dialog-Anforderung erzeugen");

        let requests = take_host_dialog_requests(&controller, &mut state);
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].kind, HostDialogRequestKind::SaveFile);
        assert_eq!(requests[1].kind, HostDialogRequestKind::BackgroundMap);
        assert_eq!(requests[2].kind, HostDialogRequestKind::CurseplayExport);
    }

    #[test]
    fn apply_host_action_dispatches_mapped_action() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::ToggleCommandPalette,
        )
        .expect("ToggleCommandPalette muss verarbeitet werden");

        assert!(handled);
        assert!(
            state
                .ui
                .dialog_requests
                .iter()
                .any(|r| matches!(r, fs25_auto_drive_engine::app::ui_contract::DialogRequest::ToggleCommandPalette)),
            "ToggleCommandPalette muss in dialog_requests stehen"
        );
    }

    #[test]
    fn apply_host_action_returns_false_for_dialog_cancel_without_intent() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::Cancelled {
                    kind: HostDialogRequestKind::OpenFile,
                },
            },
        )
        .expect("Abgebrochene Dialoge duerfen keinen Fehler ausloesen");

        assert!(!handled);
        assert!(state.ui.dialog_requests.is_empty());
    }

    #[test]
    fn apply_host_action_dispatches_dialog_path_selected_into_state() {
        let mut controller = AppController::new();
        let mut state = AppState::new();
        let selected_path = "/tmp/test_heightmap.png".to_string();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::Heightmap,
                    path: selected_path.clone(),
                },
            },
        )
        .expect("PathSelected muss einen Intent erzeugen und verarbeitet werden");

        assert!(handled);
        assert_eq!(state.ui.heightmap_path, Some(selected_path));
    }

    #[test]
    fn build_viewport_geometry_snapshot_exposes_minimal_geometry_transport() {
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(geometry_test_map()));
        state.view.camera.position = Vec2::new(3.0, -4.0);
        state.view.camera.zoom = 2.0;
        state.selection.ids_mut().insert(2);

        let snapshot = build_viewport_geometry_snapshot(&state, [640.0, 320.0]);

        assert!(snapshot.has_map);
        assert_eq!(snapshot.viewport_size, [640.0, 320.0]);
        assert_eq!(snapshot.camera_position, [3.0, -4.0]);
        assert_eq!(snapshot.zoom, 2.0);
        assert_eq!(snapshot.nodes.len(), 2);
        assert_eq!(snapshot.connections.len(), 1);
        assert_eq!(snapshot.markers.len(), 1);
        assert_eq!(snapshot.nodes[0].id, 1);
        assert_eq!(snapshot.nodes[0].kind, HostViewportNodeKind::Regular);
        assert!(!snapshot.nodes[0].selected);
        assert_eq!(snapshot.nodes[1].id, 2);
        assert_eq!(snapshot.nodes[1].kind, HostViewportNodeKind::SubPrio);
        assert!(snapshot.nodes[1].selected);
        assert_eq!(
            snapshot.connections[0].direction,
            HostViewportConnectionDirection::Dual
        );
        assert_eq!(
            snapshot.connections[0].priority,
            HostViewportConnectionPriority::SubPriority
        );
        assert_eq!(snapshot.markers[0].position, [0.0, 0.0]);
        assert!(snapshot.world_per_pixel.is_finite());
        assert!(snapshot.world_per_pixel > 0.0);
    }

    #[test]
    fn build_render_frame_couples_scene_and_assets_for_local_hosts() {
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(geometry_test_map()));

        let frame = build_render_frame(&state, [640.0, 320.0]);

        assert!(frame.scene.has_map());
        assert_eq!(frame.scene.viewport_size(), [640.0, 320.0]);
        assert_eq!(frame.assets.background_asset_revision(), 0);
        assert_eq!(frame.assets.background_transform_revision(), 0);
        assert!(frame.assets.background().is_none());
    }

    #[test]
    fn map_host_action_to_intent_covers_new_dialog_result_branches() {
        let save_intent = map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
            result: HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::SaveFile,
                path: "/tmp/savegame.xml".to_string(),
            },
        });
        let background_zip_intent =
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::BackgroundMap,
                    path: "/tmp/map_overview.zip".to_string(),
                },
            });
        let curseplay_export_intent =
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::CurseplayExport,
                    path: "/tmp/customField.xml".to_string(),
                },
            });

        assert!(matches!(
            save_intent,
            Some(AppIntent::SaveFilePathSelected { path }) if path == "/tmp/savegame.xml"
        ));
        assert!(matches!(
            background_zip_intent,
            Some(AppIntent::ZipBackgroundBrowseRequested { path }) if path == "/tmp/map_overview.zip"
        ));
        assert!(matches!(
            curseplay_export_intent,
            Some(AppIntent::CurseplayExportPathSelected { path }) if path == "/tmp/customField.xml"
        ));
    }

    #[test]
    fn map_host_action_to_intent_covers_route_tool_and_chrome_writes() {
        let route_intent = map_host_action_to_intent(HostSessionAction::RouteTool {
            action: HostRouteToolAction::ScrollRotate { delta: -1.0 },
        });
        let default_direction_intent =
            map_host_action_to_intent(HostSessionAction::SetDefaultDirection {
                direction: HostDefaultConnectionDirection::Reverse,
            });
        let default_priority_intent =
            map_host_action_to_intent(HostSessionAction::SetDefaultPriority {
                priority: HostDefaultConnectionPriority::SubPriority,
            });

        assert!(matches!(
            route_intent,
            Some(AppIntent::RouteToolScrollRotated { delta }) if (delta + 1.0).abs() < f32::EPSILON
        ));
        assert!(matches!(
            default_direction_intent,
            Some(AppIntent::SetDefaultDirectionRequested {
                direction: ConnectionDirection::Reverse
            })
        ));
        assert!(matches!(
            default_priority_intent,
            Some(AppIntent::SetDefaultPriorityRequested {
                priority: ConnectionPriority::SubPriority
            })
        ));
    }

    #[test]
    fn map_intent_to_host_action_covers_stable_bridge_intents() {
        let cases = vec![
            (AppIntent::OpenFileRequested, HostSessionAction::OpenFile),
            (AppIntent::SaveRequested, HostSessionAction::Save),
            (AppIntent::SaveAsRequested, HostSessionAction::SaveAs),
            (
                AppIntent::HeightmapSelectionRequested,
                HostSessionAction::RequestHeightmapSelection,
            ),
            (
                AppIntent::BackgroundMapSelectionRequested,
                HostSessionAction::RequestBackgroundMapSelection,
            ),
            (
                AppIntent::GenerateOverviewRequested,
                HostSessionAction::GenerateOverview,
            ),
            (
                AppIntent::CurseplayImportRequested,
                HostSessionAction::CurseplayImport,
            ),
            (
                AppIntent::CurseplayExportRequested,
                HostSessionAction::CurseplayExport,
            ),
            (
                AppIntent::ResetCameraRequested,
                HostSessionAction::ResetCamera,
            ),
            (AppIntent::ZoomToFitRequested, HostSessionAction::ZoomToFit),
            (
                AppIntent::ZoomToSelectionBoundsRequested,
                HostSessionAction::ZoomToSelectionBounds,
            ),
            (AppIntent::ExitRequested, HostSessionAction::Exit),
            (
                AppIntent::CommandPaletteToggled,
                HostSessionAction::ToggleCommandPalette,
            ),
            (
                AppIntent::SetEditorToolRequested {
                    tool: fs25_auto_drive_engine::app::EditorTool::Route,
                },
                HostSessionAction::SetEditorTool {
                    tool: HostActiveTool::Route,
                },
            ),
            (
                AppIntent::SetDefaultDirectionRequested {
                    direction: ConnectionDirection::Reverse,
                },
                HostSessionAction::SetDefaultDirection {
                    direction: HostDefaultConnectionDirection::Reverse,
                },
            ),
            (
                AppIntent::SetDefaultPriorityRequested {
                    priority: ConnectionPriority::SubPriority,
                },
                HostSessionAction::SetDefaultPriority {
                    priority: HostDefaultConnectionPriority::SubPriority,
                },
            ),
            (
                AppIntent::OptionsChanged {
                    options: Box::new(fs25_auto_drive_engine::shared::EditorOptions::default()),
                },
                HostSessionAction::ApplyOptions {
                    options: Box::new(fs25_auto_drive_engine::shared::EditorOptions::default()),
                },
            ),
            (
                AppIntent::ResetOptionsRequested,
                HostSessionAction::ResetOptions,
            ),
            (
                AppIntent::SelectRouteToolRequested {
                    tool_id: fs25_auto_drive_engine::app::tool_contract::RouteToolId::CurveCubic,
                },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::SelectTool {
                        tool: HostRouteToolId::CurveCubic,
                    },
                },
            ),
            (
                AppIntent::RouteToolPanelActionRequested {
                    action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(2.5)),
                },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::PanelAction {
                        action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(2.5)),
                    },
                },
            ),
            (
                AppIntent::RouteToolTangentSelected {
                    start: fs25_auto_drive_engine::app::tool_contract::TangentSource::Connection {
                        neighbor_id: 7,
                        angle: 0.5,
                    },
                    end: fs25_auto_drive_engine::app::tool_contract::TangentSource::None,
                },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::ApplyTangent {
                        start: HostTangentSource::Connection {
                            neighbor_id: 7,
                            angle: 0.5,
                        },
                        end: HostTangentSource::None,
                    },
                },
            ),
            (
                AppIntent::RouteToolScrollRotated { delta: 1.0 },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::ScrollRotate { delta: 1.0 },
                },
            ),
            (
                AppIntent::OpenOptionsDialogRequested,
                HostSessionAction::OpenOptionsDialog,
            ),
            (
                AppIntent::CloseOptionsDialogRequested,
                HostSessionAction::CloseOptionsDialog,
            ),
            (AppIntent::UndoRequested, HostSessionAction::Undo),
            (AppIntent::RedoRequested, HostSessionAction::Redo),
        ];

        for (intent, expected_action) in cases {
            assert_eq!(map_intent_to_host_action(&intent), Some(expected_action));
        }
    }

    #[test]
    fn map_intent_to_host_action_keeps_high_frequency_intents_unmapped() {
        let cases = vec![
            AppIntent::ViewportResized {
                size: [1920.0, 1080.0],
            },
            AppIntent::CameraPan {
                delta: Vec2::new(3.0, -2.0),
            },
            AppIntent::CameraZoom {
                factor: 1.1,
                focus_world: Some(Vec2::ZERO),
            },
            AppIntent::NodePickRequested {
                world_pos: Vec2::new(5.0, 6.0),
                additive: false,
                extend_path: false,
            },
            AppIntent::AddNodeRequested {
                world_pos: Vec2::new(9.0, 1.0),
            },
        ];

        for intent in cases {
            assert!(map_intent_to_host_action(&intent).is_none());
        }
    }

    #[test]
    fn apply_mapped_intent_dispatches_open_file_request() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled =
            apply_mapped_intent(&mut controller, &mut state, &AppIntent::OpenFileRequested)
                .expect("OpenFileRequested muss ueber die Bridge-Seam verarbeitet werden");

        assert!(handled);
        assert_eq!(state.ui.dialog_requests.len(), 1);
    }

    #[test]
    fn apply_mapped_intent_returns_false_for_unmapped_intents() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_mapped_intent(
            &mut controller,
            &mut state,
            &AppIntent::ViewportResized {
                size: [640.0, 480.0],
            },
        )
        .expect("Unmapped Intent darf keinen Fehler ausloesen");

        assert!(!handled);
    }

    #[test]
    fn read_helpers_delegate_to_controller_read_seams() {
        let mut state = AppState::new();

        let host_ui = build_host_ui_snapshot(&state);
        let overlay = build_viewport_overlay_snapshot(&mut state, None);
        let scene = build_render_scene(&state, [640.0, 480.0]);
        let assets = build_render_assets(&state);

        assert!(host_ui.command_palette_state().is_some());
        assert!(overlay.route_tool_preview.is_none());
        assert_eq!(scene.viewport_size(), [640.0, 480.0]);
        assert_eq!(assets.background_asset_revision(), 0);
    }

    #[test]
    fn build_route_tool_viewport_snapshot_exposes_straight_tool_flags() {
        let road_map = RoadMap::default();
        let mut state = AppState::new();

        state.editor.active_tool = fs25_auto_drive_engine::app::EditorTool::Route;
        state
            .editor
            .tool_manager
            .set_active_by_id(fs25_auto_drive_engine::app::tool_contract::RouteToolId::Straight);
        state
            .editor
            .tool_manager
            .active_tool_mut()
            .expect("Straight-Tool muss fuer den Snapshot-Test aktiv sein")
            .on_click(Vec2::new(0.0, 0.0), &road_map, false);

        let snapshot = build_route_tool_viewport_snapshot(&state);

        assert!(snapshot.has_pending_input);
        assert!(snapshot.drag_targets.is_empty());
        assert!(snapshot.segment_shortcuts_active);
        assert!(snapshot.tangent_menu_data.is_none());
        assert!(!snapshot.needs_lasso_input);
    }

    #[test]
    fn build_host_chrome_snapshot_exposes_status_defaults_and_route_tool_entries() {
        let mut state = AppState::new();
        state.ui.status_message = Some("bereit".to_string());
        state.ui.show_command_palette = true;
        state.editor.active_tool = fs25_auto_drive_engine::app::EditorTool::Route;
        state
            .editor
            .tool_manager
            .set_active_by_id(fs25_auto_drive_engine::app::tool_contract::RouteToolId::CurveCubic);
        state.editor.default_direction = ConnectionDirection::Dual;
        state.editor.default_priority = ConnectionPriority::SubPriority;

        let chrome = build_host_chrome_snapshot(&state);

        assert_eq!(chrome.status_message.as_deref(), Some("bereit"));
        assert!(chrome.show_command_palette);
        assert_eq!(chrome.active_tool, HostActiveTool::Route);
        assert_eq!(chrome.active_route_tool, Some(HostRouteToolId::CurveCubic));
        assert_eq!(
            chrome.default_direction,
            HostDefaultConnectionDirection::Dual
        );
        assert_eq!(
            chrome.default_priority,
            HostDefaultConnectionPriority::SubPriority
        );

        let defaults_entry = chrome
            .route_tool_entries
            .iter()
            .find(|entry| {
                entry.surface == HostRouteToolSurface::DefaultsPanel
                    && entry.group == HostRouteToolGroup::Basics
                    && entry.tool == HostRouteToolId::CurveCubic
            })
            .expect("Defaults-Panel muss Cubic-Tool-Eintrag enthalten");
        assert!(defaults_entry.enabled);
        assert_eq!(defaults_entry.icon_key, HostRouteToolIconKey::CurveCubic);
        assert!(defaults_entry.disabled_reason.is_none());

        let disabled_analysis_entry = chrome
            .route_tool_entries
            .iter()
            .find(|entry| {
                entry.surface == HostRouteToolSurface::MainMenu
                    && entry.group == HostRouteToolGroup::Analysis
                    && entry.tool == HostRouteToolId::FieldBoundary
            })
            .expect("MainMenu muss Analysis-Eintrag enthalten");
        assert!(!disabled_analysis_entry.enabled);
        assert_eq!(
            disabled_analysis_entry.disabled_reason,
            Some(HostRouteToolDisabledReason::MissingFarmland)
        );
    }

    /// Prüft, dass build_host_chrome_snapshot bei unveraendertem State identische
    /// Schlüsselfelder liefert (Grundlage für zukünftiges Frame-Caching).
    #[test]
    fn build_host_chrome_snapshot_is_idempotent_for_unchanged_state() {
        let mut state = AppState::new();
        state.ui.status_message = Some("bereit".to_string());
        state.editor.default_direction = ConnectionDirection::Dual;

        let first = build_host_chrome_snapshot(&state);
        let second = build_host_chrome_snapshot(&state);

        assert_eq!(first.status_message, second.status_message, "status_message muss stabil sein");
        assert_eq!(first.active_tool, second.active_tool, "active_tool muss stabil sein");
        assert_eq!(first.default_direction, second.default_direction, "default_direction muss stabil sein");
        assert_eq!(first.route_tool_entries.len(), second.route_tool_entries.len(), "route_tool_entries-Laenge muss stabil sein");
        assert_eq!(first.options, second.options, "EditorOptions müssen stabil sein");
    }

    /// Prüft, dass build_host_chrome_snapshot die aktuellen EditorOptions korrekt
    /// überträgt — insbesondere angepasste Werte statt Default-Werte.
    #[test]
    fn build_host_chrome_snapshot_propagates_non_default_options() {
        let mut state = AppState::new();
        state.options.node_size_world = 99.0;
        state.options.snap_scale_percent = 42.0;

        let chrome = build_host_chrome_snapshot(&state);

        assert!(
            (chrome.options.node_size_world - 99.0).abs() < f32::EPSILON,
            "node_size_world muss korrekt übertragen werden, war: {}",
            chrome.options.node_size_world
        );
        assert!(
            (chrome.options.snap_scale_percent - 42.0).abs() < f32::EPSILON,
            "snap_scale_percent muss korrekt übertragen werden, war: {}",
            chrome.options.snap_scale_percent
        );
    }
}
