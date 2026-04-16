//! Rendering- und Darstellungswerte fuer Nodes, Verbindungen und Marker.

use serde::{Deserialize, Serialize};

/// Standard-Terrain-Hoehenskala (FS25: normalized_pixel x Faktor = Y-Meter).
pub const TERRAIN_HEIGHT_SCALE: f32 = 255.0;

/// Groessenfaktor fuer selektierte Nodes in Prozent.
pub const SELECTION_SIZE_FACTOR: f32 = 175.0;

/// Standard-Node-Groesse in Welteinheiten.
pub const NODE_SIZE_WORLD: f32 = 1.0000001;
/// Standard-Farbe normaler Nodes (RGBA: Blau).
pub const NODE_COLOR_DEFAULT: [f32; 4] = [0.11764706, 1.0, 0.0, 1.0];
/// Farbe fuer Sub-Prioritaets-Nodes (RGBA: Gelborange).
pub const NODE_COLOR_SUBPRIO: [f32; 4] = [0.0, 0.7490196, 1.0, 1.0];
/// Farbe fuer selektierte Nodes (RGBA: Violett).
pub const NODE_COLOR_SELECTED: [f32; 4] = [1.0, 0.0, 0.8235294, 1.0];
/// Farbe fuer Nodes mit Warnungen (RGBA: Rot).
pub const NODE_COLOR_WARNING: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

/// Linienstaerke normaler Verbindungen in Welteinheiten.
pub const CONNECTION_THICKNESS_WORLD: f32 = 0.1;
/// Linienstaerke fuer Sub-Prioritaets-Verbindungen.
pub const CONNECTION_THICKNESS_SUBPRIO_WORLD: f32 = 0.05;
/// Pfeil-Laenge in Welteinheiten.
pub const ARROW_LENGTH_WORLD: f32 = 1.0;
/// Pfeil-Breite in Welteinheiten.
pub const ARROW_WIDTH_WORLD: f32 = 0.5;
/// Farbe fuer regulaere (Einrichtungs-)Verbindungen (RGBA: Blau).
pub const CONNECTION_COLOR_REGULAR: [f32; 4] = [0.0, 0.69411767, 1.0, 1.0];
/// Farbe fuer bidirektionale (Dual-)Verbindungen (RGBA: Hellgruen).
pub const CONNECTION_COLOR_DUAL: [f32; 4] = [0.8901961, 1.0, 0.39607844, 1.0];
/// Farbe fuer Rueckwaerts-Verbindungen (RGBA: Orange).
pub const CONNECTION_COLOR_REVERSE: [f32; 4] = [1.0, 0.5, 0.1, 1.0];

/// Marker-Groesse in Welteinheiten.
pub const MARKER_SIZE_WORLD: f32 = 5.6;

/// Standard-Maximum fuer den Zoom-Kompensationsfaktor.
///
/// Bei `1.0` ist die Kompensation deaktiviert (keine Vergroesserung beim Herauszoomen).
pub const DEFAULT_ZOOM_COMPENSATION_MAX: f32 = 5.0;

/// Mindestgroesse fuer Nodes in Pixeln beim Zoomout (0.0 = deaktiviert).
pub const MIN_NODE_SIZE_PX: f32 = 4.0;
/// Mindestbreite fuer Verbindungslinien in Pixeln beim Zoomout (0.0 = deaktiviert).
pub const MIN_CONNECTION_WIDTH_PX: f32 = 1.0;
/// Mindestgroesse fuer Richtungspfeile in Pixeln beim Zoomout (0.0 = deaktiviert).
pub const MIN_ARROW_SIZE_PX: f32 = 3.0;
/// Mindestgroesse fuer Marker-Pins in Pixeln beim Zoomout (0.0 = deaktiviert).
pub const MIN_MARKER_SIZE_PX: f32 = 10.0;
/// Mindestabstand zwischen Nodes in Pixeln fuer Grid-Decimation (0.0 = deaktiviert).
pub const NODE_DECIMATION_SPACING_PX: f32 = 10.0;

/// Fuellfarbe der Map-Marker (RGBA: Dunkelgruen).
pub const MARKER_COLOR: [f32; 4] = [1.0, 0.49411765, 0.0, 1.0];
/// Outline-Farbe der Map-Marker (RGBA: Goldgelb).
pub const MARKER_OUTLINE_COLOR: [f32; 4] = [1.0, 0.6431373, 0.0, 1.0];
/// Standard-Umrissstärke für Map-Marker (Anteil am Radius, 0.01–0.3).
pub const MARKER_OUTLINE_WIDTH: f32 = 0.2;

/// Darstellungsmodus fuer selektierte Nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SelectionStyle {
    /// Selektierte Nodes erhalten einen farbigen Ring am Rand.
    #[default]
    Ring,
    /// Selektierte Nodes werden als Farbverlauf (Mitte -> Rand) dargestellt.
    Gradient,
}

/// Konfigurierbare Layer-Optionen fuer die Uebersichtskarten-Generierung.
/// Wird als Teil der `EditorOptions` persistent gespeichert.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct OverviewLayerOptions {
    /// Terrain-Basisbild einbeziehen
    pub terrain: bool,
    /// Hillshade-Schattierung anwenden
    pub hillshade: bool,
    /// Farmland-Grenzen einzeichnen
    pub farmlands: bool,
    /// Farmland-ID-Nummern einzeichnen
    pub farmland_ids: bool,
    /// POI-Marker einzeichnen
    pub pois: bool,
    /// Legende einzeichnen
    pub legend: bool,
}

impl Default for OverviewLayerOptions {
    fn default() -> Self {
        Self {
            terrain: true,
            hillshade: true,
            farmlands: true,
            farmland_ids: true,
            pois: false,
            legend: false,
        }
    }
}
