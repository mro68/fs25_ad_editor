//! Use-Cases fuer die Generierung und das Speichern von Uebersichtskarten.

use super::field_polygons::{extract_field_polygons_from_source, save_farmland_json};
use super::state_helpers::{apply_background_map, persist_overview_defaults};
use crate::app::state::PendingOverviewBundle;
use crate::app::AppState;
use crate::core::{BackgroundMap, FarmlandGrid, FieldPolygon};
use crate::shared::BackgroundLayerKind;
use anyhow::{Context, Result};
use glam::Vec2;
use image::DynamicImage;
use std::path::Path;
use std::sync::Arc;

/// Generiert eine Uebersichtskarte mit den Optionen aus dem Dialog und laedt sie als Background.
///
/// Liest ZIP-Pfad, Layer-Optionen und die gewaehlte Feldpolygon-Quelle aus dem
/// `OverviewOptionsDialogState`, persistiert die Layer-Einstellungen in den
/// `EditorOptions` und generiert die Karte mit `fs25_map_overview`.
/// Die einzelnen Layer-PNGs werden sofort persistiert; im State bleiben danach
/// nur das Preview-Bild, der Layer-Katalog und ein Pending-Marker aktiv.
pub fn generate_overview_with_options(state: &mut AppState) -> Result<()> {
    let zip_path = state.ui.overview_options_dialog.zip_path.clone();
    let layers = state.ui.overview_options_dialog.layers.clone();
    let field_source = state.ui.overview_options_dialog.field_detection_source;

    log::info!("Generiere Uebersichtskarte aus: {}", zip_path);

    // Layer-Optionen persistent speichern
    state.options.overview_layers = layers.clone();
    state.options.overview_field_detection_source = field_source;
    state.refresh_options_arc();
    persist_overview_defaults(state);

    let options = fs25_map_overview::OverviewOptions {
        terrain: layers.terrain,
        hillshade: layers.hillshade,
        farmlands: layers.farmlands,
        farmland_ids: layers.farmland_ids,
        pois: layers.pois,
        legend: layers.legend,
    };

    let bundle = fs25_map_overview::generate_overview_layer_bundle_from_zip(&zip_path, &options)?;

    let (width, height) = bundle.combined.dimensions();
    log::info!("Uebersichtskarte generiert: {}x{} Pixel", width, height);

    // Savegame-Verzeichnis (Elternordner der aktuell geladenen Config)
    let savegame_dir = state.ui.current_file_path.as_ref().and_then(|xml_path| {
        Path::new(xml_path.as_str())
            .parent()
            .map(|p| p.to_path_buf())
    });

    // Feldpolygone gemaess gewaehlter Quelle extrahieren
    let extracted =
        extract_field_polygons_from_source(&zip_path, savegame_dir.as_deref(), field_source);

    // Rohe Polygone und Rasterdimensionen ermitteln
    let (field_polygons, grle_w, grle_h) = match extracted {
        Some((polygons, w, h)) => {
            let scale_x = bundle.map_size / w.max(1) as f32;
            let scale_y = bundle.map_size / h.max(1) as f32;
            let half = bundle.map_size / 2.0;
            let polygons: Vec<FieldPolygon> = polygons
                .into_iter()
                .map(|fp| FieldPolygon {
                    id: fp.id,
                    vertices: fp
                        .vertices
                        .into_iter()
                        .map(|(px, py)| Vec2::new(px * scale_x - half, py * scale_y - half))
                        .collect(),
                })
                .collect();
            (polygons, w, h)
        }
        None => {
            let scale_x = bundle.map_size / bundle.grle_width.max(1) as f32;
            let scale_y = bundle.map_size / bundle.grle_height.max(1) as f32;
            let half = bundle.map_size / 2.0;
            let polygons: Vec<FieldPolygon> = bundle
                .farmland_polygons
                .iter()
                .map(|fp| FieldPolygon {
                    id: fp.id,
                    vertices: fp
                        .vertices
                        .iter()
                        .map(|(px, py)| Vec2::new(*px * scale_x - half, *py * scale_y - half))
                        .collect(),
                })
                .collect();
            (polygons, bundle.grle_width, bundle.grle_height)
        }
    };

    if !field_polygons.is_empty() {
        log::info!(
            "Feldpolygone in Weltkoordinaten umgerechnet: {} Felder",
            field_polygons.len()
        );
        state.farmland_polygons = Some(Arc::new(field_polygons));
    } else {
        state.farmland_polygons = None;
    }

    // FarmlandGrid aus rohen GRLE/PNG-IDs aufbauen (falls vorhanden)
    if let Some(ids) = bundle.farmland_ids_raw.clone() {
        state.farmland_grid = Some(Arc::new(FarmlandGrid::new(
            ids,
            grle_w.max(1),
            grle_h.max(1),
            bundle.map_size,
        )));
        log::info!("FarmlandGrid gespeichert: {}x{} Pixel", grle_w, grle_h);
    } else {
        state.farmland_grid = None;
    }

    let target_dir = savegame_dir.unwrap_or_else(|| {
        Path::new(&zip_path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });

    std::fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "Overview-Verzeichnis konnte nicht erstellt werden: {}",
            target_dir.display()
        )
    })?;
    write_layer_pngs_to_directory(&bundle, &target_dir)?;

    let bg_map = BackgroundMap::from_image(
        DynamicImage::ImageRgba8(bundle.combined.clone()),
        &zip_path,
        None,
    )?;
    drop(bundle);

    let files = super::super::background_layers::discover_background_layer_files(&target_dir);
    let catalog = super::super::background_layers::load_background_layer_catalog(files, &layers)?;

    apply_background_map(state, bg_map);
    state.background_layers = Some(catalog);
    log::info!(
        "Layer-PNGs gespeichert und Katalog aktiviert: {}",
        target_dir.display()
    );
    state.pending_overview_bundle = Some(PendingOverviewBundle { target_dir });

    // Dialog schliessen
    state.ui.overview_options_dialog.visible = false;

    // Speichern als overview.png anbieten (falls XML geladen)
    prompt_save_as_overview(state);

    Ok(())
}

