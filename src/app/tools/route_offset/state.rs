//! State-Definitionen und Konstruktor fuer das Strecken-Versatz-Tool.

use crate::app::tools::common::ToolLifecycleState;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;
use std::cell::RefCell;

/// Konfigurationsschluessel fuer den Preview-Cache des RouteOffset-Tools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RouteOffsetPreviewKey {
    pub left_enabled: bool,
    pub right_enabled: bool,
    pub left_distance: f32,
    pub right_distance: f32,
    pub base_spacing: f32,
}

/// Gecachte Preview-Geometrie fuer die berechneten Versatzseiten.
#[derive(Debug, Clone)]
pub(crate) struct RouteOffsetPreviewCache {
    pub chain_revision: u64,
    pub key: RouteOffsetPreviewKey,
    pub left_points: Option<Vec<Vec2>>,
    pub right_points: Option<Vec<Vec2>>,
}

/// Konfiguration fuer das RouteOffsetTool.
#[derive(Debug, Clone)]
pub struct OffsetConfig {
    /// Links-Versatz aktiviert?
    pub left_enabled: bool,
    /// Rechts-Versatz aktiviert?
    pub right_enabled: bool,
    /// Versatz-Distanz links in Metern (immer positiv)
    pub left_distance: f32,
    /// Versatz-Distanz rechts in Metern (immer positiv)
    pub right_distance: f32,
    /// Original-Kette beibehalten? false = Original-Nodes entfernen
    pub keep_original: bool,
    /// Maximaler Abstand zwischen Nodes auf der Offset-Kette
    pub base_spacing: f32,
}

impl Default for OffsetConfig {
    fn default() -> Self {
        Self {
            left_enabled: true,
            right_enabled: false,
            left_distance: 8.0,
            right_distance: 8.0,
            keep_original: true,
            base_spacing: 6.0,
        }
    }
}

/// Strecken-Versatz-Tool — generiert eine oder zwei Parallel-Versatz-Ketten
/// zur selektierten Kette ohne S-Kurven-Uebergaenge.
pub struct RouteOffsetTool {
    /// Geordnete Positionen der Quell-Kette (aus Selektion gesetzt)
    pub(crate) chain_positions: Vec<Vec2>,
    /// ID des ersten Ketten-Nodes (existenter Start-Anker)
    pub(crate) chain_start_id: u64,
    /// ID des letzten Ketten-Nodes (existenter End-Anker)
    pub(crate) chain_end_id: u64,
    /// IDs der inneren Ketten-Nodes (ohne Start/Ende) fuer "Original entfernen".
    ///
    /// Wird durch sequenzielle Inferenz in `load_chain` oder explizit via
    /// `set_chain_inner_ids` durch den Handler gesetzt.
    pub(crate) chain_inner_ids: Vec<u64>,
    /// Monotoner Revisionszaehler fuer Kettenaenderungen.
    pub(crate) chain_revision: u64,
    /// Verbindungsrichtung fuer die erzeugten Verbindungen
    pub direction: ConnectionDirection,
    /// Prioritaet fuer die erzeugten Verbindungen
    pub priority: ConnectionPriority,
    /// Tool-Konfiguration (Versatz, Original, Spacing)
    pub config: OffsetConfig,
    /// Gemeinsamer Lifecycle-Zustand (Snap-Radius, letzte IDs, Recreate-Flag)
    pub(crate) lifecycle: ToolLifecycleState,
    /// Cache fuer berechnete Preview-Offsets.
    pub(crate) preview_cache: RefCell<Option<RouteOffsetPreviewCache>>,
}

impl RouteOffsetTool {
    /// Erstellt ein neues RouteOffsetTool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            chain_positions: Vec::new(),
            chain_start_id: 0,
            chain_end_id: 0,
            chain_inner_ids: Vec::new(),
            chain_revision: 0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            config: OffsetConfig::default(),
            lifecycle: ToolLifecycleState::new(3.0),
            preview_cache: RefCell::new(None),
        }
    }

    /// Gibt `true` zurueck wenn eine gueltige Kette geladen ist (mind. 2 Punkte).
    pub fn has_chain(&self) -> bool {
        self.chain_positions.len() >= 2
    }

    /// Verwirft die gecachte Preview-Geometrie.
    pub(crate) fn invalidate_preview_cache(&self) {
        self.preview_cache.borrow_mut().take();
    }
}

impl Default for RouteOffsetTool {
    fn default() -> Self {
        Self::new()
    }
}
