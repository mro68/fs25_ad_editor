//! State-Strukturen fuer das ColorPathTool.

use super::skeleton::SkeletonNetwork;
use crate::app::tools::common::ToolLifecycleState;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;
use image::RgbImage;
use std::sync::Arc;

/// Aktuelle Phase des ColorPathTool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPathPhase {
    /// Leerer Grundzustand ohne aktives Sampling.
    Idle,
    /// User sampelt Farben per Klick oder Alt+Lasso.
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

/// Stage A: Nutzereingaben fuer die Farb-Pfad-Erkennung.
#[derive(Debug, Clone, Default)]
pub(super) struct SamplingInput {
    /// Alle Lasso-Polygone in Weltkoordinaten.
    pub lasso_regions: Vec<Vec<Vec2>>,
    /// Gesammelte RGB-Farben aus Klicks und Lasso-Regionen.
    pub sampled_colors: Vec<[u8; 3]>,
    /// Berechneter RGB-Mittelwert aller Samples fuer die UI.
    pub avg_color: Option<[u8; 3]>,
    /// Weltposition des ersten Sampling-Klicks als Flood-Fill-Seed.
    pub lasso_start_world: Option<Vec2>,
}

/// Stage B: Aufbereitete Matching-Spezifikation fuer Flood-Fill und Preview.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct MatchingSpec {
    /// Wirksame Farbtoleranz des Matchings.
    pub tolerance: f32,
    /// Aktive Farbreferenzmenge fuer Matching und Anzeige.
    pub palette: Vec<[u8; 3]>,
}

/// Bitstabile Schluessel-Repräsentation fuer eine Weltposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Vec2CacheKey {
    /// Bitmuster der X-Koordinate.
    pub x_bits: u32,
    /// Bitmuster der Y-Koordinate.
    pub y_bits: u32,
}

impl From<Vec2> for Vec2CacheKey {
    fn from(value: Vec2) -> Self {
        Self {
            x_bits: value.x.to_bits(),
            y_bits: value.y.to_bits(),
        }
    }
}

/// Cache-Key fuer die Matching-Spezifikation (Stage B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MatchingCacheKey {
    /// Revision der Sampling-Eingaben.
    pub sampling_revision: u64,
    /// Exaktmodus fuer die Farbauswahl.
    pub exact_color_match: bool,
    /// Effektive Toleranz-Konfiguration als Bitmuster.
    pub color_tolerance_bits: u32,
}

/// Cache-Key fuer Sampling-Maske und Boundary-Preview (Stage C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SamplingPreviewCacheKey {
    /// Identitaet des aktuell gebundenen Hintergrundbilds.
    pub background_image_id: usize,
    /// Aktuelle Kartenkante als Bitmuster.
    pub map_size_bits: u32,
    /// Lasso-Startpunkt in bitstabiler Form.
    pub lasso_start_world: Vec2CacheKey,
    /// Upstream-Revision der Matching-Spezifikation.
    pub matching_revision: u64,
}

/// Cache-Key fuer vorbereitete Maske und Skeleton-Netz (Stages D/E).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PreviewCoreCacheKey {
    /// Revision der Sampling-Vorschau.
    pub sampling_preview_revision: u64,
    /// Aktivierter Rauschfilter.
    pub noise_filter: bool,
}

/// Cache-Key fuer PreparedSegments (Stage F).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PreparedSegmentsCacheKey {
    /// Revision des Preview-Kerns.
    pub preview_core_revision: u64,
    /// Vereinfachung als Bitmuster.
    pub simplify_tolerance_bits: u32,
    /// Node-Abstand als Bitmuster.
    pub node_spacing_bits: u32,
    /// Kreuzungs-Radius als Bitmuster.
    pub junction_radius_bits: u32,
}

/// Lazy gecachte RGB-Sicht auf das Hintergrundbild.
#[derive(Debug, Clone)]
pub(super) struct CachedRgbImage {
    /// Identitaet des zugrunde liegenden Hintergrundbilds.
    pub background_image_id: usize,
    /// Materialisierte RGB-Pixel fuer wiederholte Flood-Fills.
    pub image: Arc<RgbImage>,
}

/// Interner Cache- und Revisionszustand fuer die ColorPath-Stages.
#[derive(Debug, Clone, Default)]
pub(super) struct ColorPathCacheState {
    /// Revision der Sampling-Eingaben (Stage A).
    pub sampling_revision: u64,
    /// Revision der Matching-Spezifikation (Stage B).
    pub matching_revision: u64,
    /// Revision der Sampling-Vorschau (Stage C).
    pub sampling_preview_revision: u64,
    /// Revision von vorbereiteter Maske und Skeleton-Netz (Stages D/E).
    pub preview_core_revision: u64,
    /// Revision der PreparedSegments (Stage F).
    pub prepared_segments_revision: u64,
    /// Letzter gueltiger Cache-Key fuer Stage B.
    pub matching_key: Option<MatchingCacheKey>,
    /// Letzter gueltiger Cache-Key fuer Stage C.
    pub sampling_preview_key: Option<SamplingPreviewCacheKey>,
    /// Letzter gueltiger Cache-Key fuer Stages D/E.
    pub preview_core_key: Option<PreviewCoreCacheKey>,
    /// Letzter gueltiger Cache-Key fuer Stage F.
    pub prepared_segments_key: Option<PreparedSegmentsCacheKey>,
    /// Lazy RGB-Cache fuer wiederholte Flood-Fills auf demselben Bild.
    pub rgb_image: Option<CachedRgbImage>,
}

