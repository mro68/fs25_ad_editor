//! Integrationstests fuer Copy/Paste ueber AppController-Flow.
//!
//! Prueft copy_selected_to_clipboard, start_paste_preview, confirm_paste
//! und cancel_paste_preview Ende-zu-Ende.

use fs25_auto_drive_editor::app::{AppController, AppIntent, AppState};
use fs25_auto_drive_editor::core::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use std::sync::Arc;

fn make_test_map() -> AppState {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        glam::Vec2::new(10.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(0.0, 0.0),
        glam::Vec2::new(10.0, 0.0),
    ));
    map.ensure_spatial_index();
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(map));
    state
}

/// Selektion kopieren fuellt Clipboard korrekt.
#[test]
fn test_copy_selection_fills_clipboard() {
    let mut state = make_test_map();
    let mut ctrl = AppController::new();

    // Beide Nodes selektieren
    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);

    ctrl.handle_intent(&mut state, AppIntent::CopySelectionRequested)
        .expect("CopySelectionRequested muss erfolgreich sein");

    assert_eq!(
        state.clipboard.nodes.len(),
        2,
        "Clipboard soll 2 Nodes enthalten"
    );
    assert_eq!(
        state.clipboard.connections.len(),
        1,
        "Clipboard soll 1 Verbindung enthalten"
    );
    // Zentrum liegt zwischen (0,0) und (10,0) → (5,0)
    assert!(
        (state.clipboard.center.x - 5.0).abs() < 0.01,
        "Zentrum x sollte 5.0 sein"
    );
}

/// Paste-Vorschau starten setzt paste_preview_pos.
#[test]
fn test_paste_start_sets_preview_pos() {
    let mut state = make_test_map();
    let mut ctrl = AppController::new();

    // Clipboard vorbereiten
    state.selection.ids_mut().insert(1);
    ctrl.handle_intent(&mut state, AppIntent::CopySelectionRequested)
        .expect("Copy muss funktionieren");

    ctrl.handle_intent(&mut state, AppIntent::PasteStartRequested)
        .expect("PasteStartRequested muss erfolgreich sein");

    assert!(
        state.paste_preview_pos.is_some(),
        "paste_preview_pos muss nach PasteStartRequested gesetzt sein"
    );
}

/// PastePreviewMoved aktualisiert paste_preview_pos.
#[test]
fn test_paste_preview_moved_updates_pos() {
    let mut state = make_test_map();
    let mut ctrl = AppController::new();

    state.selection.ids_mut().insert(1);
    ctrl.handle_intent(&mut state, AppIntent::CopySelectionRequested)
        .unwrap();
    ctrl.handle_intent(&mut state, AppIntent::PasteStartRequested)
        .unwrap();

    let new_pos = glam::Vec2::new(50.0, 50.0);
    ctrl.handle_intent(
        &mut state,
        AppIntent::PastePreviewMoved { world_pos: new_pos },
    )
    .expect("PastePreviewMoved muss erfolgreich sein");

    assert_eq!(
        state.paste_preview_pos,
        Some(new_pos),
        "paste_preview_pos muss auf neue Position gesetzt sein"
    );
}

/// PasteConfirmRequested fuegt neue Nodes ein und loescht Preview.
#[test]
fn test_confirm_paste_inserts_nodes() {
    let mut state = make_test_map();
    let mut ctrl = AppController::new();

    // Beide Nodes kopieren
    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);
    ctrl.handle_intent(&mut state, AppIntent::CopySelectionRequested)
        .unwrap();
    ctrl.handle_intent(&mut state, AppIntent::PasteStartRequested)
        .unwrap();

    // Vorschau auf Offset-Position setzen
    ctrl.handle_intent(
        &mut state,
        AppIntent::PastePreviewMoved {
            world_pos: glam::Vec2::new(5.0, 20.0),
        },
    )
    .unwrap();

    let node_count_before = state
        .road_map
        .as_ref()
        .map(|rm| rm.node_count())
        .unwrap_or(0);

    ctrl.handle_intent(&mut state, AppIntent::PasteConfirmRequested)
        .expect("PasteConfirmRequested muss erfolgreich sein");

    let node_count_after = state
        .road_map
        .as_ref()
        .map(|rm| rm.node_count())
        .unwrap_or(0);

    assert_eq!(
        node_count_after,
        node_count_before + 2,
        "Paste soll 2 neue Nodes einfuegen"
    );
    assert!(
        state.paste_preview_pos.is_none(),
        "paste_preview_pos muss nach Confirm geleert sein"
    );
    assert_eq!(
        state.selection.selected_node_ids.len(),
        2,
        "Neue Nodes sollen selektiert sein"
    );
}

/// PasteCancelled loescht paste_preview_pos ohne Nodes einzufuegen.
#[test]
fn test_paste_cancelled_clears_preview() {
    let mut state = make_test_map();
    let mut ctrl = AppController::new();

    state.selection.ids_mut().insert(1);
    ctrl.handle_intent(&mut state, AppIntent::CopySelectionRequested)
        .unwrap();
    ctrl.handle_intent(&mut state, AppIntent::PasteStartRequested)
        .unwrap();

    let node_count = state
        .road_map
        .as_ref()
        .map(|rm| rm.node_count())
        .unwrap_or(0);

    ctrl.handle_intent(&mut state, AppIntent::PasteCancelled)
        .expect("PasteCancelled muss erfolgreich sein");

    assert!(
        state.paste_preview_pos.is_none(),
        "paste_preview_pos muss nach Abbruch geleert sein"
    );
    assert_eq!(
        state
            .road_map
            .as_ref()
            .map(|rm| rm.node_count())
            .unwrap_or(0),
        node_count,
        "Keine Nodes nach Abbruch eingefuegt"
    );
}

/// Clipboard bleibt nach Paste erhalten (Mehrfach-Paste moeglich).
#[test]
fn test_clipboard_persists_after_paste() {
    let mut state = make_test_map();
    let mut ctrl = AppController::new();

    state.selection.ids_mut().insert(1);
    ctrl.handle_intent(&mut state, AppIntent::CopySelectionRequested)
        .unwrap();
    ctrl.handle_intent(&mut state, AppIntent::PasteStartRequested)
        .unwrap();
    ctrl.handle_intent(
        &mut state,
        AppIntent::PastePreviewMoved {
            world_pos: glam::Vec2::new(100.0, 0.0),
        },
    )
    .unwrap();
    ctrl.handle_intent(&mut state, AppIntent::PasteConfirmRequested)
        .unwrap();

    // Clipboard soll noch Daten enthalten
    assert!(
        !state.clipboard.nodes.is_empty(),
        "Clipboard soll nach Paste noch Daten enthalten (Mehrfach-Paste)"
    );
}
