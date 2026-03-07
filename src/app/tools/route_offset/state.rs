//! State-Definitionen und Konstruktor fuer das Strecken-Versatz-Tool.

use crate::app::tools::common::ToolLifecycleState;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

type CachedPreview = (Vec<Vec2>, Vec<(usize, usize)>);

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
    /// Verbindungsrichtung fuer die erzeugten Verbindungen
    pub direction: ConnectionDirection,
    /// Prioritaet fuer die erzeugten Verbindungen
    pub priority: ConnectionPriority,
    /// Tool-Konfiguration (Versatz, Original, Spacing)
    pub config: OffsetConfig,
    /// Gecachte Preview-Daten: (Nodes, Connections) — None = Cache ungueltig
    pub(crate) cached_preview: Option<CachedPreview>,
    /// Gemeinsamer Lifecycle-Zustand (Snap-Radius, letzte IDs, Recreate-Flag)
    pub(crate) lifecycle: ToolLifecycleState,
}

impl RouteOffsetTool {
    /// Erstellt ein neues RouteOffsetTool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            chain_positions: Vec::new(),
            chain_start_id: 0,
            chain_end_id: 0,
            chain_inner_ids: Vec::new(),
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            config: OffsetConfig::default(),
            cached_preview: None,
            lifecycle: ToolLifecycleState::new(3.0),
        }
    }

    /// Gibt `true` zurueck wenn eine gueltige Kette geladen ist (mind. 2 Punkte).
    pub fn has_chain(&self) -> bool {
        self.chain_positions.len() >= 2
    }
}

impl Default for RouteOffsetTool {
    fn default() -> Self {
        Self::new()
    }
}
