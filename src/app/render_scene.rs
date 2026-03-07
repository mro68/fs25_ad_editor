//! Builder fuer Render-Szenen aus dem AppState.
//!
//! Dieses Modul ist verantwortlich fuer die Transformation des internen AppState
//! in den expliziten Render-Vertrag `RenderScene`. Die gebaute Szene enthaelt alle
//! Informationen, die der Render-Layer benoetigt, ohne den State direkt zu koppeln.

use crate::app::{AppState, SegmentRegistry};
use crate::shared::RenderScene;
use indexmap::IndexSet;
use std::sync::{Arc, OnceLock};

/// Gibt einen Arc auf eine leere, statisch initialisierte `IndexSet<u64>` zurueck.
///
/// Verhindert eine Heap-Allokation pro Frame, wenn kein Node ausgeblendet werden soll.
/// Die Instanz wird beim ersten Aufruf lazy erstellt und danach wiederverwendet.
fn empty_hidden_ids() -> Arc<IndexSet<u64>> {
    static EMPTY: OnceLock<Arc<IndexSet<u64>>> = OnceLock::new();
    Arc::clone(EMPTY.get_or_init(|| Arc::new(IndexSet::new())))
}

/// Berechnet die zu dimmenden Node-IDs fuer einen Frame.
///
/// Wenn genau 1 Node selektiert ist und dieser zu einem Segment gehoert,
/// werden alle anderen Nodes des Segments in die Rueckgabemenge aufgenommen.
/// In allen anderen Faellen wird eine leere Menge zurueckgegeben.
fn compute_dimmed_ids(
    registry: &SegmentRegistry,
    selected: &Arc<IndexSet<u64>>,
) -> Arc<IndexSet<u64>> {
    if selected.len() != 1 {
        return empty_hidden_ids();
    }
    let node_id = *selected.iter().next().expect("len == 1");
    let Some(record) = registry.find_first_by_node_id(node_id) else {
        return empty_hidden_ids();
    };
    let dimmed: IndexSet<u64> = record
        .node_ids
        .iter()
        .copied()
        .filter(|&id| id != node_id)
        .collect();
    if dimmed.is_empty() {
        empty_hidden_ids()
    } else {
        Arc::new(dimmed)
    }
}

/// Baut eine RenderScene aus dem aktuellen AppState.
///
/// Diese Funktion extrahiert die notwendigen Daten aus dem `AppState` und
/// montiert sie in das explizite `RenderScene`-Datenmodell. Die Szene ist
/// der Render-Layer-Vertrag und deckt folgende Bereiche ab:
///
/// - **Geometrie**: `road_map`, `selected_node_ids`, Verbindungen fuer Preview
/// - **Sichtbarkeit**: `background_map`, `background_visible`, `hidden_node_ids`
/// - **Viewport**: Kamera, Groesse der Anzeige, Render-Qualitaet
/// - **Interaktion**: Connection-Tool State (`connect_source_node`)
/// - **Konfiguration**: `options_arc` (EditorOptions als shared Arc)
///
/// # Besonderheiten
///
/// - `hidden_node_ids` wird automatisch mit selektierten Nodes gefuellt,
///   wenn die Distanzen-Vorschau aktiv ist und "Original ausblenden" aktiviert wurde.
/// - `options_arc` ist ein Arc-Clone von `state.options_arc()` — das ermoeglicht
///   CoW-Updates ohne per-Frame Allokationen.
///
/// # Parameter
/// - `state` – Referenz zum aktuellen AppState
/// - `viewport_size` – Fenstergroesse in Pixeln als `[width, height]`
///
/// # Rueckgabe
/// Eine vollstaendige `RenderScene`, bereit zum Rendering.
pub fn build(state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
    // Arc einmal klonen — wiederverwendet fuer selected_node_ids UND hidden_node_ids
    let selected_arc = state.selection.selected_node_ids.clone();

    // Wenn Distanzen-Vorschau aktiv + hide_original → selektierte Nodes ausblenden.
    // Statt nochmals zu klonen verwenden wir den gleichen Arc (billiger O(1)-Clone).
    let hidden_node_ids = if state.ui.distanzen.should_hide_original() {
        Arc::clone(&selected_arc)
    } else {
        empty_hidden_ids()
    };

    // Gedimmte Nodes: alle anderen Nodes des Segments wenn 1 Segment-Node selektiert.
    let dimmed_node_ids = compute_dimmed_ids(&state.segment_registry, &selected_arc);

    RenderScene {
        road_map: state.road_map.clone(),
        camera: state.view.camera.clone(),
        viewport_size,
        render_quality: state.view.render_quality,
        selected_node_ids: selected_arc,
        connect_source_node: state.editor.connect_source_node,
        background_map: state.view.background_map.clone(),
        background_visible: state.view.background_visible,
        options: state.options_arc(),
        hidden_node_ids,
        dimmed_node_ids,
    }
}
