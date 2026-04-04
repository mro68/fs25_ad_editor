//! State-Strukturen fuer das FieldPathTool.

use crate::app::tools::common::ToolLifecycleState;
use crate::core::{
    ConnectionDirection, ConnectionPriority, FarmlandGrid, FieldPolygon, VoronoiGrid,
};
use glam::Vec2;
use std::sync::Arc;

/// Modus der Feldweg-Erkennung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPathMode {
    /// Felder auswaehlen (ganze Farmland-Polygone pro Seite)
    Fields,
    /// Feldgrenzen auswaehlen (Grenz-Segmente pro Seite)
    Boundaries,
}

/// Aktuelle Phase des FieldPathTool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPathPhase {
    /// Warten auf Nutzerinteraktion (Startphase)
    Idle,
    /// Felder oder Grenzsegmente fuer Seite 1 sammeln
    SelectingSide1,
    /// Felder oder Grenzsegmente fuer Seite 2 sammeln
    SelectingSide2,
    /// Berechnung abgeschlossen — Vorschau der Mittellinie aktiv
    Preview,
}

/// Konfigurationsparameter fuer die Feldweg-Erkennung.
pub struct FieldPathConfig {
    /// Abstand zwischen generierten Nodes in Metern (Standard: 5.0)
    pub node_spacing: f32,
    /// Toleranz fuer Douglas-Peucker-Vereinfachung in Metern (Standard: 1.0)
    pub simplify_tolerance: f32,
    /// An naechste bestehende Nodes anschliessen (Standard: true)
    pub connect_to_existing: bool,
}

impl Default for FieldPathConfig {
    fn default() -> Self {
        Self {
            node_spacing: 5.0,
            simplify_tolerance: 1.0,
            connect_to_existing: true,
        }
    }
}

/// Tool zur automatischen Feldweg-Erkennung zwischen zwei Seiten.
///
/// Berechnet die Mittellinie eines Korridors zwischen zwei Gruppen von
/// Farmland-Polygonen oder Feldgrenz-Segmenten via Voronoi-BFS.
pub struct FieldPathTool {
    /// Aktueller Erkennungsmodus (Felder oder Grenzen)
    pub(crate) mode: FieldPathMode,
    /// Aktuelle Interaktionsphase
    pub(crate) phase: FieldPathPhase,
    /// Konfigurationsparameter
    pub(crate) config: FieldPathConfig,

    /// Ausgewaehlte Farmland-IDs fuer Seite 1 (Feld-Modus)
    pub(crate) side1_field_ids: Vec<u32>,
    /// Ausgewaehlte Farmland-IDs fuer Seite 2 (Feld-Modus)
    pub(crate) side2_field_ids: Vec<u32>,

    /// Ausgewaehlte Grenzsegmente fuer Seite 1 (Grenz-Modus)
    pub(crate) side1_segments: Vec<Vec<Vec2>>,
    /// Ausgewaehlte Grenzsegmente fuer Seite 2 (Grenz-Modus)
    pub(crate) side2_segments: Vec<Vec<Vec2>>,

    /// Vereinfachte Mittellinie in Weltkoordinaten
    pub(crate) centerline: Vec<Vec2>,
    /// Gleichmaessig abgetastete Nodes der Mittellinie
    pub(crate) resampled_nodes: Vec<Vec2>,

    /// Cache fuer Voronoi-BFS (wird bei Grid-Aenderung invalidiert)
    pub(crate) voronoi_cache: Option<Arc<VoronoiGrid>>,

    /// Farmland-Raster fuer Pixel-basierte Analyse
    pub(crate) farmland_grid: Option<Arc<FarmlandGrid>>,
    /// Farmland-Polygone fuer Punkt-in-Polygon- und Grenz-Abfragen
    pub(crate) farmland_polygons: Option<Arc<Vec<FieldPolygon>>>,
    /// Optionales Hintergrundbild (aktuell ungenutzt, reserviert fuer spaeteren Einsatz)
    pub(crate) background_image: Option<Arc<image::DynamicImage>>,

    /// Verbindungsrichtung der erzeugten Route
    pub direction: ConnectionDirection,
    /// Strassenkategorisierung der erzeugten Route
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (letzte IDs, End-Anker, Recreate-Flag, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
}

impl Default for FieldPathTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldPathTool {
    /// Erstellt ein neues FieldPathTool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            mode: FieldPathMode::Fields,
            phase: FieldPathPhase::Idle,
            config: FieldPathConfig::default(),
            side1_field_ids: Vec::new(),
            side2_field_ids: Vec::new(),
            side1_segments: Vec::new(),
            side2_segments: Vec::new(),
            centerline: Vec::new(),
            resampled_nodes: Vec::new(),
            voronoi_cache: None,
            farmland_grid: None,
            farmland_polygons: None,
            background_image: None,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
        }
    }
}
