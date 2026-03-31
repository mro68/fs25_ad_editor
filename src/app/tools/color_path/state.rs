//! State-Strukturen fuer das ColorPathTool.

use super::skeleton::SkeletonNetwork;
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
    /// Teilnetz berechnet, wird als Vorschau angezeigt
    Preview,
}

/// Wie das erkannte Netz an bestehende Nodes angeschlossen werden soll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistingConnectionMode {
    /// Nie an bestehende Nodes anschliessen.
    Never,
    /// Nur offene Segment-Enden anschliessen.
    OpenEnds,
    /// Offene Enden und Junctions anschliessen.
    OpenEndsAndJunctions,
}

impl ExistingConnectionMode {
    /// Alle UI-Optionen in stabiler Reihenfolge.
    pub const ALL: [Self; 3] = [Self::Never, Self::OpenEnds, Self::OpenEndsAndJunctions];

    /// Lesbares UI-Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Never => "Nie",
            Self::OpenEnds => "Nur offene Enden",
            Self::OpenEndsAndJunctions => "Offene Enden + Kreuzungen",
        }
    }
}

/// Aufbereitete Vorschau eines extrahierten Segments.
#[derive(Debug, Clone)]
pub(crate) struct PreparedSegment {
    /// Start-Knotenindex in `SkeletonNetwork.nodes`.
    pub start_node: usize,
    /// End-Knotenindex in `SkeletonNetwork.nodes`.
    pub end_node: usize,
    /// Vereinfachte und neu abgetastete Segmentpunkte inklusive Start/Ende.
    pub resampled_nodes: Vec<Vec2>,
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
    /// Anschlussmodus fuer bestehende Nodes (Standard: offene Enden)
    pub existing_connection_mode: ExistingConnectionMode,
}

impl Default for ColorPathConfig {
    fn default() -> Self {
        Self {
            color_tolerance: 25.0,
            node_spacing: 5.0,
            simplify_tolerance: 1.0,
            noise_filter: true,
            detection_bounds: None,
            existing_connection_mode: ExistingConnectionMode::OpenEnds,
        }
    }
}

/// Tool zur automatischen Wege-Erkennung anhand der Farbe im Hintergrundbild.
///
/// Der User sampelt per Alt+Lasso Farbwerte, das Tool baut daraus eine
/// Binaermaske, skelettiert via Zhang-Suen und erzeugt daraus ein Waypoint-Netz.
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
    /// Berechneter RGB-Mittelwert aller Samples (nur fuer die Anzeige)
    pub(crate) avg_color: Option<[u8; 3]>,
    /// Quantisierte Farbpalette aus dem Lasso (eindeutige Farb-Buckets)
    pub(crate) color_palette: Vec<[u8; 3]>,

    // ── Berechnung ──────────────────────────────────────────────────────────
    /// Bool-Maske (true = Pfadpixel), zeilenweise row-major
    pub(crate) mask: Vec<bool>,
    /// Maskenbreite in Pixeln
    pub(crate) mask_width: u32,
    /// Maskenhoehe in Pixeln
    pub(crate) mask_height: u32,
    /// Extrahiertes Teilnetz mit Junctions, offenen Enden und Segmenten.
    pub(crate) skeleton_network: Option<SkeletonNetwork>,
    /// Vereinfachte und gleichmaessig abgetastete Preview-Segmente.
    pub(crate) prepared_segments: Vec<PreparedSegment>,
    /// Weltposition des ersten Lasso-Klickpunkts (erster Polygon-Punkt des ersten Lassos).
    /// Wird verwendet um den relevanten Pfad-Bereich auszuwaehlen.
    pub(crate) lasso_start_world: Option<Vec2>,
    /// Alle Randsegmente des erkannten Flood-Fill-Bereichs in Weltkoordinaten.
    /// Wird nach jeder Lasso-Auswahl aktualisiert und als Vorschau angezeigt.
    pub(crate) flood_fill_boundary_segments: Vec<(Vec2, Vec2)>,

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
            color_palette: Vec::new(),
            mask: Vec::new(),
            mask_width: 0,
            mask_height: 0,
            skeleton_network: None,
            prepared_segments: Vec::new(),
            lasso_start_world: None,
            flood_fill_boundary_segments: Vec::new(),
            background_image: None,
            map_size: 2048.0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
        }
    }
}
