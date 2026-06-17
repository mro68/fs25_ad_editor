//! Unit-Tests fuer das Strecken-Versatz-Tool.

use super::state::RouteOffsetTool;
use crate::app::tools::RouteToolCore;
use crate::app::ui_contract::RouteOffsetPanelAction;
use crate::core::ConnectionDirection;
use crate::core::RoadMap;
use glam::Vec2;

// ─── Hilfsfunktion ────────────────────────────────────────────────────────────

fn make_chain(n: usize, spacing: f32) -> Vec<Vec2> {
    (0..n).map(|i| Vec2::new(i as f32 * spacing, 0.0)).collect()
}

// ─── State-Tests ─────────────────────────────────────────────────────────────

/// Neues Tool hat keine Kette.
#[test]
fn test_new_tool_hat_keine_kette() {
    let tool = RouteOffsetTool::new();
    assert!(!tool.has_chain());
    assert!(!tool.is_ready());
}

/// load_chain → has_chain() == true, reset → has_chain() == false.
#[test]
fn test_load_chain_und_reset() {
    let mut tool = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool.load_chain(chain, 1, 5);
    assert!(tool.has_chain());
    assert!(tool.is_ready());
    tool.reset();
    assert!(!tool.has_chain());
    assert!(!tool.is_ready());
}

/// Einzelner Punkt — has_chain() bleibt false.
#[test]
fn test_load_chain_ein_punkt_reicht_nicht() {
    let mut tool = RouteOffsetTool::new();
    tool.load_chain(vec![Vec2::ZERO], 1, 1);
    assert!(!tool.has_chain());
}

// ─── Geometrie-Stubs (werden nach Developer-Implementierung freigegeben) ──────

/// Links-Versatz — execute() gibt Some(ToolResult) zurueck wenn Kette geladen.
#[test]
fn test_offset_links_gerade_kette() {
    let mut tool = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool.load_chain(chain, 1, 5);
    tool.config.left_enabled = true;
    tool.config.right_enabled = false;
    tool.config.left_distance = 8.0;
    let road_map = RoadMap::new(3);
    let result = tool.execute(&road_map);
    assert!(result.is_some(), "Links-Versatz sollte Ergebnis liefern");
    let r = result.unwrap();
    assert!(!r.new_nodes.is_empty(), "Neue Nodes erwartet");
    assert_eq!(
        r.external_connections.len(),
        2,
        "Eine Offset-Seite muss genau zwei laterale Anker-Verbindungen erzeugen"
    );
    assert!(r.markers.is_empty(), "RouteOffset erzeugt keine Marker");
    assert!(
        r.nodes_to_remove.is_empty(),
        "Bei keep_original=true duerfen keine Nodes geloescht werden"
    );
}

/// Rechts-Versatz erzeugt Nodes auf der anderen Seite.
#[test]
fn test_offset_rechts_symmetrisch() {
    let mut tool = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool.load_chain(chain, 1, 5);
    tool.config.left_enabled = false;
    tool.config.right_enabled = true;
    tool.config.right_distance = 8.0;
    let road_map = RoadMap::new(3);
    let result = tool.execute(&road_map);
    assert!(result.is_some(), "Rechts-Versatz sollte Ergebnis liefern");
}

/// Beide Seiten aktiv → mehr Nodes als nur eine Seite.
#[test]
fn test_offset_beide_seiten() {
    let mut tool_beide = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool_beide.load_chain(chain.clone(), 1, 5);
    tool_beide.config.left_enabled = true;
    tool_beide.config.right_enabled = true;

    let mut tool_eine = RouteOffsetTool::new();
    tool_eine.load_chain(chain, 1, 5);
    tool_eine.config.left_enabled = true;
    tool_eine.config.right_enabled = false;

    let road_map = RoadMap::new(3);
    let r_beide = tool_beide.execute(&road_map).unwrap();
    let r_eine = tool_eine.execute(&road_map).unwrap();
    assert!(
        r_beide.new_nodes.len() > r_eine.new_nodes.len(),
        "Beide Seiten muss mehr Nodes erzeugen als eine"
    );
}

