//! Unit-Tests fuer das Ausweichstrecken-Tool.

use super::compute_bypass_positions;
use super::BypassTool;
use crate::app::group_registry::GroupKind;
use crate::app::tools::RouteTool;
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

// ─── Geometrie-Tests ─────────────────────────────────────────────────────────

/// Gerade Kette mit 5 Punkten — Standard-Anwendungsfall.
#[test]
fn test_compute_bypass_gerade_kette() {
    let chain: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    let result = compute_bypass_positions(&chain, 5.0, 6.0);
    assert!(result.is_some(), "Gerade Kette sollte Ergebnis liefern");
    let (positions, d_blend) = result.unwrap();
    assert!(!positions.is_empty(), "Mindestens ein Bypass-Node erwartet");
    assert!(d_blend > 0.0, "d_blend muss positiv sein");
}

/// Minimale Kette: genau zwei Punkte.
#[test]
fn test_compute_bypass_zwei_punkte() {
    let chain = vec![Vec2::new(0.0, 0.0), Vec2::new(30.0, 0.0)];
    let result = compute_bypass_positions(&chain, 4.0, 6.0);
    assert!(result.is_some(), "Zweipunkt-Kette sollte Ergebnis liefern");
    let (positions, _) = result.unwrap();
    assert!(!positions.is_empty());
}

/// Zu kurze Eingabe (0 oder 1 Punkt) → None.
#[test]
fn test_compute_bypass_zu_kurz_liefert_none() {
    assert!(
        compute_bypass_positions(&[], 5.0, 6.0).is_none(),
        "Leere Kette muss None liefern"
    );
    assert!(
        compute_bypass_positions(&[Vec2::ZERO], 5.0, 6.0).is_none(),
        "Einfacher Punkt muss None liefern"
    );
}

/// Offset = 0 — Bypass liegt auf der Kette, Berechnung trotzdem erfolgreich.
#[test]
fn test_compute_bypass_offset_null() {
    let chain: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();
    let result = compute_bypass_positions(&chain, 0.0, 6.0);
    assert!(result.is_some(), "Offset=0 sollte ein Ergebnis liefern");
}

/// Feines Spacing erzeugt mehr Nodes als grobes.
#[test]
fn test_compute_bypass_node_count_skaliert_mit_spacing() {
    let chain: Vec<Vec2> = (0..10).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();

    let (pos_fine, _) =
        compute_bypass_positions(&chain, 5.0, 3.0).expect("Feines Spacing sollte Ergebnis liefern");
    let (pos_coarse, _) = compute_bypass_positions(&chain, 5.0, 10.0)
        .expect("Grobes Spacing sollte Ergebnis liefern");

    assert!(
        pos_fine.len() > pos_coarse.len(),
        "Feines Spacing ({}) muss mehr Nodes erzeugen als grobes ({})",
        pos_fine.len(),
        pos_coarse.len()
    );
}

/// Negativer Offset (rechts) funktioniert symmetrisch zu positivem Offset (links).
#[test]
fn test_compute_bypass_negativer_offset() {
    let chain: Vec<Vec2> = (0..5).map(|i| Vec2::new(i as f32 * 10.0, 0.0)).collect();

    let (pos_links, _) =
        compute_bypass_positions(&chain, 5.0, 6.0).expect("Linker Offset fehlgeschlagen");
    let (pos_rechts, _) =
        compute_bypass_positions(&chain, -5.0, 6.0).expect("Rechter Offset fehlgeschlagen");

    assert_eq!(
        pos_links.len(),
        pos_rechts.len(),
        "Links/Rechts muss gleich viele Nodes erzeugen"
    );
}

// ─── Lifecycle-Tests ─────────────────────────────────────────────────────────

/// Nach new() ist kein Input vorhanden und das Tool nicht bereit.
#[test]
fn test_neues_tool_hat_keinen_input() {
    let tool = BypassTool::new();
    assert!(!tool.has_pending_input(), "Kein Input erwartet nach new()");
    assert!(!tool.is_ready(), "Tool darf nicht bereit sein nach new()");
}

/// Nach load_chain mit 2 Punkten ist Input vorhanden und Tool bereit.
#[test]
fn test_has_pending_input_nach_load_chain() {
    let mut tool = BypassTool::new();

    tool.load_chain(vec![Vec2::ZERO, Vec2::new(20.0, 0.0)], 1, 2);

    assert!(tool.has_pending_input(), "Input nach load_chain erwartet");
    assert!(tool.is_ready(), "Tool muss bereit sein nach load_chain");
}

/// is_ready() gibt erst bei mind. 2 Punkten true zurück.
#[test]
fn test_is_ready_benoetigt_zwei_punkte() {
    let mut tool = BypassTool::new();

    assert!(!tool.is_ready(), "Kein Punkt → nicht bereit");

    tool.chain_positions.push(Vec2::ZERO);
    assert!(!tool.is_ready(), "Ein Punkt → nicht bereit");

    tool.chain_positions.push(Vec2::new(10.0, 0.0));
    assert!(tool.is_ready(), "Zwei Punkte → bereit");
}

