//! Farmland-Polygon-Verwaltung: Extraktion, JSON-I/O und Koordinatenkonversion.

use crate::app::AppState;
use crate::core::FieldPolygon;
use crate::shared::OverviewFieldDetectionSource;
use std::path::Path;
use std::sync::Arc;

pub(super) fn extract_field_polygons_from_source(
    zip_path: &str,
    savegame_dir: Option<&Path>,
    field_source: OverviewFieldDetectionSource,
) -> Option<(Vec<fs25_map_overview::FarmlandPolygon>, u32, u32)> {
    match field_source {
        OverviewFieldDetectionSource::FromZip => {
            log::info!("Feldpolygone: Quelle = infoLayer_farmlands (Map-ZIP)");
            None
        }
        OverviewFieldDetectionSource::ZipGroundGdm => {
            log::info!("Feldpolygone: Quelle = densityMap_ground.gdm (Map-ZIP)");
            fs25_map_overview::try_extract_polygons_from_zip_ground_gdm(zip_path)
        }
        OverviewFieldDetectionSource::FieldTypeGrle => savegame_dir.and_then(|dir| {
            let path = dir.join("infoLayer_fieldType.grle");
            log::info!("Feldpolygone: Quelle = {}", path.display());
            fs25_map_overview::try_extract_polygons_from_field_type_grle(&path)
        }),
        OverviewFieldDetectionSource::GroundGdm => savegame_dir.and_then(|dir| {
            let path = dir.join("densityMap_ground.gdm");
            log::info!("Feldpolygone: Quelle = {}", path.display());
            fs25_map_overview::try_extract_polygons_from_ground_gdm(&path)
        }),
    }
}

/// Berechnet den JSON-Pfad fuer Farmland-Polygone parallel zur Bilddatei.
///
/// Ersetzt die Dateiendung durch `.json` (z.B. `overview.png` → `overview.json`).
pub(super) fn json_path_for(image_path: &str) -> String {
    let p = Path::new(image_path);
    p.with_extension("json").to_string_lossy().into_owned()
}

pub(super) fn load_farmland_json_for_overview_dir(state: &mut AppState, dir: &Path) {
    let overview_path = dir.join("overview.png");
    let overview_path = overview_path.to_string_lossy().into_owned();
    load_farmland_json(state, &overview_path);
}

/// Speichert Farmland-Polygone als JSON-Datei neben der Bilddatei.
///
/// Pfad wird aus dem Bildpfad abgeleitet (z.B. `overview.jpg` → `overview.json`).
/// Falls keine Polygone vorhanden sind, wird nichts geschrieben.
pub(super) fn save_farmland_json(state: &AppState, image_path: &str) {
    let Some(ref polygons) = state.farmland_polygons else {
        return;
    };
    let json_path = json_path_for(image_path);
    match serde_json::to_string(polygons.as_ref()) {
        Ok(json) => match std::fs::write(&json_path, json) {
            Ok(()) => log::info!(
                "Farmland-Polygone gespeichert: {} ({} Felder)",
                json_path,
                polygons.len()
            ),
            Err(e) => log::warn!("Farmland-JSON konnte nicht geschrieben werden: {}", e),
        },
        Err(e) => log::warn!("Farmland-Polygone konnten nicht serialisiert werden: {}", e),
    }
}

/// Laedt Farmland-Polygone aus einer JSON-Datei neben der Bilddatei.
///
/// Prueft ob eine `.json`-Datei neben dem Bildpfad existiert und liest
/// die Polygon-Daten ein. Wird beim Auto-Load der overview.jpg aufgerufen.
pub fn load_farmland_json(state: &mut AppState, image_path: &str) {
    let json_path = json_path_for(image_path);
    let path = Path::new(&json_path);
    if !path.is_file() {
        return;
    }
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<Vec<FieldPolygon>>(&content) {
            Ok(polygons) => {
                log::info!(
                    "Farmland-Polygone geladen: {} ({} Felder)",
                    json_path,
                    polygons.len()
                );
                state.farmland_polygons = Some(Arc::new(polygons));
            }
            Err(e) => log::warn!("Farmland-JSON konnte nicht deserialisiert werden: {}", e),
        },
        Err(e) => log::warn!("Farmland-JSON konnte nicht gelesen werden: {}", e),
    }
}
