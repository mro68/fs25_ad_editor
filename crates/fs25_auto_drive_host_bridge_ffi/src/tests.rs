use super::{
    fs25ad_host_bridge_abi_version, fs25ad_host_bridge_last_error_message,
    fs25ad_host_bridge_session_apply_action_json, fs25ad_host_bridge_session_chrome_snapshot_json,
    fs25ad_host_bridge_session_connection_pair_json,
    fs25ad_host_bridge_session_context_menu_snapshot_json,
    fs25ad_host_bridge_session_dialog_snapshot_json, fs25ad_host_bridge_session_dispose,
    fs25ad_host_bridge_session_editing_snapshot_json, fs25ad_host_bridge_session_is_dirty,
    fs25ad_host_bridge_session_marker_list_json, fs25ad_host_bridge_session_new,
    fs25ad_host_bridge_session_node_details_json,
    fs25ad_host_bridge_session_route_tool_viewport_json, fs25ad_host_bridge_session_snapshot_json,
    fs25ad_host_bridge_session_submit_dialog_result_json,
    fs25ad_host_bridge_session_take_dialog_requests_json,
    fs25ad_host_bridge_session_ui_snapshot_json, fs25ad_host_bridge_session_viewport_geometry_json,
    fs25ad_host_bridge_session_viewport_overlay_json, fs25ad_host_bridge_string_free,
    FS25AD_HOST_BRIDGE_ABI_VERSION,
};
use fs25_auto_drive_host_bridge::{
    HostActiveTool, HostConnectionPairSnapshot, HostContextMenuSnapshot, HostContextMenuVariant,
    HostDialogRequest, HostDialogRequestKind, HostDialogResult, HostDialogSnapshot,
    HostEditingSnapshot, HostInputModifiers, HostMarkerListSnapshot, HostPointerButton,
    HostRouteToolAction, HostRouteToolId, HostRouteToolViewportSnapshot, HostSessionAction,
    HostSessionSnapshot, HostTangentSource, HostTapKind, HostViewportGeometrySnapshot,
    HostViewportInputBatch, HostViewportInputEvent,
};
use std::ffi::{CStr, CString};

// Sicherheits-Wrapper fuer unsafe FFI-Funktionen im Testkontext.
fn string_free(ptr: *mut std::ffi::c_char) {
    unsafe { fs25ad_host_bridge_string_free(ptr) }
}
fn session_dispose(s: *mut super::HostBridgeSessionHandle) {
    unsafe { fs25ad_host_bridge_session_dispose(s) }
}
fn session_snapshot_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_snapshot_json(s) }
}
fn session_chrome_snapshot_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_chrome_snapshot_json(s) }
}
fn session_node_details_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_node_details_json(s) }
}
fn session_marker_list_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_marker_list_json(s) }
}
fn session_connection_pair_json(
    s: *mut super::HostBridgeSessionHandle,
    node_a: u64,
    node_b: u64,
) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_connection_pair_json(s, node_a, node_b) }
}
fn session_is_dirty(s: *mut super::HostBridgeSessionHandle) -> i32 {
    unsafe { fs25ad_host_bridge_session_is_dirty(s) }
}
fn session_ui_snapshot_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_ui_snapshot_json(s) }
}
fn session_dialog_snapshot_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_dialog_snapshot_json(s) }
}
fn session_editing_snapshot_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_editing_snapshot_json(s) }
}
fn session_context_menu_snapshot_json(
    s: *mut super::HostBridgeSessionHandle,
    focus_node_id_or_neg1: i64,
) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_context_menu_snapshot_json(s, focus_node_id_or_neg1) }
}
fn session_route_tool_viewport_json(
    s: *mut super::HostBridgeSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_route_tool_viewport_json(s) }
}
fn session_apply_action_json(
    s: *mut super::HostBridgeSessionHandle,
    j: *const std::ffi::c_char,
) -> bool {
    unsafe { fs25ad_host_bridge_session_apply_action_json(s, j) }
}
fn session_take_dialog_requests_json(
    s: *mut super::HostBridgeSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_take_dialog_requests_json(s) }
}
fn session_submit_dialog_result_json(
    s: *mut super::HostBridgeSessionHandle,
    j: *const std::ffi::c_char,
) -> bool {
    unsafe { fs25ad_host_bridge_session_submit_dialog_result_json(s, j) }
}
fn session_viewport_geometry_json(
    s: *mut super::HostBridgeSessionHandle,
    w: f32,
    h: f32,
) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_viewport_geometry_json(s, w, h) }
}
fn session_viewport_overlay_json(
    s: *mut super::HostBridgeSessionHandle,
    x: f32,
    y: f32,
) -> *mut std::ffi::c_char {
    unsafe { fs25ad_host_bridge_session_viewport_overlay_json(s, x, y) }
}

