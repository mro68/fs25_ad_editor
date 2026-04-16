use crate::shared::{BackgroundLayerKind, OverviewLayerOptions};
use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;

/// Ein gespeicherter Hintergrund-Layer mit Pfad und dekodierten Bilddaten.
#[derive(Clone)]
pub struct StoredBackgroundLayer {
    /// Stabile Layer-Kennung fuer Runtime-Logik und Menues.
    pub kind: BackgroundLayerKind,
    /// Dateipfad des gespeicherten PNG-Layers.
    pub path: PathBuf,
    /// Bereits dekodierte Bilddaten fuer spaetere CPU-Komposition.
    pub image: Arc<DynamicImage>,
}

/// Bekannte Layer-Dateipfade im XML-Verzeichnis.
#[derive(Debug, Clone, Default)]
pub struct BackgroundLayerFiles {
    /// Verzeichnis, in dem nach den kanonischen Layer-Dateien gesucht wurde.
    pub directory: PathBuf,
    /// Opaques Terrain-Basisbild.
    pub terrain: Option<PathBuf>,
    /// Transparente Hillshade-Schattierung.
    pub hillshade: Option<PathBuf>,
    /// Transparente Farmland-Grenzen.
    pub farmland_borders: Option<PathBuf>,
    /// Transparente Farmland-ID-Beschriftungen.
    pub farmland_ids: Option<PathBuf>,
    /// Transparente POI-Marker und Labels.
    pub poi_markers: Option<PathBuf>,
    /// Transparente Legende.
    pub legend: Option<PathBuf>,
}

/// Runtime-Katalog der geladenen Layer-Bilder mit aktueller Sichtbarkeit.
#[derive(Clone)]
pub struct BackgroundLayerCatalog {
    /// Bekannte Dateipfade des gespeicherten Layer-Bundles.
    pub files: BackgroundLayerFiles,
    /// Dekodierte Bilddaten aller vorhandenen Layer in kanonischer Reihenfolge.
    pub layers: Vec<StoredBackgroundLayer>,
    /// Aktuelle Runtime-Sichtbarkeit fuer die CPU-Komposition.
    pub visible: OverviewLayerOptions,
}

/// Noch nicht persistiertes Overview-Layer-Bundle fuer den naechsten Save-Schritt.
pub struct PendingOverviewBundle {
    /// Zielverzeichnis fuer den naechsten Save-Workflow.
    pub target_dir: PathBuf,
    /// Vollstaendiges Layer-Bundle inklusive Combined-Bild und Metadaten.
    pub bundle: fs25_map_overview::OverviewLayerBundle,
}