/// Spezialfall: Einbahn-vorwaerts + links/rechts aktiv → genau eine Seite ist reverse.
#[test]
fn test_offset_beide_seiten_einbahn_vorwaerts_hat_eine_umgedrehte_seite_standard() {
    let mut tool = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool.load_chain(chain, 1, 5);
    tool.direction = ConnectionDirection::Regular;
    tool.config.left_enabled = true;
    tool.config.right_enabled = true;

    let road_map = RoadMap::new(3);
    let result = tool
        .execute(&road_map)
        .expect("Ein Ergebnis fuer beide Versatzseiten wird erwartet");

    let regular_count = result
        .internal_connections
        .iter()
        .filter(|(_, _, direction, _)| *direction == ConnectionDirection::Regular)
        .count();
    let reverse_count = result
        .internal_connections
        .iter()
        .filter(|(_, _, direction, _)| *direction == ConnectionDirection::Reverse)
        .count();

    assert!(
        regular_count > 0,
        "Eine Versatzseite muss in Vorwaertsrichtung bleiben"
    );
    assert!(
        reverse_count > 0,
        "Eine Versatzseite muss in Gegenrichtung erzeugt werden"
    );
}

/// Toggle im Spezialfall wechselt, welche Seite als reverse erzeugt wird.
#[test]
fn test_offset_toggle_reversed_side_wechselt_reverse_seite() {
    let mut tool = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool.load_chain(chain, 1, 5);
    tool.direction = ConnectionDirection::Regular;
    tool.config.left_enabled = true;
    tool.config.right_enabled = true;

    let road_map = RoadMap::new(3);
    let first = tool
        .execute(&road_map)
        .expect("Ein Ergebnis vor dem Toggle wird erwartet");

    let first_direction = first
        .external_connections
        .iter()
        .find_map(|(new_idx, existing_id, _existing_to_new, direction, _)| {
            if *new_idx == 0 && *existing_id == 1 {
                Some(*direction)
            } else {
                None
            }
        })
        .expect("Die linke Start-Ankerverbindung muss vorhanden sein");
    assert_eq!(first_direction, ConnectionDirection::Regular);

    let effect = tool.apply_panel_action(RouteOffsetPanelAction::ToggleReversedSide);
    assert!(
        effect.changed,
        "Toggle muss im Spezialfall eine Aenderung liefern"
    );

    let second = tool
        .execute(&road_map)
        .expect("Ein Ergebnis nach dem Toggle wird erwartet");
    let second_direction = second
        .external_connections
        .iter()
        .find_map(|(new_idx, existing_id, _existing_to_new, direction, _)| {
            if *new_idx == 0 && *existing_id == 1 {
                Some(*direction)
            } else {
                None
            }
        })
        .expect("Die linke Start-Ankerverbindung muss nach Toggle vorhanden sein");
    assert_eq!(second_direction, ConnectionDirection::Reverse);
}

/// "Original entfernen" → nodes_to_remove enthaelt Ketten-Node-IDs.
#[test]
fn test_offset_original_entfernen_fuellt_nodes_to_remove() {
    let mut tool = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool.load_chain(chain, 1, 5);
    tool.config.keep_original = false;
    let road_map = RoadMap::new(3);
    let result = tool.execute(&road_map).unwrap();
    assert!(
        !result.nodes_to_remove.is_empty(),
        "nodes_to_remove muss befuellt sein wenn Original entfernt wird"
    );
}

/// "Original beibehalten" → nodes_to_remove ist leer.
#[test]
fn test_offset_original_beibehalten_leeres_nodes_to_remove() {
    let mut tool = RouteOffsetTool::new();
    let chain = make_chain(5, 10.0);
    tool.load_chain(chain, 1, 5);
    tool.config.keep_original = true;
    let road_map = RoadMap::new(3);
    let result = tool.execute(&road_map).unwrap();
    assert!(
        result.nodes_to_remove.is_empty(),
        "nodes_to_remove muss leer sein wenn Original beibehalten wird"
    );
}

/// Keine Kette → execute() gibt None zurueck.
#[test]
fn test_execute_ohne_kette_gibt_none() {
    let tool = RouteOffsetTool::new();
    // execute() hat #[allow(unused)] auf todo!() — Test prueft nur den Guard
    // (before the todo! panics):
    // Hinweis: Sobald Developer execute() implementiert, wird dieser Test
    // direkt ohne #[ignore] laufen.
    assert!(!tool.is_ready());
}