#[cfg(feature = "flutter")]
fn flutter_session_dispose(session: *mut super::flutter_api::FlutterSessionHandle) {
    unsafe { super::fs25ad_flutter_session_dispose(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_apply_action_json(
    session: *const super::flutter_api::FlutterSessionHandle,
    json: *const std::ffi::c_char,
) -> bool {
    unsafe { super::fs25ad_flutter_session_apply_action_json(session, json) }
}

#[cfg(feature = "flutter")]
fn flutter_session_take_dialog_requests_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_take_dialog_requests_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_submit_dialog_result_json(
    session: *const super::flutter_api::FlutterSessionHandle,
    json: *const std::ffi::c_char,
) -> bool {
    unsafe { super::fs25ad_flutter_session_submit_dialog_result_json(session, json) }
}

#[cfg(feature = "flutter")]
fn flutter_session_snapshot_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_snapshot_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_node_details_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_node_details_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_marker_list_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_marker_list_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_route_tool_viewport_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_route_tool_viewport_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_connection_pair_json(
    session: *const super::flutter_api::FlutterSessionHandle,
    node_a: u64,
    node_b: u64,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_connection_pair_json(session, node_a, node_b) }
}

#[cfg(feature = "flutter")]
fn flutter_session_is_dirty(session: *const super::flutter_api::FlutterSessionHandle) -> i32 {
    unsafe { super::fs25ad_flutter_session_is_dirty(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_ui_snapshot_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_ui_snapshot_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_chrome_snapshot_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_chrome_snapshot_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_dialog_snapshot_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_dialog_snapshot_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_editing_snapshot_json(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> *mut std::ffi::c_char {
    unsafe { super::fs25ad_flutter_session_editing_snapshot_json(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_context_menu_snapshot_json(
    session: *const super::flutter_api::FlutterSessionHandle,
    focus_node_id_or_neg1: i64,
) -> *mut std::ffi::c_char {
    unsafe {
        super::fs25ad_flutter_session_context_menu_snapshot_json(session, focus_node_id_or_neg1)
    }
}

#[cfg(feature = "flutter")]
fn flutter_session_viewport_overlay_json(
    session: *const super::flutter_api::FlutterSessionHandle,
    cursor_world_x: f32,
    cursor_world_y: f32,
) -> *mut std::ffi::c_char {
    unsafe {
        super::fs25ad_flutter_session_viewport_overlay_json(session, cursor_world_x, cursor_world_y)
    }
}

#[cfg(feature = "flutter")]
fn flutter_session_viewport_geometry_json(
    session: *const super::flutter_api::FlutterSessionHandle,
    viewport_width: f32,
    viewport_height: f32,
) -> *mut std::ffi::c_char {
    unsafe {
        super::fs25ad_flutter_session_viewport_geometry_json(
            session,
            viewport_width,
            viewport_height,
        )
    }
}

#[cfg(feature = "flutter")]
fn flutter_session_acquire_shared_arc_raw(
    session: *const super::flutter_api::FlutterSessionHandle,
) -> i64 {
    unsafe { super::fs25ad_flutter_session_acquire_shared_arc_raw(session) }
}

#[cfg(feature = "flutter")]
fn flutter_session_release_shared_arc_raw(raw: i64) {
    unsafe { super::fs25ad_flutter_session_release_shared_arc_raw(raw) }
}

fn read_and_free_string(ptr: *mut std::ffi::c_char) -> String {
    assert!(!ptr.is_null());
    let value = unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("FFI string must be valid UTF-8")
        .to_string();
    string_free(ptr);
    value
}

fn apply_action_json(session: *mut super::HostBridgeSessionHandle, action: HostSessionAction) {
    let action_json =
        CString::new(serde_json::to_string(&action).expect("action JSON must serialize"))
            .expect("CString must build");

    if session_apply_action_json(session, action_json.as_ptr()) {
        return;
    }

    let error = read_and_free_string(fs25ad_host_bridge_last_error_message());
    panic!("HostSessionAction failed unexpectedly: {error}");
}

#[cfg(feature = "flutter")]
fn assert_last_error_contains(expected: &str) {
    let error = read_and_free_string(fs25ad_host_bridge_last_error_message());
    assert!(
        error.contains(expected),
        "expected last error to contain '{expected}', got '{error}'"
    );
}

#[cfg(feature = "flutter")]
fn apply_action_json_flutter(
    session: *const super::flutter_api::FlutterSessionHandle,
    action: HostSessionAction,
) {
    let action_json =
        CString::new(serde_json::to_string(&action).expect("action JSON must serialize"))
            .expect("CString must build");

    if flutter_session_apply_action_json(session, action_json.as_ptr()) {
        return;
    }

    let error = read_and_free_string(fs25ad_host_bridge_last_error_message());
    panic!("Flutter HostSessionAction failed unexpectedly: {error}");
}

#[test]
fn ffi_transport_reports_stable_abi_version() {
    assert_eq!(
        fs25ad_host_bridge_abi_version(),
        FS25AD_HOST_BRIDGE_ABI_VERSION
    );
    assert_eq!(fs25ad_host_bridge_abi_version(), 4);
}

#[cfg(feature = "flutter")]
#[test]
fn ffi_flutter_session_lifecycle_roundtrip() {
    let session = super::fs25ad_flutter_session_new();
    assert!(
        !session.is_null(),
        "Flutter-Session-FFI muss einen gueltigen Handle liefern"
    );
    flutter_session_dispose(session);
}

#[cfg(feature = "flutter")]
#[test]
fn ffi_flutter_session_roundtrips_apply_action_and_snapshot_reads() {
    let session = super::fs25ad_flutter_session_new();
    assert!(!session.is_null());

    apply_action_json_flutter(session, HostSessionAction::ToggleCommandPalette);

    let snapshot_json = read_and_free_string(flutter_session_snapshot_json(session));
    let snapshot: HostSessionSnapshot =
        serde_json::from_str(&snapshot_json).expect("flutter snapshot JSON must parse");
    assert!(snapshot.show_command_palette);

    let chrome_json = read_and_free_string(flutter_session_chrome_snapshot_json(session));
    let chrome_snapshot: fs25_auto_drive_host_bridge::HostChromeSnapshot =
        serde_json::from_str(&chrome_json).expect("flutter chrome JSON must parse");
    assert!(chrome_snapshot.show_command_palette);
    assert!(!chrome_snapshot.background_layers_available);
    assert!(chrome_snapshot.background_layer_entries.is_empty());

    flutter_session_dispose(session);
}

#[cfg(feature = "flutter")]
#[test]
fn ffi_flutter_session_is_dirty_reports_zero_for_fresh_session() {
    let session = super::fs25ad_flutter_session_new();
    assert!(!session.is_null());

    assert_eq!(flutter_session_is_dirty(session), 0);

    flutter_session_dispose(session);
}

#[cfg(feature = "flutter")]
#[test]
fn ffi_flutter_session_acquire_and_release_shared_arc_raw_roundtrip() {
    let session = super::fs25ad_flutter_session_new();
    assert!(!session.is_null());

    let raw = flutter_session_acquire_shared_arc_raw(session);
    assert_ne!(raw, 0, "shared Arc raw pointer must not be zero");

    flutter_session_release_shared_arc_raw(raw);
    flutter_session_dispose(session);
}

#[cfg(feature = "flutter")]
#[test]
fn ffi_flutter_session_rejects_null_pointers() {
    let action_json = CString::new(
        serde_json::to_string(&HostSessionAction::ToggleCommandPalette)
            .expect("action JSON must serialize"),
    )
    .expect("CString must build");
    let result_json = CString::new(
        serde_json::to_string(&HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::Heightmap,
            path: "/tmp/test_heightmap.png".to_string(),
        })
        .expect("dialog result JSON must serialize"),
    )
    .expect("CString must build");

    assert!(!flutter_session_apply_action_json(
        std::ptr::null(),
        action_json.as_ptr(),
    ));
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_take_dialog_requests_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(!flutter_session_submit_dialog_result_json(
        std::ptr::null(),
        result_json.as_ptr(),
    ));
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_snapshot_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_node_details_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_marker_list_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_route_tool_viewport_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_connection_pair_json(std::ptr::null(), 7, 9).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert_eq!(flutter_session_is_dirty(std::ptr::null()), -1);
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_ui_snapshot_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_chrome_snapshot_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_dialog_snapshot_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_editing_snapshot_json(std::ptr::null()).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_context_menu_snapshot_json(std::ptr::null(), -1).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_viewport_overlay_json(std::ptr::null(), 0.0, 0.0).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert!(flutter_session_viewport_geometry_json(std::ptr::null(), 320.0, 200.0).is_null());
    assert_last_error_contains("FlutterSessionHandle pointer");

    assert_eq!(flutter_session_acquire_shared_arc_raw(std::ptr::null()), 0);
    assert_last_error_contains("FlutterSessionHandle pointer");

    let session = super::fs25ad_flutter_session_new();
    assert!(!session.is_null());

    assert!(!flutter_session_apply_action_json(
        session,
        std::ptr::null()
    ));
    assert_last_error_contains("HostSessionAction JSON pointer must not be null");

    assert!(!flutter_session_submit_dialog_result_json(
        session,
        std::ptr::null(),
    ));
    assert_last_error_contains("HostDialogResult JSON pointer must not be null");

    flutter_session_release_shared_arc_raw(0);
    assert!(fs25ad_host_bridge_last_error_message().is_null());

    flutter_session_dispose(session);
}

#[test]
fn ffi_transport_roundtrips_session_actions_dialogs_and_snapshots() {
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    let action_json = CString::new(
        serde_json::to_string(&HostSessionAction::ToggleCommandPalette)
            .expect("action JSON must serialize"),
    )
    .expect("CString must build");
    assert!(session_apply_action_json(session, action_json.as_ptr()));

    let snapshot_json = read_and_free_string(session_snapshot_json(session));
    let snapshot: HostSessionSnapshot =
        serde_json::from_str(&snapshot_json).expect("snapshot JSON must parse");
    assert!(snapshot.show_command_palette);

    let chrome_snapshot_json = read_and_free_string(session_chrome_snapshot_json(session));
    let chrome_snapshot: fs25_auto_drive_host_bridge::HostChromeSnapshot =
        serde_json::from_str(&chrome_snapshot_json).expect("chrome snapshot JSON must parse");
    assert!(chrome_snapshot.show_command_palette);
    assert_eq!(chrome_snapshot.status_message, None);
    assert!(!chrome_snapshot.background_layers_available);
    assert!(chrome_snapshot.background_layer_entries.is_empty());

    let route_tool_viewport_json = read_and_free_string(session_route_tool_viewport_json(session));
    let route_tool_viewport: HostRouteToolViewportSnapshot =
        serde_json::from_str(&route_tool_viewport_json)
            .expect("route tool viewport JSON must parse");
    assert!(!route_tool_viewport.has_pending_input);
    assert!(route_tool_viewport.drag_targets.is_empty());

    let request_action_json = CString::new(
        serde_json::to_string(&HostSessionAction::RequestHeightmapSelection)
            .expect("dialog action JSON must serialize"),
    )
    .expect("CString must build");
    assert!(session_apply_action_json(
        session,
        request_action_json.as_ptr()
    ));

    let requests_json = read_and_free_string(session_take_dialog_requests_json(session));
    let requests: Vec<HostDialogRequest> =
        serde_json::from_str(&requests_json).expect("dialog request JSON must parse");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].kind, HostDialogRequestKind::Heightmap);

    let result_json = CString::new(
        serde_json::to_string(&HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::Heightmap,
            path: "/tmp/test_heightmap.png".to_string(),
        })
        .expect("dialog result JSON must serialize"),
    )
    .expect("CString must build");
    assert!(session_submit_dialog_result_json(
        session,
        result_json.as_ptr()
    ));

    let geometry_json = read_and_free_string(session_viewport_geometry_json(session, 800.0, 600.0));
    let geometry: HostViewportGeometrySnapshot =
        serde_json::from_str(&geometry_json).expect("geometry JSON must parse");
    assert_eq!(geometry.viewport_size, [800.0, 600.0]);

    let viewport_input_json = CString::new(
        serde_json::to_string(&HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![
                    HostViewportInputEvent::Resize {
                        size_px: [1024.0, 768.0],
                    },
                    HostViewportInputEvent::Scroll {
                        screen_pos: Some([512.0, 384.0]),
                        smooth_delta_y: 1.0,
                        raw_delta_y: 0.0,
                        modifiers: HostInputModifiers::default(),
                    },
                    HostViewportInputEvent::Tap {
                        button: HostPointerButton::Primary,
                        tap_kind: HostTapKind::Double,
                        screen_pos: [512.0, 384.0],
                        modifiers: HostInputModifiers::default(),
                    },
                ],
            },
        })
        .expect("viewport input JSON must serialize"),
    )
    .expect("CString must build");
    assert!(session_apply_action_json(
        session,
        viewport_input_json.as_ptr()
    ));

    let viewport_snapshot_json = read_and_free_string(session_snapshot_json(session));
    let viewport_snapshot: HostSessionSnapshot = serde_json::from_str(&viewport_snapshot_json)
        .expect("snapshot JSON after viewport input must parse");
    assert!(viewport_snapshot.viewport.zoom > 1.0);

    let updated_geometry_json =
        read_and_free_string(session_viewport_geometry_json(session, 1024.0, 768.0));
    let updated_geometry: HostViewportGeometrySnapshot =
        serde_json::from_str(&updated_geometry_json).expect("updated geometry JSON must parse");
    assert_eq!(updated_geometry.viewport_size, [1024.0, 768.0]);
    assert!(updated_geometry.zoom > 1.0);

    session_dispose(session);
}

#[test]
fn ffi_transport_reports_errors_for_invalid_json() {
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    let invalid_json = CString::new("{not valid json}").expect("CString must build");
    assert!(!session_apply_action_json(session, invalid_json.as_ptr()));

    let error = read_and_free_string(fs25ad_host_bridge_last_error_message());
    assert!(error.contains("HostSessionAction"));

    session_dispose(session);
}

#[test]
fn ffi_transport_roundtrips_generic_read_endpoints() {
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    assert_eq!(session_is_dirty(session), 0);

    let marker_list_json = read_and_free_string(session_marker_list_json(session));
    let marker_list: HostMarkerListSnapshot =
        serde_json::from_str(&marker_list_json).expect("marker list JSON must parse");
    assert!(marker_list.markers.is_empty());
    assert!(marker_list.groups.is_empty());

    let connection_pair_json = read_and_free_string(session_connection_pair_json(session, 7, 9));
    let connection_pair: HostConnectionPairSnapshot =
        serde_json::from_str(&connection_pair_json).expect("connection pair JSON must parse");
    assert_eq!(connection_pair.node_a, 7);
    assert_eq!(connection_pair.node_b, 9);
    assert!(connection_pair.connections.is_empty());

    let ui_json = read_and_free_string(session_ui_snapshot_json(session));
    let ui_value: serde_json::Value =
        serde_json::from_str(&ui_json).expect("ui snapshot JSON must parse");
    assert!(ui_value.get("panels").is_some());

    let dialog_json = read_and_free_string(session_dialog_snapshot_json(session));
    let dialog_snapshot: HostDialogSnapshot =
        serde_json::from_str(&dialog_json).expect("dialog snapshot JSON must parse");
    assert!(!dialog_snapshot.heightmap_warning.visible);

    let editing_json = read_and_free_string(session_editing_snapshot_json(session));
    let editing_snapshot: HostEditingSnapshot =
        serde_json::from_str(&editing_json).expect("editing snapshot JSON must parse");
    assert!(editing_snapshot.editable_groups.is_empty());
    assert!(!editing_snapshot.resample.active);

    let context_menu_json = read_and_free_string(session_context_menu_snapshot_json(session, -1));
    let context_menu_snapshot: HostContextMenuSnapshot =
        serde_json::from_str(&context_menu_json).expect("context menu snapshot JSON must parse");
    assert_eq!(
        context_menu_snapshot.variant,
        HostContextMenuVariant::EmptyArea
    );
    assert!(context_menu_snapshot.available_actions.is_empty());

    let overlay_json = read_and_free_string(session_viewport_overlay_json(session, 0.0, 0.0));
    let overlay_value: serde_json::Value =
        serde_json::from_str(&overlay_json).expect("overlay snapshot JSON must parse");
    assert!(overlay_value.get("show_no_file_hint").is_some());

    session_dispose(session);
}

#[test]
fn ffi_transport_node_details_returns_null_without_error_when_no_node_is_inspected() {
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    let action_json = CString::new(
        serde_json::json!({
            "kind": "query_node_details",
            "node_id": 99,
        })
        .to_string(),
    )
    .expect("CString must build");
    assert!(session_apply_action_json(session, action_json.as_ptr()));

    assert!(session_node_details_json(session).is_null());
    assert!(fs25ad_host_bridge_last_error_message().is_null());

    session_dispose(session);
}

#[test]
fn ffi_transport_rejects_invalid_context_menu_focus_node_id() {
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    assert!(session_context_menu_snapshot_json(session, -2).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message()).contains("focus_node_id")
    );

    session_dispose(session);
}

#[test]
fn ffi_transport_rejects_null_session_and_payload_pointers() {
    assert!(session_snapshot_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_chrome_snapshot_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_node_details_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_marker_list_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_connection_pair_json(std::ptr::null_mut(), 1, 2).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert_eq!(session_is_dirty(std::ptr::null_mut()), -1);
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_ui_snapshot_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_dialog_snapshot_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_editing_snapshot_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_context_menu_snapshot_json(std::ptr::null_mut(), -1).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_route_tool_viewport_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_viewport_overlay_json(std::ptr::null_mut(), 0.0, 0.0).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_take_dialog_requests_json(std::ptr::null_mut()).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    assert!(session_viewport_geometry_json(std::ptr::null_mut(), 320.0, 200.0).is_null());
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostBridgeSession pointer")
    );

    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    assert!(!session_apply_action_json(session, std::ptr::null()));
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostSessionAction JSON pointer must not be null")
    );

    assert!(!session_submit_dialog_result_json(
        session,
        std::ptr::null()
    ));
    assert!(
        read_and_free_string(fs25ad_host_bridge_last_error_message())
            .contains("HostDialogResult JSON pointer must not be null")
    );

    session_dispose(session);
}

#[test]
fn ffi_route_tool_write_actions_cover_anchors_click_drag_lasso_and_related_json_paths() {
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    apply_action_json(session, HostSessionAction::OpenFile);

    let requests_json = read_and_free_string(session_take_dialog_requests_json(session));
    let requests: Vec<HostDialogRequest> =
        serde_json::from_str(&requests_json).expect("dialog request JSON must parse");
    assert!(!requests.is_empty());
    assert_eq!(requests[0].kind, HostDialogRequestKind::OpenFile);

    let test_map_path = format!(
        "{}/../../ad_sample_data/AutoDrive_config-test.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    let open_result_json = CString::new(
        serde_json::to_string(&HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::OpenFile,
            path: test_map_path,
        })
        .expect("dialog result JSON must serialize"),
    )
    .expect("CString must build");
    assert!(session_submit_dialog_result_json(
        session,
        open_result_json.as_ptr()
    ));

    let geometry_json =
        read_and_free_string(session_viewport_geometry_json(session, 1024.0, 768.0));
    let geometry: HostViewportGeometrySnapshot =
        serde_json::from_str(&geometry_json).expect("geometry JSON must parse");
    assert!(geometry.nodes.len() >= 2);

    let start_id = geometry.nodes[0].id;
    let end_id = geometry.nodes[1].id;
    let start_pos = geometry.nodes[0].position;
    let end_pos = geometry.nodes[1].position;

    apply_action_json(
        session,
        HostSessionAction::SetEditorTool {
            tool: HostActiveTool::Route,
        },
    );

    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::SelectToolWithAnchors {
                tool: HostRouteToolId::CurveCubic,
                start_node_id: start_id,
                end_node_id: end_id,
            },
        },
    );

    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::Click {
                world_pos: start_pos,
                ctrl: false,
            },
        },
    );
    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragStart {
                world_pos: start_pos,
            },
        },
    );
    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragUpdate {
                world_pos: [
                    (start_pos[0] + end_pos[0]) * 0.5,
                    (start_pos[1] + end_pos[1]) * 0.5,
                ],
            },
        },
    );
    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragEnd,
        },
    );

    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::LassoCompleted {
                polygon: vec![
                    start_pos,
                    [end_pos[0], start_pos[1]],
                    end_pos,
                    [start_pos[0], end_pos[1]],
                ],
            },
        },
    );

    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::ApplyTangent {
                start: HostTangentSource::None,
                end: HostTangentSource::None,
            },
        },
    );
    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::ScrollRotate { delta: 1.0 },
        },
    );
    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::IncreaseNodeCount,
        },
    );
    apply_action_json(
        session,
        HostSessionAction::RouteTool {
            action: HostRouteToolAction::DecreaseSegmentLength,
        },
    );

    let viewport_json = read_and_free_string(session_route_tool_viewport_json(session));
    let viewport_snapshot: HostRouteToolViewportSnapshot =
        serde_json::from_str(&viewport_json).expect("route tool viewport JSON must parse");
    assert!(viewport_snapshot.has_pending_input);

    session_dispose(session);
}