impl MatchingSpec {
    /// Gibt `true` zurueck, wenn noch keine Match-Farben vorliegen.
    pub(super) fn is_empty(&self) -> bool {
        self.palette.is_empty()
    }
}

/// Gemeinsames Maskenartefakt fuer Sampling-Preview und Berechnen-Pipeline.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ColorPathMask {
    /// Bool-Maske (`true` = Pfadpixel), zeilenweise row-major.
    pub pixels: Vec<bool>,
    /// Maskenbreite in Pixeln.
    pub width: u32,
    /// Maskenhoehe in Pixeln.
    pub height: u32,
}

impl ColorPathMask {
    /// Erstellt ein neues Maskenartefakt aus Pixeln und Dimensionen.
    pub(super) fn new(pixels: Vec<bool>, width: u32, height: u32) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }

    /// Gibt `true` zurueck, wenn keine gueltige Maskengeometrie vorliegt.
    pub(super) fn is_empty(&self) -> bool {
        self.pixels.is_empty() || self.width == 0 || self.height == 0
    }
}

/// Stage C: Sampling-Maske und deren Boundary-Preview im Viewport.
#[derive(Debug, Clone, Default)]
pub(super) struct SamplingPreviewData {
    /// Flood-Fill-Maske direkt aus Sampling-Input + Matching-Spezifikation.
    pub input_mask: ColorPathMask,
    /// Randsegmente der Stage-C-Maske in Weltkoordinaten.
    pub boundary_segments: Vec<(Vec2, Vec2)>,
    /// Startpixel des Flood-Fills fuer spaetere Stage-Uebergaenge.
    pub start_pixel: (u32, u32),
}

/// Stages D-F: vorbereitete Maske, extrahiertes Netz und gemeinsame Preview-Segmente.
#[derive(Debug, Clone, Default)]
pub(super) struct PreviewData {
    /// Stage D: optional entrauschte und geschlossene Arbeitsmaske.
    #[allow(dead_code)]
    pub prepared_mask: ColorPathMask,
    /// Stage E: extrahiertes Skeleton-/Netz-Artefakt.
    pub network: SkeletonNetwork,
    /// Stage F: vereinfachte und neu abgetastete Segmente als gemeinsame Wahrheit.
    pub prepared_segments: Vec<PreparedSegment>,
}

/// Konfigurationsparameter fuer die Farb-Pfad-Erkennung.
#[derive(Clone)]
pub struct ColorPathConfig {
    /// Exakte Farbübereinstimmung gegen eine der gelasso-ten Farben verwenden.
    ///
    /// Wenn aktiv, wird die Toleranz ignoriert und nur auf exakte RGB-Treffer
    /// gegen die gelasso-ten Farben gematcht.
    pub exact_color_match: bool,
    /// Farbtoleranz fuer die Binaermaske im unscharfen Modus (Standard: 25.0, Bereich: 1–80)
    pub color_tolerance: f32,
    /// Abstand zwischen generierten Nodes in Metern (Standard: 5.0, Bereich: 1–50)
    pub node_spacing: f32,
    /// Toleranz fuer Douglas-Peucker-Vereinfachung in Metern (Standard: 1.0, Bereich: 0–20)
    pub simplify_tolerance: f32,
    /// Radius in Metern, der in Stage F rund um Junctions ausgespart wird (Standard: 0.0, Bereich: 0–100)
    pub junction_radius: f32,
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
            exact_color_match: true,
            color_tolerance: 25.0,
            node_spacing: 5.0,
            simplify_tolerance: 1.0,
            junction_radius: 0.0,
            noise_filter: true,
            detection_bounds: None,
            existing_connection_mode: ExistingConnectionMode::OpenEnds,
        }
    }
}

/// Tool zur automatischen Wege-Erkennung anhand der Farbe im Hintergrundbild.
///
/// Der User sampelt per Klick oder Alt+Lasso Farbwerte, das Tool baut daraus eine
/// Binaermaske, skelettiert via Zhang-Suen und erzeugt daraus ein Waypoint-Netz.
#[derive(Clone)]
pub struct ColorPathTool {
    /// Aktuelle Interaktionsphase
    pub(crate) phase: ColorPathPhase,
    /// Konfigurationsparameter
    pub(crate) config: ColorPathConfig,

    // ── Stage-A-bis-F-Artefakte ─────────────────────────────────────────────
    /// Stage A: rohe Nutzereingaben (Lasso, Farben, Startpunkt).
    pub(super) sampling: SamplingInput,
    /// Stage B: abgeleitete Matching-Spezifikation.
    pub(super) matching: MatchingSpec,
    /// Stage C: Pixel-Maske + Sampling-Vorschau.
    pub(super) sampling_preview: Option<SamplingPreviewData>,
    /// Stages D-F: Maskenaufbereitung, Netzextraktion und PreparedSegments.
    pub(super) preview_data: Option<PreviewData>,

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
    /// Interne Stage-Keys, Revisionen und Lazy-Caches.
    pub(super) cache: ColorPathCacheState,
}

impl Default for ColorPathTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPathTool {
    /// Erstellt ein neues ColorPathTool mit Standardwerten und aktivem Sampling.
    pub fn new() -> Self {
        Self {
            phase: ColorPathPhase::Sampling,
            config: ColorPathConfig::default(),
            sampling: SamplingInput::default(),
            matching: MatchingSpec::default(),
            sampling_preview: None,
            preview_data: None,
            background_image: None,
            map_size: 2048.0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
            cache: ColorPathCacheState::default(),
        }
    }
}
