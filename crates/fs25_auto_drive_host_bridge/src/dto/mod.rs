//! Stabile, host-neutrale DTO-Schicht der fs25_auto_drive_host_bridge.
//!
//! Intern in thematische Submodule aufgeteilt; alle oeffentlichen Items bleiben
//! direkt ueber `crate::dto::*` erreichbar.

mod actions;
mod chrome;
mod dialogs;
mod input;
mod route_tool;
mod ui_json;
mod viewport;

// ─────────────────────────────── Re-Exports ──────────────────────────────────

pub use actions::{HostActiveTool, HostRouteToolAction, HostSessionAction, HostTangentSource};
pub use chrome::HostChromeSnapshot;
pub use dialogs::{HostDialogRequest, HostDialogRequestKind, HostDialogResult};
pub use input::{
    HostInputModifiers, HostPointerButton, HostTapKind, HostViewportInputBatch,
    HostViewportInputEvent,
};
pub use route_tool::{
    HostDefaultConnectionDirection, HostDefaultConnectionPriority, HostRouteToolDisabledReason,
    HostRouteToolEntrySnapshot, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId,
    HostRouteToolSelectionSnapshot, HostRouteToolSurface, HostRouteToolViewportSnapshot,
    HostTangentMenuSnapshot, HostTangentOptionSnapshot,
};
pub use ui_json::{host_ui_snapshot_json, viewport_overlay_snapshot_json};
pub use viewport::{
    HostSelectionSnapshot, HostSessionSnapshot, HostViewportConnectionDirection,
    HostViewportConnectionPriority, HostViewportConnectionSnapshot, HostViewportGeometrySnapshot,
    HostViewportMarkerSnapshot, HostViewportNodeKind, HostViewportNodeSnapshot,
    HostViewportSnapshot,
};

