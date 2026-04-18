//! Host-neutrale Vertrage fuer Hintergrund-Layer und Feldquellen.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Stabile Kennung eines gespeicherten Hintergrund-Layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundLayerKind {
    /// Opaque Terrain-Basis.
    Terrain,
    /// Transparente Hillshade-Schattierung.
    Hillshade,
    /// Transparente Farmland-Grenzen.
    FarmlandBorders,
    /// Transparente Farmland-ID-Beschriftungen.
    FarmlandIds,
    /// Transparente POI-Marker.
    PoiMarkers,
    /// Transparente Legende.
    Legend,
}

impl BackgroundLayerKind {
    /// Alle bekannten Hintergrund-Layer in kanonischer Reihenfolge.
    pub const ALL: [Self; 6] = [
        Self::Terrain,
        Self::Hillshade,
        Self::FarmlandBorders,
        Self::FarmlandIds,
        Self::PoiMarkers,
        Self::Legend,
    ];

    /// Gibt den kanonischen PNG-Dateinamen fuer diesen Layer zurueck.
    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Terrain => "overview_terrain.png",
            Self::Hillshade => "overview_hillshade.png",
            Self::FarmlandBorders => "overview_farmland_borders.png",
            Self::FarmlandIds => "overview_farmland_ids.png",
            Self::PoiMarkers => "overview_poi_markers.png",
            Self::Legend => "overview_legend.png",
        }
    }
}

impl fmt::Display for BackgroundLayerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Terrain => "Terrain",
            Self::Hillshade => "Hillshade",
            Self::FarmlandBorders => "Farmland Borders",
            Self::FarmlandIds => "Farmland IDs",
            Self::PoiMarkers => "POI Markers",
            Self::Legend => "Legend",
        })
    }
}

/// Host-neutrale Quelle fuer die Feldpolygon-Erkennung der Uebersichtskarte.
///
/// Gueltige serialisierte Werte: `from_zip`, `zip_ground_gdm`,
/// `field_type_grle`, `ground_gdm`.
/// Der fruehere Wert `fruits_gdm` ist seit Release 2.1.0 nicht mehr Teil
/// dieses Vertrags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverviewFieldDetectionSource {
    /// Felder aus `infoLayer_farmlands` der Map-ZIP ableiten.
    FromZip,
    /// Felder aus `densityMap_ground.gdm` innerhalb der Map-ZIP ableiten.
    #[default]
    ZipGroundGdm,
    /// Felder aus `infoLayer_fieldType.grle` des Savegames ableiten.
    FieldTypeGrle,
    /// Felder aus `densityMap_ground.gdm` des Savegames ableiten.
    GroundGdm,
}