/// Prueft ob dem User das Speichern als overview.png angeboten werden soll.
///
/// Zeigt Dialog immer an. Falls overview.png bereits existiert, wird der User
/// gefragt ob er die bestehende Datei ueberschreiben moechte.
pub(super) fn prompt_save_as_overview(state: &mut AppState) {
    let Some(ref xml_path) = state.ui.current_file_path else {
        return;
    };
    let Some(dir) = Path::new(xml_path).parent() else {
        return;
    };
    let target = dir.join("overview.png");
    let is_overwrite = target.exists();
    state.ui.save_overview_dialog.visible = true;
    state.ui.save_overview_dialog.target_path = target.to_string_lossy().into_owned();
    state.ui.save_overview_dialog.is_overwrite = is_overwrite;
    log::info!(
        "Angebot: Hintergrund als overview.png speichern in {} (ueberschreiben: {})",
        dir.display(),
        is_overwrite,
    );
}

/// Speichert die aktuelle Background-Map als overview.png (verlustfreies PNG).
///
/// Bei einem Pending-Marker sind die kanonischen Layer-Dateien bereits geschrieben.
/// Der Save-Schritt aktualisiert dann nur noch `overview.png` und `overview.json`.
pub fn save_background_as_overview(state: &mut AppState, path: String) -> Result<()> {
    if let Some(pending) = state.pending_overview_bundle.as_ref() {
        let target_dir = Path::new(&path)
            .parent()
            .map(|dir| dir.to_path_buf())
            .unwrap_or_else(|| pending.target_dir.clone());
        std::fs::create_dir_all(&target_dir).with_context(|| {
            format!(
                "Overview-Verzeichnis konnte nicht erstellt werden: {}",
                target_dir.display()
            )
        })?;
    }

    let bg_map = state
        .view
        .background_map
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Keine Background-Map geladen"))?;

    let rgb_image = bg_map.image_data().to_rgb8();
    rgb_image.save(&path)?;

    log::info!("Background-Map als overview.png gespeichert: {}", path);

    // Farmland-Polygone als JSON parallel zur Bilddatei speichern
    save_farmland_json(state, &path);

    if state.pending_overview_bundle.is_some() {
        state.pending_overview_bundle = None;
        log::info!(
            "Pending-Overview bestaetigt; Layer-Katalog bleibt aktiv: {}",
            path
        );
    }

    Ok(())
}

pub(super) fn write_layer_pngs_to_directory(
    bundle: &fs25_map_overview::OverviewLayerBundle,
    target_dir: &Path,
) -> Result<()> {
    for (kind, image) in [
        (BackgroundLayerKind::Terrain, &bundle.terrain),
        (BackgroundLayerKind::Hillshade, &bundle.hillshade),
        (
            BackgroundLayerKind::FarmlandBorders,
            &bundle.farmland_borders,
        ),
        (BackgroundLayerKind::FarmlandIds, &bundle.farmland_ids),
        (BackgroundLayerKind::PoiMarkers, &bundle.poi_markers),
        (BackgroundLayerKind::Legend, &bundle.legend),
    ] {
        let layer_path = target_dir.join(kind.file_name());
        image.save(&layer_path).with_context(|| {
            format!(
                "Overview-Layer konnte nicht gespeichert werden: {}",
                layer_path.display()
            )
        })?;
    }

    Ok(())
}
