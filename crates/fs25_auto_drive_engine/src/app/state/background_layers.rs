use crate::shared::{BackgroundLayerKind, OverviewLayerOptions};
use std::path::PathBuf;

/// Ein gespeicherter Hintergrund-Layer als Metadaten-Eintrag.
#[derive(Clone)]
pub struct StoredBackgroundLayer {
    /// Stabile Layer-Kennung fuer Runtime-Logik und Menues.
    pub kind: BackgroundLayerKind,
    /// Dateipfad des gespeicherten PNG-Layers.
    pub path: PathBuf,
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

/// Runtime-Katalog eines gespeicherten Layer-Bundles mit aktueller Sichtbarkeit.
#[derive(Clone)]
pub struct BackgroundLayerCatalog {
    /// Bekannte Dateipfade des gespeicherten Layer-Bundles.
    pub files: BackgroundLayerFiles,
    /// Metadaten aller vorhandenen Layer in kanonischer Reihenfolge.
    ///
    /// Die PNG-Dateien werden bei Bedarf sequenziell von Platte geladen.
    pub layers: Vec<StoredBackgroundLayer>,
    /// Aktuelle Runtime-Sichtbarkeit fuer die On-Demand-CPU-Komposition.
    pub visible: OverviewLayerOptions,
}

/// Marker fuer eine noch nicht als `overview.png` gespeicherte Generierung.
///
/// Die einzelnen Layer-PNGs liegen bereits im Zielverzeichnis auf Platte.
pub struct PendingOverviewBundle {
    /// Zielverzeichnis fuer den naechsten Save-Workflow.
    pub target_dir: PathBuf,
}
