//! Explizite Render-Asset-Snapshots fuer host-neutrale Renderer.
//!
//! Dieses Modul enthaelt langlebige Asset-Daten, die nicht pro Frame neu
//! aufgebaut werden sollen. Der per-frame Vertrag `RenderScene` bleibt davon
//! getrennt und beschreibt nur die aktuelle Szene.

use image::DynamicImage;
use std::sync::Arc;

/// Weltkoordinaten-Bereich eines Background-Assets.
///
/// Die Engine beschreibt Hintergruende weiterhin im Domain-Koordinatensystem
/// X/Z. Host-Adapter koennen `min_z`/`max_z` bei Bedarf auf ihre 2D-Y-Achse
/// abbilden, bevor sie das Asset in einen Render-Core hochladen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderBackgroundWorldBounds {
    /// Linke Kante in Weltkoordinaten.
    pub min_x: f32,
    /// Rechte Kante in Weltkoordinaten.
    pub max_x: f32,
    /// Untere Kante in Weltkoordinaten (Z-Achse in der Domain).
    pub min_z: f32,
    /// Obere Kante in Weltkoordinaten (Z-Achse in der Domain).
    pub max_z: f32,
}

impl RenderBackgroundWorldBounds {
    /// Erstellt render-neutrale Bounds aus expliziten Komponenten.
    pub fn new(min_x: f32, max_x: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_z,
            max_z,
        }
    }
}

/// Snapshot eines Background-Assets fuer Renderer-Hosts.
///
/// Der Snapshot enthaelt nur read-only Daten und keinerlei GPU-Zustand. Hosts
/// entscheiden lokal, ob sie aus den Revisionen ein Upload, Update oder Clear
/// ableiten.
#[derive(Debug, Clone)]
pub struct RenderBackgroundAssetSnapshot {
    /// Hintergrundbild als geteiltes Arc-Asset.
    pub image: Arc<DynamicImage>,
    /// Weltkoordinaten des unskalierten Quads.
    pub world_bounds: RenderBackgroundWorldBounds,
    /// Aktuelle Skalierung des Background-Quads.
    pub scale: f32,
    /// Monotone Revision fuer Bildinhalt/Existenz.
    pub asset_revision: u64,
    /// Monotone Revision fuer Platzierung/Skalierung.
    pub transform_revision: u64,
}

/// Einzelner Render-Asset-Snapshot.
///
/// Der Enum ist absichtlich offen fuer zusaetzliche langlebige Assets. Aktuell
/// existiert nur die Background-Variante.
#[derive(Debug, Clone)]
pub enum RenderAssetSnapshot {
    /// Background-Asset inklusive Bild, Bounds und Revisionen.
    Background(RenderBackgroundAssetSnapshot),
}

impl RenderAssetSnapshot {
    /// Erstellt einen Render-Asset-Snapshot fuer den Background.
    pub fn background(snapshot: RenderBackgroundAssetSnapshot) -> Self {
        Self::Background(snapshot)
    }

    /// Gibt den Background-Snapshot zurueck, falls dieses Asset ein Background ist.
    pub fn as_background(&self) -> Option<&RenderBackgroundAssetSnapshot> {
        match self {
            Self::Background(snapshot) => Some(snapshot),
        }
    }
}

/// Sammlung aller expliziten Render-Assets fuer einen Host.
///
/// Hosts koennen die globalen Revisionen mit lokalem Upload-Zustand
/// vergleichen, ohne Render-Ressourcen in den `AppState` zurueckzuschreiben.
#[derive(Debug, Clone, Default)]
pub struct RenderAssetsSnapshot {
    background_asset_revision: u64,
    background_transform_revision: u64,
    assets: Vec<RenderAssetSnapshot>,
}

impl RenderAssetsSnapshot {
    /// Erstellt einen neuen Asset-Snapshot inklusive globaler Background-Revisionen.
    pub fn new(
        background_asset_revision: u64,
        background_transform_revision: u64,
        assets: Vec<RenderAssetSnapshot>,
    ) -> Self {
        Self {
            background_asset_revision,
            background_transform_revision,
            assets,
        }
    }

    /// Liefert die monotone Revision fuer Bildinhalt/Existenz des Backgrounds.
    pub fn background_asset_revision(&self) -> u64 {
        self.background_asset_revision
    }

    /// Liefert die monotone Revision fuer Bounds/Skalierung des Backgrounds.
    pub fn background_transform_revision(&self) -> u64 {
        self.background_transform_revision
    }

    /// Liefert alle enthaltenen Asset-Snapshots.
    pub fn assets(&self) -> &[RenderAssetSnapshot] {
        &self.assets
    }

    /// Liefert den Background-Asset-Snapshot, falls vorhanden.
    pub fn background(&self) -> Option<&RenderBackgroundAssetSnapshot> {
        self.assets
            .iter()
            .find_map(RenderAssetSnapshot::as_background)
    }
}