#[test]
fn ffi_e2e_layer_toggle_positive_flow_with_stable_snapshot_assertions() {
    /// E2E-Test fuer Layer-Toggle ueber FFI mit stabilen Snapshot-Assertions.
    /// Testet den SetBackgroundLayerVisibility-Flow ohne externe ZIP-Datei,
    /// fokussiert auf stabilen Chrome-State und Visibility-Flags.
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    // Lade Test-Map ueber OpenFile-Dialog.
    apply_action_json(session, HostSessionAction::OpenFile);

    let requests_json = read_and_free_string(session_take_dialog_requests_json(session));
    let requests: Vec<HostDialogRequest> =
        serde_json::from_str(&requests_json).expect("dialog request JSON must parse");
    assert!(!requests.is_empty());
    assert_eq!(requests[0].kind, HostDialogRequestKind::OpenFile);

    // Fuege Test-XML-Datei als Dialog-Ergebnis ein.
    let test_map_path = format!(
        "{}/../../ad_sample_data/AutoDrive_config-test.xml",
        env!("CARGO_MANIFEST_DIR")
    );
    let open_result_json = CString::new(
        serde_json::to_string(&HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::OpenFile,
            path: test_map_path,
        })
        .expect("dialog result JSON must serialize"),
    )
    .expect("CString must build");
    assert!(session_submit_dialog_result_json(
        session,
        open_result_json.as_ptr()
    ));

    // Snapshot nach Laden: sollte Map enthalten.
    let loaded_snapshot_json = read_and_free_string(session_snapshot_json(session));
    let loaded_snapshot: HostSessionSnapshot =
        serde_json::from_str(&loaded_snapshot_json).expect("snapshot JSON must parse");
    assert!(loaded_snapshot.has_map);

    // Chrome-Snapshot vor Toggle: Pruefen auf initiale Layer-Entries.
    let chrome_before_json = read_and_free_string(session_chrome_snapshot_json(session));
    let chrome_before: fs25_auto_drive_host_bridge::HostChromeSnapshot =
        serde_json::from_str(&chrome_before_json).expect("chrome snapshot JSON must parse");
    let initial_visibility = chrome_before
        .background_layer_entries
        .iter()
        .map(|entry| (entry.id.clone(), entry.visible))
        .collect::<Vec<_>>();

    // Wende SetBackgroundLayerVisibility an (toggle erste Layer-ID wenn vorhanden).
    if !chrome_before.background_layer_entries.is_empty() {
        let first_entry = &chrome_before.background_layer_entries[0];
        let layer_id = first_entry.id.clone();
        let new_visibility = !first_entry.visible;

        apply_action_json(
            session,
            HostSessionAction::SetBackgroundLayerVisibility {
                layer: layer_id.clone(),
                visible: new_visibility,
            },
        );

        // Chrome-Snapshot nach Toggle: Pruefen auf veraenderte Visibility.
        let chrome_after_json = read_and_free_string(session_chrome_snapshot_json(session));
        let chrome_after: fs25_auto_drive_host_bridge::HostChromeSnapshot =
            serde_json::from_str(&chrome_after_json).expect("chrome snapshot JSON must parse");

        // Assertion auf stabiles Feld: Visibility-Flag der getoggelt Layer muss sich geaendert haben.
        let toggled_entry = chrome_after
            .background_layer_entries
            .iter()
            .find(|e| e.id == layer_id)
            .expect("toggled layer must exist in updated snapshot");
        assert_eq!(
            toggled_entry.visible, new_visibility,
            "Layer visibility toggle must be reflected in chrome snapshot"
        );

        // Assertion auf weitere stabile Felder: Layer-Entries sollten nicht verloren gehen.
        assert_eq!(
            chrome_before.background_layer_entries.len(),
            chrome_after.background_layer_entries.len(),
            "Number of background layer entries must be stable"
        );
    }

    session_dispose(session);
}