// ───────────────────────── Kompatibilitaetsaliase ────────────────────────────

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineActiveTool = HostActiveTool;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogRequestKind = HostDialogRequestKind;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogRequest = HostDialogRequest;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogResult = HostDialogResult;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EnginePointerButton = HostPointerButton;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineTapKind = HostTapKind;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineInputModifiers = HostInputModifiers;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportInputBatch = HostViewportInputBatch;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportInputEvent = HostViewportInputEvent;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSessionAction = HostSessionAction;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSelectionSnapshot = HostSelectionSnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportSnapshot = HostViewportSnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportGeometrySnapshot = HostViewportGeometrySnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSessionSnapshot = HostSessionSnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDefaultConnectionDirection = HostDefaultConnectionDirection;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDefaultConnectionPriority = HostDefaultConnectionPriority;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineTangentSource = HostTangentSource;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolId = HostRouteToolId;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolAction = HostRouteToolAction;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolGroup = HostRouteToolGroup;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolSurface = HostRouteToolSurface;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolIconKey = HostRouteToolIconKey;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolDisabledReason = HostRouteToolDisabledReason;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolEntrySnapshot = HostRouteToolEntrySnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolSelectionSnapshot = HostRouteToolSelectionSnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineTangentOptionSnapshot = HostTangentOptionSnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineTangentMenuSnapshot = HostTangentMenuSnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolViewportSnapshot = HostRouteToolViewportSnapshot;
/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineChromeSnapshot = HostChromeSnapshot;

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::ui_contract::{BypassPanelAction, RouteToolPanelAction};
    use fs25_auto_drive_engine::shared::{EditorOptions, RenderQuality};
    use serde_json::json;

    use super::{
        EngineActiveTool, EngineChromeSnapshot, EngineDialogRequestKind, EngineDialogResult,
        EngineInputModifiers, EnginePointerButton, EngineRouteToolAction,
        EngineRouteToolViewportSnapshot, EngineSessionAction, EngineSessionSnapshot,
        EngineTangentSource, EngineTapKind, EngineViewportGeometrySnapshot,
        EngineViewportInputBatch, EngineViewportInputEvent, HostActiveTool, HostChromeSnapshot,
        HostDefaultConnectionDirection, HostDefaultConnectionPriority, HostDialogResult,
        HostInputModifiers, HostPointerButton, HostRouteToolAction, HostRouteToolDisabledReason,
        HostRouteToolEntrySnapshot, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId,
        HostRouteToolSelectionSnapshot, HostRouteToolSurface, HostRouteToolViewportSnapshot,
        HostSelectionSnapshot, HostSessionAction, HostSessionSnapshot, HostTangentMenuSnapshot,
        HostTangentOptionSnapshot, HostTangentSource, HostTapKind, HostViewportConnectionDirection,
        HostViewportConnectionPriority, HostViewportConnectionSnapshot,
        HostViewportGeometrySnapshot, HostViewportInputBatch, HostViewportInputEvent,
        HostViewportMarkerSnapshot, HostViewportNodeKind, HostViewportNodeSnapshot,
        HostViewportSnapshot,
    };

    #[test]
    fn engine_session_action_alias_uses_stable_host_json_contract() {
        let action = EngineSessionAction::SetEditorTool {
            tool: EngineActiveTool::Route,
        };

        let payload = serde_json::to_value(&action)
            .expect("SetEditorTool muss als stabiles Host-JSON serialisierbar sein");
        assert_eq!(
            payload,
            json!({ "kind": "set_editor_tool", "tool": "route" })
        );

        let parsed: HostSessionAction = serde_json::from_value(payload)
            .expect("Alias-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostSessionAction::SetEditorTool {
                tool: HostActiveTool::Route,
            }
        );
    }

    #[test]
    fn engine_route_tool_action_alias_roundtrips_with_panel_and_world_payloads() {
        let action = EngineSessionAction::RouteTool {
            action: EngineRouteToolAction::PanelAction {
                action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(3.5)),
            },
        };

        let payload = serde_json::to_value(&action)
            .expect("Route-Tool-Aktion muss als stabiles Host-JSON serialisierbar sein");
        assert_eq!(payload.get("kind"), Some(&json!("route_tool")));

        let parsed: HostSessionAction = serde_json::from_value(payload)
            .expect("Route-Tool-Aktions-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::PanelAction {
                    action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(3.5)),
                },
            }
        );
    }

    #[test]
    fn route_tool_viewport_snapshot_alias_roundtrips_without_schema_drift() {
        let host_snapshot = HostRouteToolViewportSnapshot {
            drag_targets: vec![[1.0, 2.0], [3.0, 4.0]],
            has_pending_input: true,
            segment_shortcuts_active: true,
            tangent_menu_data: Some(HostTangentMenuSnapshot {
                start_options: vec![HostTangentOptionSnapshot {
                    source: HostTangentSource::Connection {
                        neighbor_id: 42,
                        angle: 1.5,
                    },
                    label: "Node #42".to_string(),
                }],
                end_options: vec![HostTangentOptionSnapshot {
                    source: HostTangentSource::None,
                    label: "Manuell".to_string(),
                }],
                current_start: HostTangentSource::Connection {
                    neighbor_id: 42,
                    angle: 1.5,
                },
                current_end: HostTangentSource::None,
            }),
            needs_lasso_input: false,
        };

        let payload = serde_json::to_value(&host_snapshot)
            .expect("Route-Tool-Viewport-Snapshot muss serialisierbar sein");

        let alias_snapshot: EngineRouteToolViewportSnapshot =
            serde_json::from_value(payload.clone())
                .expect("Engine-Viewport-Alias muss kanonisches Host-JSON lesen koennen");
        let canonical_snapshot: HostRouteToolViewportSnapshot = serde_json::from_value(payload)
            .expect("Host-Viewport-Snapshot muss das gleiche JSON lesen koennen");

        assert_eq!(alias_snapshot, host_snapshot);
        assert_eq!(canonical_snapshot, host_snapshot);
    }

    #[test]
    fn tangent_source_alias_roundtrips_with_stable_json_shape() {
        let source = EngineTangentSource::Connection {
            neighbor_id: 7,
            angle: -0.75,
        };

        let payload = serde_json::to_value(source)
            .expect("TangentSource muss als stabiles Host-JSON serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "kind": "connection",
                "neighbor_id": 7,
                "angle": -0.75
            })
        );

        let parsed: HostTangentSource = serde_json::from_value(payload)
            .expect("TangentSource-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostTangentSource::Connection {
                neighbor_id: 7,
                angle: -0.75,
            }
        );
    }

    #[test]
    fn engine_dialog_result_alias_roundtrips_with_host_json_shape() {
        let result = EngineDialogResult::PathSelected {
            kind: EngineDialogRequestKind::BackgroundMap,
            path: "/tmp/overview.zip".to_string(),
        };

        let payload = serde_json::to_value(&result)
            .expect("Dialog-Ergebnis muss als stabiles Host-JSON serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "status": "path_selected",
                "kind": "background_map",
                "path": "/tmp/overview.zip"
            })
        );

        let parsed: HostDialogResult = serde_json::from_value(payload)
            .expect("Alias-Dialog-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostDialogResult::PathSelected {
                kind: EngineDialogRequestKind::BackgroundMap,
                path: "/tmp/overview.zip".to_string(),
            }
        );
    }

    #[test]
    fn engine_session_snapshot_alias_roundtrips_without_schema_drift() {
        let host_snapshot = HostSessionSnapshot {
            has_map: true,
            node_count: 7,
            connection_count: 9,
            active_tool: HostActiveTool::Connect,
            status_message: Some("bereit".to_string()),
            show_command_palette: true,
            show_options_dialog: false,
            can_undo: true,
            can_redo: false,
            pending_dialog_request_count: 2,
            selection: HostSelectionSnapshot {
                selected_node_ids: vec![11, 42],
            },
            viewport: HostViewportSnapshot {
                camera_position: [12.5, -8.0],
                zoom: 1.25,
            },
        };

        let payload = serde_json::to_value(&host_snapshot)
            .expect("HostSnapshot muss fuer den Alias-Contract serialisierbar sein");

        let alias_snapshot: EngineSessionSnapshot = serde_json::from_value(payload.clone())
            .expect("EngineSessionSnapshot-Alias muss kanonisches Host-JSON lesen koennen");
        let canonical_snapshot: HostSessionSnapshot = serde_json::from_value(payload)
            .expect("HostSessionSnapshot muss das gleiche JSON lesen koennen");

        assert_eq!(alias_snapshot, host_snapshot);
        assert_eq!(canonical_snapshot, host_snapshot);
    }

    #[test]
    fn engine_chrome_snapshot_alias_roundtrips_with_route_tool_metadata() {
        let host_snapshot = HostChromeSnapshot {
            status_message: Some("bereit".to_string()),
            show_command_palette: true,
            show_options_dialog: false,
            has_map: true,
            has_selection: true,
            has_clipboard: false,
            can_undo: true,
            can_redo: false,
            active_tool: HostActiveTool::Route,
            active_route_tool: Some(HostRouteToolId::CurveCubic),
            default_direction: HostDefaultConnectionDirection::Dual,
            default_priority: HostDefaultConnectionPriority::SubPriority,
            route_tool_memory: HostRouteToolSelectionSnapshot {
                basics: HostRouteToolId::CurveCubic,
                section: HostRouteToolId::Bypass,
                analysis: HostRouteToolId::FieldBoundary,
            },
            options: EditorOptions::default(),
            route_tool_entries: vec![
                HostRouteToolEntrySnapshot {
                    surface: HostRouteToolSurface::DefaultsPanel,
                    group: HostRouteToolGroup::Basics,
                    tool: HostRouteToolId::CurveCubic,
                    slot: 2,
                    icon_key: HostRouteToolIconKey::CurveCubic,
                    enabled: true,
                    disabled_reason: None,
                },
                HostRouteToolEntrySnapshot {
                    surface: HostRouteToolSurface::MainMenu,
                    group: HostRouteToolGroup::Analysis,
                    tool: HostRouteToolId::FieldPath,
                    slot: 8,
                    icon_key: HostRouteToolIconKey::FieldPath,
                    enabled: false,
                    disabled_reason: Some(HostRouteToolDisabledReason::MissingFarmland),
                },
            ],
            node_count: 0,
            connection_count: 0,
            marker_count: 0,
            map_name: None,
            camera_zoom: 1.0,
            camera_position: [0.0, 0.0],
            heightmap_path: None,
            selection_count: 0,
            selection_example_id: None,
            background_map_loaded: false,
            render_quality: RenderQuality::High,
            has_farmland: false,
            background_visible: true,
            background_scale: 1.0,
        };

        let payload = serde_json::to_value(&host_snapshot)
            .expect("Chrome-Snapshot muss fuer den Alias-Contract serialisierbar sein");
        let payload_obj = payload
            .as_object()
            .expect("Chrome-Snapshot muss als JSON-Objekt serialisiert werden");
        assert_eq!(payload_obj.get("active_tool"), Some(&json!("route")));
        assert_eq!(
            payload_obj.get("active_route_tool"),
            Some(&json!("curve_cubic"))
        );
        assert_eq!(payload_obj.get("default_direction"), Some(&json!("dual")));
        assert_eq!(
            payload_obj.get("default_priority"),
            Some(&json!("sub_priority"))
        );

        let route_tool_entries = payload_obj
            .get("route_tool_entries")
            .and_then(|entries| entries.as_array())
            .expect("Route-Tool-Eintraege muessen als JSON-Array serialisiert werden");
        assert_eq!(route_tool_entries.len(), 2);
        assert_eq!(
            route_tool_entries[0],
            json!({
                "surface": "defaults_panel",
                "group": "basics",
                "tool": "curve_cubic",
                "slot": 2,
                "icon_key": "curve_cubic",
                "enabled": true,
                "disabled_reason": null
            })
        );
        assert_eq!(
            route_tool_entries[1],
            json!({
                "surface": "main_menu",
                "group": "analysis",
                "tool": "field_path",
                "slot": 8,
                "icon_key": "field_path",
                "enabled": false,
                "disabled_reason": "missing_farmland"
            })
        );

        let options = payload_obj
            .get("options")
            .and_then(|options| options.as_object())
            .expect("Optionen muessen als JSON-Objekt serialisiert werden");
        assert_eq!(options.get("language"), Some(&json!("De")));

        let alias_snapshot: EngineChromeSnapshot = serde_json::from_value(payload.clone())
            .expect("EngineChromeSnapshot-Alias muss kanonisches Host-JSON lesen koennen");
        let canonical_snapshot: HostChromeSnapshot = serde_json::from_value(payload)
            .expect("HostChromeSnapshot muss das gleiche JSON lesen koennen");

        assert_eq!(alias_snapshot.route_tool_entries.len(), 2);
        assert_eq!(canonical_snapshot.route_tool_entries.len(), 2);
        assert!(alias_snapshot.show_command_palette);
        assert_eq!(
            canonical_snapshot.default_direction,
            HostDefaultConnectionDirection::Dual
        );
        assert_eq!(
            alias_snapshot.options.language,
            host_snapshot.options.language
        );
    }

    #[test]
    fn viewport_input_batch_roundtrips_with_stable_json_shape() {
        let batch = HostViewportInputBatch {
            events: vec![
                HostViewportInputEvent::Resize {
                    size_px: [1280.0, 720.0],
                },
                HostViewportInputEvent::Tap {
                    button: HostPointerButton::Primary,
                    tap_kind: HostTapKind::Single,
                    screen_pos: [32.0, 48.0],
                    modifiers: HostInputModifiers {
                        shift: true,
                        alt: false,
                        command: true,
                    },
                },
                HostViewportInputEvent::Tap {
                    button: HostPointerButton::Primary,
                    tap_kind: HostTapKind::Double,
                    screen_pos: [64.0, 96.0],
                    modifiers: HostInputModifiers::default(),
                },
                HostViewportInputEvent::DragEnd {
                    button: HostPointerButton::Secondary,
                    screen_pos: None,
                },
                HostViewportInputEvent::Scroll {
                    screen_pos: Some([300.0, 200.0]),
                    smooth_delta_y: 12.0,
                    raw_delta_y: 1.0,
                    modifiers: HostInputModifiers::default(),
                },
            ],
        };

        let payload = serde_json::to_value(&batch)
            .expect("Viewport-Input-Batch muss stabil serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "events": [
                    { "kind": "resize", "size_px": [1280.0, 720.0] },
                    {
                        "kind": "tap",
                        "button": "primary",
                        "tap_kind": "single",
                        "screen_pos": [32.0, 48.0],
                        "modifiers": {
                            "shift": true,
                            "alt": false,
                            "command": true
                        }
                    },
                    {
                        "kind": "tap",
                        "button": "primary",
                        "tap_kind": "double",
                        "screen_pos": [64.0, 96.0],
                        "modifiers": {
                            "shift": false,
                            "alt": false,
                            "command": false
                        }
                    },
                    {
                        "kind": "drag_end",
                        "button": "secondary",
                        "screen_pos": null
                    },
                    {
                        "kind": "scroll",
                        "screen_pos": [300.0, 200.0],
                        "smooth_delta_y": 12.0,
                        "raw_delta_y": 1.0,
                        "modifiers": {
                            "shift": false,
                            "alt": false,
                            "command": false
                        }
                    }
                ]
            })
        );

        let parsed: HostViewportInputBatch =
            serde_json::from_value(payload).expect("Viewport-Input-Batch muss wieder lesbar sein");
        assert_eq!(parsed, batch);
    }

    #[test]
    fn engine_viewport_input_alias_roundtrips_with_canonical_host_contract() {
        let action = EngineSessionAction::SubmitViewportInput {
            batch: EngineViewportInputBatch {
                events: vec![EngineViewportInputEvent::Tap {
                    button: EnginePointerButton::Primary,
                    tap_kind: EngineTapKind::Double,
                    screen_pos: [10.0, 20.0],
                    modifiers: EngineInputModifiers {
                        shift: false,
                        alt: false,
                        command: true,
                    },
                }],
            },
        };

        let payload = serde_json::to_value(&action)
            .expect("Viewport-Input-Alias muss stabil serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "kind": "submit_viewport_input",
                "batch": {
                    "events": [{
                        "kind": "tap",
                        "button": "primary",
                        "tap_kind": "double",
                        "screen_pos": [10.0, 20.0],
                        "modifiers": {
                            "shift": false,
                            "alt": false,
                            "command": true
                        }
                    }]
                }
            })
        );

        let parsed: HostSessionAction = serde_json::from_value(payload)
            .expect("Alias-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![HostViewportInputEvent::Tap {
                        button: HostPointerButton::Primary,
                        tap_kind: HostTapKind::Double,
                        screen_pos: [10.0, 20.0],
                        modifiers: HostInputModifiers {
                            shift: false,
                            alt: false,
                            command: true,
                        },
                    }],
                },
            }
        );
    }

    #[test]
    fn engine_viewport_geometry_snapshot_alias_roundtrips_without_schema_drift() {
        let host_snapshot = HostViewportGeometrySnapshot {
            has_map: true,
            viewport_size: [1280.0, 720.0],
            camera_position: [32.0, -16.0],
            zoom: 1.5,
            world_per_pixel: 0.75,
            has_background: true,
            background_visible: true,
            nodes: vec![HostViewportNodeSnapshot {
                id: 7,
                position: [10.0, 20.0],
                kind: HostViewportNodeKind::Warning,
                preserve_when_decimating: true,
                selected: true,
                hidden: false,
                dimmed: false,
            }],
            connections: vec![HostViewportConnectionSnapshot {
                start_id: 7,
                end_id: 8,
                start_position: [10.0, 20.0],
                end_position: [15.0, 25.0],
                direction: HostViewportConnectionDirection::Dual,
                priority: HostViewportConnectionPriority::SubPriority,
                hidden: false,
                dimmed: true,
            }],
            markers: vec![HostViewportMarkerSnapshot {
                position: [12.0, 18.0],
            }],
        };

        let payload = serde_json::to_value(&host_snapshot)
            .expect("Geometry-Snapshot muss fuer den Alias-Contract serialisierbar sein");

        let alias_snapshot: EngineViewportGeometrySnapshot = serde_json::from_value(
            payload.clone(),
        )
        .expect("EngineViewportGeometrySnapshot-Alias muss kanonisches Host-JSON lesen koennen");
        let canonical_snapshot: HostViewportGeometrySnapshot = serde_json::from_value(payload)
            .expect("HostViewportGeometrySnapshot muss das gleiche JSON lesen koennen");

        assert_eq!(alias_snapshot, host_snapshot);
        assert_eq!(canonical_snapshot, host_snapshot);
    }
}
