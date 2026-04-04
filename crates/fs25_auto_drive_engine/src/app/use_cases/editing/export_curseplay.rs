//! Use-Case: Selektierte Nodes als Curseplay XML exportieren.
//!
//! Liest die Positionen der aktuell selektierten Nodes in Selektionsreihenfolge
//! und schreibt sie als Curseplay-`<customField>`-XML in eine Datei.

use crate::app::AppState;
use crate::xml::write_curseplay;
use glam::Vec2;

/// Exportiert die selektierten Nodes als Curseplay-XML-Datei.
///
/// - Liest die Positionen der selektierten Nodes in Reihenfolge aus der Selektion
/// - Schreibt die Datei ueber `write_curseplay`
/// - Bei leerer Selektion oder fehlender RoadMap wird fruehzeitig zurueckgekehrt
pub fn export_curseplay(state: &AppState, path: &str) {
    let selected_ids = &state.selection.selected_node_ids;
    if selected_ids.is_empty() {
        log::warn!("Keine Nodes selektiert — Curseplay-Export abgebrochen");
        return;
    }

    let road_map = match state.road_map.as_ref() {
        Some(rm) => rm,
        None => {
            log::warn!("Keine RoadMap geladen — Curseplay-Export abgebrochen");
            return;
        }
    };

    // Positionen in Selektionsreihenfolge sammeln
    let positions: Vec<Vec2> = selected_ids
        .iter()
        .filter_map(|id| road_map.node(*id))
        .map(|node| node.position)
        .collect();

    if positions.is_empty() {
        log::warn!(
            "Selektierte Nodes haben keine gueltigen Positionen — Curseplay-Export abgebrochen"
        );
        return;
    }

    let xml = write_curseplay(&positions);

    match std::fs::write(path, xml) {
        Ok(()) => {
            log::info!(
                "Exported {} nodes to Curseplay file '{}'",
                positions.len(),
                path
            );
        }
        Err(e) => {
            log::error!("Failed to write Curseplay file '{}': {}", path, e);
        }
    }
}
