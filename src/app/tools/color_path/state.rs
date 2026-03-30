//! State-Strukturen fuer das ColorPathTool.

use crate::app::tools::common::ToolLifecycleState;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;
use std::sync::Arc;

/// Aktuelle Phase des ColorPathTool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPathPhase {
    /// Warten auf Nutzerinteraktion
    Idle,
    /// User sampelt Farben per Alt+Lasso
    Sampling,
    /// Mittellinie berechnet, wird als Vorschau angezeigt
    Preview,
}

/// Konfigurationsparameter fuer die Farb-Pfad-Erkennung.
pub struct ColorPathConfig {
    /// Farbtoleranz fuer die Binärmaske (Standard: 25.0, Bereich: 5–80)
    pub color_tolerance: f32,
    /// Abstand zwischen generierten Nodes in Metern (Standard: 5.0, Bereich: 1–50)
    pub node_spacing: f32,
    /// Toleranz fuer Douglas-Peucker-Vereinfachung in Metern (Standard: 1.0, Bereich: 0–20)
    pub simplify_tolerance: f32,
    /// Rauschfilter aktivieren (Standard: true)
    pub noise_filter: bool,
    /// Optionaler Erkennungsbereich — None = gesamtes Bild
    #[allow(dead_code)] // Geplantes Feature: Rect-Begrenzung fuer die Erkennung
    pub detection_bounds: Option<(Vec2, Vec2)>,
    /// An naechste bestehende Nodes anschliessen (Standard: true)
    pub connect_to_existing: bool,
}

impl Default for ColorPathConfig {
    fn default() -> Self {
        Self {
            color_tolerance: 25.0,
            node_spacing: 5.0,
            simplify_tolerance: 1.0,
            noise_filter: true,
            detection_bounds: None,
            connect_to_existing: true,
        }
    }
}

/// Tool zur automatischen Wege-Erkennung anhand der Farbe im Hintergrundbild.
///
/// Der User sampelt per Alt+Lasso Farbwerte, das Tool baut daraus eine
/// Binärmaske, skelettiert via Zhang-Suen und erzeugt daraus Waypoint-Nodes.
pub struct ColorPathTool {
    /// Aktuelle Interaktionsphase
    pub(crate) phase: ColorPathPhase,
    /// Konfigurationsparameter
    pub(crate) config: ColorPathConfig,

    // ── Farb-Daten ──────────────────────────────────────────────────────────
    /// Alle Lasso-Polygone (Weltkoordinaten)
    pub(crate) lasso_regions: Vec<Vec<Vec2>>,
    /// Gesammelte RGB-Farben aus den Lasso-Regionen
    pub(crate) sampled_colors: Vec<[u8; 3]>,
    /// Berechneter RGB-Mittelwert aller Samples
    pub(crate) avg_color: Option<[u8; 3]>,

    // ── Berechnung ──────────────────────────────────────────────────────────
    /// Bool-Maske (true = Pfadpixel), zeilenweise row-major
    pub(crate) mask: Vec<bool>,
    /// Maskenbreite in Pixeln
    pub(crate) mask_width: u32,
    /// Maskenhoehe in Pixeln
    pub(crate) mask_height: u32,
    /// Alle gefundenen Skelett-Pfade in Weltkoordinaten
    pub(crate) skeleton_paths: Vec<Vec<Vec2>>,
    /// Vom User ausgewaehlter Pfad-Index
    pub(crate) selected_path_index: Option<usize>,
    /// Vereinfachte Mittellinie in Weltkoordinaten
    pub(crate) centerline: Vec<Vec2>,
    /// Gleichmaessig abgetastete Nodes der Mittellinie
    pub(crate) resampled_nodes: Vec<Vec2>,
    /// Weltposition des ersten Lasso-Klickpunkts (erster Polygon-Punkt des ersten Lassos).
    /// Wird verwendet um den relevanten Pfad-Bereich auszuwaehlen.
    pub(crate) lasso_start_world: Option<Vec2>,

    // ── Shared ──────────────────────────────────────────────────────────────
    /// Hintergrundbild fuer die Farberkennung
    pub(crate) background_image: Option<Arc<image::DynamicImage>>,
    /// Weltgroesse der Karte in Metern (wird aus FarmlandGrid abgeleitet)
    pub(crate) map_size: f32,
    /// Verbindungsrichtung der erzeugten Route
    pub direction: ConnectionDirection,
    /// Strassenkategorisierung der erzeugten Route
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (letzte IDs, End-Anker, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
}

impl Default for ColorPathTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPathTool {
    /// Erstellt ein neues ColorPathTool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            phase: ColorPathPhase::Idle,
            config: ColorPathConfig::default(),
            lasso_regions: Vec::new(),
            sampled_colors: Vec::new(),
            avg_color: None,
            mask: Vec::new(),
            mask_width: 0,
            mask_height: 0,
            skeleton_paths: Vec::new(),
            selected_path_index: None,
            centerline: Vec::new(),
            resampled_nodes: Vec::new(),
            lasso_start_world: None,
            background_image: None,
            map_size: 2048.0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
        }
    }
}
