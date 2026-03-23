//! State-Strukturen fuer das FieldBoundaryTool.

use crate::app::tools::common::ToolLifecycleState;
use crate::core::{ConnectionDirection, ConnectionPriority, FieldPolygon};
use std::sync::Arc;

/// Interaktionsphasen des FieldBoundaryTool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldBoundaryPhase {
    /// Warten auf Klick in ein Feld.
    Idle,
    /// Feldgrenze erkannt, Vorschau aktiv, Konfiguration moeglich.
    Configuring,
}

/// Feldgrenz-Erkennungs-Tool.
///
/// Klick in ein Feld → Feldumriss erkennen → Route als geschlossenen Ring erzeugen.
pub struct FieldBoundaryTool {
    pub(crate) phase: FieldBoundaryPhase,
    /// Das per Klick ausgewaehlte Feld-Polygon.
    pub(crate) selected_polygon: Option<FieldPolygon>,
    /// Alle verfuegbaren Farmland-Polygone (gesetzt beim Tool-Aktivieren).
    pub(crate) farmland_data: Option<Arc<Vec<FieldPolygon>>>,
    /// Abstand zwischen generierten Nodes in Metern.
    pub(crate) node_spacing: f32,
    /// Versatz der Route nach innen (negativ) oder aussen (positiv) in Metern.
    pub(crate) offset: f32,
    /// Toleranz fuer Douglas-Peucker-Vereinfachung in Metern (0 = keine).
    pub(crate) straighten_tolerance: f32,
    /// Ecken-Erkennung aktiviert?
    pub(crate) corner_detection_enabled: bool,
    /// Winkel-Schwellwert fuer Ecken-Erkennung in Grad (Standard: 90°).
    pub(crate) corner_angle_threshold_deg: f32,
    /// Eckenverrundung aktiviert?
    pub(crate) corner_rounding_enabled: bool,
    /// Radius der Eckenverrundung in Metern (Standard: 5.0).
    pub(crate) corner_rounding_radius: f32,
    /// Verbindungsrichtung.
    pub direction: ConnectionDirection,
    /// Strassenart.
    pub priority: ConnectionPriority,
    pub(crate) lifecycle: ToolLifecycleState,
}

impl Default for FieldBoundaryTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldBoundaryTool {
    /// Erstellt ein neues FieldBoundaryTool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            phase: FieldBoundaryPhase::Idle,
            selected_polygon: None,
            farmland_data: None,
            node_spacing: 10.0,
            offset: 0.0,
            straighten_tolerance: 0.0,
            corner_detection_enabled: false,
            corner_angle_threshold_deg: 90.0,
            corner_rounding_enabled: false,
            corner_rounding_radius: 5.0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
        }
    }
}
