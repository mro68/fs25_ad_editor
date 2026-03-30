//! Builder fuer Render-Szenen aus dem AppState.
//!
//! Dieses Modul ist verantwortlich fuer die Transformation des internen AppState
//! in den expliziten Render-Vertrag `RenderScene`. Die gebaute Szene enthaelt alle
//! Informationen, die der Render-Layer benoetigt, ohne den State direkt zu koppeln.

use crate::app::{AppState, GroupRegistry};
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
/// Fuer alle selektierten Nodes werden die betroffenen Segmente ermittelt.
/// Alle Segment-Nodes, die NICHT selektiert sind, werden in die Dimm-Menge aufgenommen.
/// Bei leerer Selektion oder wenn kein Node zu einem Segment gehoert, wird eine
/// leere Menge zurueckgegeben.
///
/// Implementierung als einziger Pass ueber alle Records statt pro-Node-Lookup —
/// effizienter fuer den Frame-Hot-Path bei vielen selektierten Nodes.
fn compute_dimmed_ids(
    registry: &GroupRegistry,
    selected: &Arc<IndexSet<u64>>,
) -> Arc<IndexSet<u64>> {
    if selected.is_empty() {
        return empty_hidden_ids();
    }
    let mut dimmed = IndexSet::new();
    for record in registry.records() {
        if record.node_ids.iter().any(|id| selected.contains(id)) {
            for &id in &record.node_ids {
                if !selected.contains(&id) {
                    dimmed.insert(id);
                }
            }
        }
    }
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
    // Cache-Hit wenn weder Selektion noch Registry seit dem letzten Build geaendert haben.
    let dimmed_node_ids = {
        let sel_gen = state.selection.generation;
        let reg_gen = state.group_registry.dimmed_generation;
        let mut cache = state.dimmed_ids_cache.borrow_mut();
        match cache.as_ref() {
            Some((s, r, result)) if *s == sel_gen && *r == reg_gen => Arc::clone(result),
            _ => {
                let result = compute_dimmed_ids(&state.group_registry, &selected_arc);
                *cache = Some((sel_gen, reg_gen, Arc::clone(&result)));
                result
            }
        }
    };

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