/// reset() leert Kette, Cache und d_blend.
#[test]
fn test_reset_leert_alle_felder() {
    let mut tool = BypassTool::new();
    tool.load_chain(vec![Vec2::ZERO, Vec2::new(20.0, 0.0)], 1, 2);

    // Cache manuell befuellen um Reset zu pruefen
    tool.cached_positions = Some(vec![Vec2::ZERO]);
    tool.cached_connections = Some(vec![(0, 1)]);
    tool.d_blend = 3.5;

    tool.reset();

    assert!(!tool.has_pending_input(), "Kein Input nach reset()");
    assert!(!tool.is_ready(), "Nicht bereit nach reset()");
    assert!(
        tool.chain_positions.is_empty(),
        "chain_positions muss leer sein"
    );
    assert!(tool.cached_positions.is_none(), "Cache muss ungueltig sein");
    assert!(
        tool.cached_connections.is_none(),
        "Connection-Cache muss ungueltig sein"
    );
    assert_eq!(tool.d_blend, 0.0, "d_blend muss 0 sein");
}

/// load_chain() invalidiert bestehenden Cache.
#[test]
fn test_load_chain_invalidiert_cache() {
    let mut tool = BypassTool::new();

    tool.cached_positions = Some(vec![Vec2::ZERO]);
    tool.cached_connections = Some(vec![(0, 1)]);

    tool.load_chain(vec![Vec2::ZERO, Vec2::new(15.0, 0.0)], 1, 2);

    assert!(
        tool.cached_positions.is_none(),
        "Cache nach load_chain ungueltig"
    );
    assert!(
        tool.cached_connections.is_none(),
        "Connection-Cache nach load_chain ungueltig"
    );
}

/// execute() ohne geladene Kette liefert None.
#[test]
fn test_execute_ohne_kette_liefert_none() {
    let tool = BypassTool::new();
    let road_map = RoadMap::new(3);
    assert!(tool.execute(&road_map).is_none());
}

// ─── ToolLifecycleState-Tests ─────────────────────────────────────────────────

/// set_snap_radius() wird im lifecycle gespeichert.
#[test]
fn test_set_snap_radius_wird_gespeichert() {
    let mut tool = BypassTool::new();
    tool.set_snap_radius(7.5);
    assert_eq!(tool.lifecycle.snap_radius, 7.5);
}

/// Snap-Radius bleibt nach reset() erhalten.
#[test]
fn test_snap_radius_bleibt_nach_reset() {
    let mut tool = BypassTool::new();
    tool.set_snap_radius(5.0);
    tool.load_chain(vec![Vec2::ZERO, Vec2::new(10.0, 0.0)], 1, 2);
    tool.reset();
    assert_eq!(
        tool.lifecycle.snap_radius, 5.0,
        "Snap-Radius muss reset() ueberleben"
    );
}

/// set_last_created() speichert Node-IDs.
#[test]
fn test_set_last_created_speichert_ids() {
    let mut tool = BypassTool::new();
    let road_map = RoadMap::new(3);

    tool.set_last_created(&[10, 20, 30], &road_map);
    assert_eq!(tool.last_created_ids(), &[10, 20, 30]);
}

// ─── GroupRecord-Tests ─────────────────────────────────────────────────────

/// Roundtrip: make_group_record erstellt den korrekten Record,
/// load_for_edit stellt alle Felder exakt wieder her.
#[test]
fn bypass_segment_record_roundtrip() {
    let mut tool = BypassTool::new();
    let chain = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(20.0, 5.0),
    ];
    tool.load_chain(chain.clone(), 100, 102);
    tool.offset = 6.0;
    tool.base_spacing = 5.0;
    tool.direction = ConnectionDirection::Regular;
    tool.priority = ConnectionPriority::SubPriority;

    let record = tool.make_group_record(42, &[100, 101, 102]);
    assert!(record.is_some(), "Record muss vorhanden sein");
    let record = record.unwrap();

    let GroupKind::Bypass {
        ref chain_positions,
        chain_start_id,
        chain_end_id,
        offset,
        base_spacing,
        ref base,
    } = record.kind
    else {
        panic!("Erwartetes GroupKind::Bypass, bekam etwas anderes");
    };
    assert_eq!(
        chain_positions, &chain,
        "chain_positions muss uebereinstimmen"
    );
    assert_eq!(chain_start_id, 100, "chain_start_id muss uebereinstimmen");
    assert_eq!(chain_end_id, 102, "chain_end_id muss uebereinstimmen");
    assert_eq!(offset, 6.0, "Offset muss uebereinstimmen");
    assert_eq!(base_spacing, 5.0, "base_spacing muss uebereinstimmen");
    assert_eq!(
        base.direction,
        ConnectionDirection::Regular,
        "direction im base"
    );
    assert_eq!(
        base.priority,
        ConnectionPriority::SubPriority,
        "priority im base"
    );

    // Roundtrip: neues Tool, load_for_edit
    let mut tool2 = BypassTool::new();
    tool2.load_for_edit(&record, &record.kind);

    assert_eq!(
        tool2.chain_positions, chain,
        "chain_positions nach load_for_edit"
    );
    assert_eq!(
        tool2.chain_start_id, 100,
        "chain_start_id nach load_for_edit"
    );
    assert_eq!(tool2.chain_end_id, 102, "chain_end_id nach load_for_edit");
    assert_eq!(tool2.offset, 6.0, "Offset nach load_for_edit");
    assert_eq!(tool2.base_spacing, 5.0, "base_spacing nach load_for_edit");
    assert_eq!(
        tool2.direction,
        ConnectionDirection::Regular,
        "direction nach load_for_edit"
    );
    assert_eq!(
        tool2.priority,
        ConnectionPriority::SubPriority,
        "priority nach load_for_edit"
    );
}

/// Ohne geladene Kette muss make_group_record None liefern.
#[test]
fn bypass_segment_record_none_ohne_chain() {
    let tool = BypassTool::new();
    let record = tool.make_group_record(0, &[]);
    assert!(
        record.is_none(),
        "Ohne Kette muss make_group_record None liefern"
    );
}