#[test]
fn ffi_e2e_draft_confirm_overview_options_positive_flow() {
    /// E2E-Test fuer Draft->Confirm Overview-Options Flow ueber FFI.
    /// Testet update_overview_options_dialog mit ConfirmOverviewOptions
    /// auf stabilen State-Assertions ohne externe ZIP-Datei.
    let session = fs25ad_host_bridge_session_new();
    assert!(!session.is_null());

    // Oeffne den Options-Dialog via OpenOptionsDialog Action.
    apply_action_json(session, HostSessionAction::OpenOptionsDialog);

    let snapshot_after_open = read_and_free_string(session_snapshot_json(session));
    let snapshot: HostSessionSnapshot =
        serde_json::from_str(&snapshot_after_open).expect("snapshot JSON must parse");
    assert!(
        snapshot.show_options_dialog,
        "Options dialog must be open after OpenOptionsDialog action"
    );

    // Bestaetigung: Dialog ist nun sichtbar, stellt sicheren Zustand dar.
    let ui_json = read_and_free_string(session_ui_snapshot_json(session));
    let ui_value: serde_json::Value =
        serde_json::from_str(&ui_json).expect("ui snapshot JSON must parse");
    assert!(ui_value.get("panels").is_some());

    // Wende ConfirmOverviewOptions an (ohne zuvor draft-Aenderungen zu machen,
    // testet damit den Confirm-Flow mit Default-Werten).
    apply_action_json(session, HostSessionAction::ConfirmOverviewOptions);

    // Snapshot nach Confirm: Dialog sollte geschlossen sein.
    let snapshot_after_confirm = read_and_free_string(session_snapshot_json(session));
    let snapshot_confirmed: HostSessionSnapshot =
        serde_json::from_str(&snapshot_after_confirm).expect("snapshot JSON must parse");
    assert!(
        !snapshot_confirmed.show_options_dialog,
        "Options dialog must be closed after ConfirmOverviewOptions action"
    );

    // Assertion auf stabiles Feld: show_options_dialog Flag aendert sich deterministisch.
    // Weitere stabile Assertions auf Chrome/Snapshot-Zustand sind moeglich,
    // aber hier fokussieren wir auf den kritischen Flow.
    assert_eq!(
        snapshot_confirmed.show_options_dialog, false,
        "show_options_dialog must be false in stable state after confirm"
    );

    session_dispose(session);
}

