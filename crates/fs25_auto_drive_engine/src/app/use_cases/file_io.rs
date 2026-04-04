//! Use-Case-Funktionen fuer Dateiaktionen.
//! Alle Dateisystem-Operationen (I/O) sind hier zentralisiert.

use crate::app::ui_contract::{DialogRequest, DialogRequestKind};
use crate::app::AppState;
use std::sync::Arc;

/// Oeffnet den Open-Datei-Dialog ueber UI-State.
pub fn request_open_file(state: &mut AppState) {
    state
        .ui
        .request_dialog(DialogRequest::pick_path(DialogRequestKind::OpenFile));
}

/// Laedt die ausgewaehlte Datei in den AppState.
///
/// Erkennt duplizierte Nodes und zeigt ggf. einen Bestaetigungsdialog.
pub fn load_selected_file(state: &mut AppState, path: String) -> anyhow::Result<()> {
    let xml_content = std::fs::read_to_string(&path)?;
    let road_map = crate::xml::parse_autodrive_config(&xml_content)?;

    // Merke Pfad fuer spaeteres Save
    state.ui.current_file_path = Some(path.to_string());
    state.selection.ids_mut().clear();

    log::info!(
        "Loaded RoadMap: {} nodes, {} connections",
        road_map.node_count(),
        road_map.connection_count()
    );

    // Duplikate nur zaehlen, noch nicht bereinigen
    let (dup_count, dup_groups) = road_map.count_duplicates(0.01);
    if dup_count > 0 {
        log::warn!(
            "Duplicate nodes detected: {} duplicates in {} groups",
            dup_count,
            dup_groups
        );
        state.ui.dedup_dialog.visible = true;
        state.ui.dedup_dialog.duplicate_count = dup_count;
        state.ui.dedup_dialog.group_count = dup_groups;
    } else {
        state.ui.dedup_dialog.visible = false;
        state.ui.status_message = None;
    }

    // Berechne Bounding Box und zentriere Kamera
    super::camera::center_on_road_map(state, &road_map);

    state.road_map = Some(Arc::new(road_map));
    Ok(())
}

/// Fuehrt die Duplikat-Bereinigung auf der geladenen RoadMap durch.
pub fn deduplicate_loaded_roadmap(state: &mut AppState) {
    let Some(road_map_arc) = state.road_map.take() else {
        return;
    };

    let mut road_map = Arc::unwrap_or_clone(road_map_arc);
    let result = road_map.deduplicate_nodes(0.01);

    let msg = format!(
        "Duplikate bereinigt: {} Nodes entfernt ({} Gruppen), {} Verbindungen umgeleitet, {} Marker angepasst",
        result.removed_nodes,
        result.duplicate_groups,
        result.remapped_connections,
        result.remapped_markers
    );
    log::info!("{}", msg);
    state.ui.status_message = Some(msg);

    log::info!(
        "After deduplication: {} nodes, {} connections",
        road_map.node_count(),
        road_map.connection_count()
    );

    state.road_map = Some(Arc::new(road_map));
    state.ui.dedup_dialog.visible = false;
}

/// Oeffnet den Save-Datei-Dialog ueber UI-State.
pub fn request_save_file(state: &mut AppState) {
    let default_name = state
        .ui
        .current_file_path
        .as_ref()
        .and_then(|path| std::path::Path::new(path).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("AutoDrive_config.xml")
        .to_owned();
    state
        .ui
        .request_dialog(DialogRequest::save_path_with_name(default_name));
}

/// Speichert die aktuelle Datei (wenn Pfad bekannt) oder oeffnet Dialog.
pub fn save_current_file(state: &mut AppState) -> anyhow::Result<()> {
    if let Some(path) = state.ui.current_file_path.clone() {
        write_roadmap_to_file(state, &path)?;
        log::info!("File saved successfully");
        Ok(())
    } else {
        // Kein Pfad bekannt → Save As Dialog oeffnen
        request_save_file(state);
        Ok(())
    }
}

/// Speichert die Datei unter dem angegebenen Pfad.
pub fn save_file_as(state: &mut AppState, path: String) -> anyhow::Result<()> {
    write_roadmap_to_file(state, &path)?;
    state.ui.current_file_path = Some(path.clone());
    log::info!("File saved as: {}", path);
    Ok(())
}

/// Schreibt die RoadMap als XML in eine Datei.
fn write_roadmap_to_file(state: &mut AppState, path: &str) -> anyhow::Result<()> {
    let road_map = state
        .road_map
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Keine Datei geladen"))?;

    // Lade Heightmap falls ausgewaehlt (Bit-Tiefe & Map-Groesse werden automatisch erkannt)
    let heightmap = if let Some(ref hm_path) = state.ui.heightmap_path {
        match crate::core::Heightmap::load(hm_path) {
            Ok(hm) => {
                log::info!(
                    "Heightmap geladen: {} ({}-Bit, {}x{})",
                    hm_path,
                    hm.bit_depth(),
                    hm.dimensions().0,
                    hm.dimensions().1
                );
                Some(hm)
            }
            Err(e) => {
                log::warn!(
                    "Fehler beim Laden der Heightmap: {}. Y-Werte werden auf 0 gesetzt.",
                    e
                );
                None
            }
        }
    } else {
        log::info!("Keine Heightmap ausgewaehlt, Y-Werte werden auf 0 gesetzt");
        None
    };

    let xml_content = crate::xml::write_autodrive_config(
        road_map,
        heightmap.as_ref(),
        state.options.terrain_height_scale,
    )?;
    std::fs::write(path, xml_content)?;
    Ok(())
}

/// Speichert mit Heightmap-Pruefung (zeigt Warnung wenn keine Heightmap ausgewaehlt).
///
/// `path = None` speichert unter `current_file_path` bzw. `pending_save_path`.
/// `path = Some(p)` speichert explizit unter Pfad `p`.
/// Wurde noch keine Heightmap ausgewaehlt und nicht bestaetigt, wird stattdessen
/// die Heightmap-Warnung angezeigt.
pub fn save_with_heightmap_check(state: &mut AppState, path: Option<String>) -> anyhow::Result<()> {
    let actual_path = match path {
        Some(p) => p,
        None => state.ui.pending_save_path.take().unwrap_or_default(),
    };

    if actual_path.is_empty() {
        // Kein Pfad gegeben -> Save (mit current_file_path)
        if state.ui.heightmap_path.is_none()
            && !state.ui.heightmap_warning_confirmed
            && state.road_map.is_some()
        {
            state.ui.pending_save_path = state.ui.current_file_path.clone();
            state.ui.show_heightmap_warning = true;
        } else {
            save_current_file(state)?;
            state.ui.heightmap_warning_confirmed = false;
        }
    } else {
        // Pfad gegeben (Save As oder nach Warnung)
        if state.ui.heightmap_path.is_none() && !state.ui.heightmap_warning_confirmed {
            state.ui.pending_save_path = Some(actual_path);
            state.ui.show_heightmap_warning = true;
        } else {
            save_file_as(state, actual_path)?;
            state.ui.heightmap_warning_confirmed = false;
        }
    }
    Ok(())
}

/// Fuehrt nach Bestaetigung der Heightmap-Warnung den Speichervorgang aus.
pub fn confirm_and_save(state: &mut AppState) -> anyhow::Result<()> {
    state.ui.heightmap_warning_confirmed = true;
    let path = state.ui.pending_save_path.take().unwrap_or_default();
    if !path.is_empty() {
        save_file_as(state, path)?;
    } else {
        save_current_file(state)?;
    }
    state.ui.heightmap_warning_confirmed = false;
    Ok(())
}
