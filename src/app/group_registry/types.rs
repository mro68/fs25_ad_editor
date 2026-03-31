//! Typen und Konstanten fuer die Segment-Registry.
//!
//! Enthaelt alle Datentypes (`GroupBase`, `GroupKind`, `GroupRecord`)
//! sowie die Tool-Index-Konstanten fuer den `ToolManager`.

use crate::app::tools::common::TangentSource;
use crate::app::tools::parking::ParkingConfig;
use crate::app::tools::ToolAnchor;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Gemeinsame Segment-Parameter aller Route-Tools.
#[derive(Debug, Clone)]
pub struct GroupBase {
    /// Verbindungsrichtung
    pub direction: ConnectionDirection,
    /// Strassenart
    pub priority: ConnectionPriority,
    /// Maximaler Abstand zwischen Zwischen-Nodes
    pub max_segment_length: f32,
}

/// Art des Segments — enthaelt alle tool-spezifischen Parameter.
#[derive(Debug, Clone)]
pub enum GroupKind {
    /// Gerade Strecke
    Straight {
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Kubische Bézier-Kurve (Grad 3)
    CurveCubic {
        /// Erster Steuerpunkt
        cp1: Vec2,
        /// Zweiter Steuerpunkt
        cp2: Vec2,
        /// Quell-Tangente am Startpunkt
        tangent_start: TangentSource,
        /// Quell-Tangente am Endpunkt
        tangent_end: TangentSource,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Quadratische Bézier-Kurve (Grad 2)
    CurveQuad {
        /// Steuerpunkt
        cp1: Vec2,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Catmull-Rom-Spline
    Spline {
        /// Anker-Punkte (alle geklickten Punkte inkl. Start/Ende)
        anchors: Vec<ToolAnchor>,
        /// Quell-Tangente am Startpunkt
        tangent_start: TangentSource,
        /// Quell-Tangente am Endpunkt
        tangent_end: TangentSource,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Geglättete Kurve (winkelgeglaettet mit automatischen Tangenten)
    SmoothCurve {
        /// Zwischen-Kontrollpunkte
        control_nodes: Vec<Vec2>,
        /// Maximale Richtungsaenderung pro Segment (Grad)
        max_angle_deg: f32,
        /// Minimaldistanz-Filter (Meter)
        min_distance: f32,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Ausweichstrecke zur selektierten Kette
    Bypass {
        /// Geordnete Positionen der Quell-Kette
        chain_positions: Vec<Vec2>,
        /// ID des ersten Ketten-Nodes
        chain_start_id: u64,
        /// ID des letzten Ketten-Nodes
        chain_end_id: u64,
        /// Seitlicher Versatz
        offset: f32,
        /// Node-Abstand auf der Bypass-Strecke
        base_spacing: f32,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Parkplatz-Layout (Wendekreis + Parkreihen)
    Parking {
        /// Ursprungspunkt des Layouts
        origin: Vec2,
        /// Rotationswinkel (Radiant)
        angle: f32,
        /// Parkplatz-Konfiguration
        config: ParkingConfig,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Feldgrenz-Route (geschlossener Ring entlang eines Feldes)
    FieldBoundary {
        /// Farmland-ID des verwendeten Feldes
        field_id: u32,
        /// Node-Abstand (Meter)
        node_spacing: f32,
        /// Versatz nach innen (negativ) oder aussen (positiv) in Metern
        offset: f32,
        /// Vereinfachungs-Toleranz Douglas-Peucker (Meter)
        straighten_tolerance: f32,
        /// Winkel-Schwellwert fuer Ecken-Erkennung in Grad (None = deaktiviert).
        corner_angle_threshold: Option<f32>,
        /// Verrundungsradius fuer erkannte Ecken in Metern (None = keine Verrundung).
        corner_rounding_radius: Option<f32>,
        /// Maximale Winkelabweichung zwischen Bogenpunkten in Grad (None = 15°).
        corner_rounding_max_angle_deg: Option<f32>,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Manuell gruppierte Nodes (kein Tool-Hintergrund).
    Manual {
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
    /// Parallelversatz einer selektierten Kette (ohne S-Kurven-Anbindung).
    RouteOffset {
        /// Geordnete Positionen der Quell-Kette
        chain_positions: Vec<Vec2>,
        /// ID des ersten Ketten-Nodes
        chain_start_id: u64,
        /// ID des letzten Ketten-Nodes
        chain_end_id: u64,
        /// Versatz links in Metern (0.0 = deaktiviert)
        offset_left: f32,
        /// Versatz rechts in Metern (0.0 = deaktiviert, intern immer positiv)
        offset_right: f32,
        /// Original-Kette beibehalten?
        keep_original: bool,
        /// Node-Abstand auf der Offset-Kette
        base_spacing: f32,
        /// Gemeinsame Basis-Parameter
        base: GroupBase,
    },
}

/// Richtung einer Gruppen-Grenz-Verbindung (Ein-/Ausfahrtstyp).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryDirection {
    /// Nur eingehende externe Verbindungen an diesem Node.
    Entry,
    /// Nur ausgehende externe Verbindungen an diesem Node.
    Exit,
    /// Ein- und ausgehende externe Verbindungen an diesem Node.
    Bidirectional,
}

/// Gecachte Information ueber einen Gruppen-Grenz-Node.
#[derive(Debug, Clone)]
pub struct BoundaryInfo {
    /// ID des Nodes an der Gruppengrenze.
    pub node_id: u64,
    /// true = mindestens eine Verbindung fuehrt zu einem Node ausserhalb JEDER registrierten Gruppe.
    pub has_external_connection: bool,
    /// Richtung der externen Verbindungen an diesem Node.
    pub direction: BoundaryDirection,
    /// Maximale Winkelabweichung zwischen interner Fahrtrichtung und externer Verbindung (Radiant, 0..PI).
    /// `None` wenn keine internen Verbindungen vorhanden (Winkelvergleich nicht moeglich).
    pub max_external_angle_deviation: Option<f32>,
}

/// Tool-Index fuer `StraightLineTool` im `ToolManager` (Registrierungs-Slot 0).
///
/// Muss mit der Reihenfolge in `ToolManager::new()` uebereinstimmen.
pub const TOOL_INDEX_STRAIGHT: usize = 0;
/// Tool-Index fuer `CurveTool(Grad 2)` im `ToolManager` (Registrierungs-Slot 1).
pub const TOOL_INDEX_CURVE_QUAD: usize = 1;
/// Tool-Index fuer `CurveTool(Grad 3)` im `ToolManager` (Registrierungs-Slot 2).
pub const TOOL_INDEX_CURVE_CUBIC: usize = 2;
/// Tool-Index fuer `SplineTool` im `ToolManager` (Registrierungs-Slot 3).
pub const TOOL_INDEX_SPLINE: usize = 3;
/// Tool-Index fuer `BypassTool` im `ToolManager` (Registrierungs-Slot 4).
pub const TOOL_INDEX_BYPASS: usize = 4;
/// Tool-Index fuer `SmoothCurveTool` im `ToolManager` (Registrierungs-Slot 5).
pub const TOOL_INDEX_SMOOTH_CURVE: usize = 5;
/// Tool-Index fuer `ParkingTool` im `ToolManager` (Registrierungs-Slot 6).
pub const TOOL_INDEX_PARKING: usize = 6;
/// Tool-Index fuer `FieldBoundaryTool` im `ToolManager` (Registrierungs-Slot 7).
pub const TOOL_INDEX_FIELD_BOUNDARY: usize = 7;
/// Tool-Index fuer `FieldPathTool` im `ToolManager` (Registrierungs-Slot 8).
pub const TOOL_INDEX_FIELD_PATH: usize = 8;
/// Tool-Index fuer `RouteOffsetTool` im `ToolManager` (Registrierungs-Slot 9).
pub const TOOL_INDEX_ROUTE_OFFSET: usize = 9;
/// Tool-Index fuer `ColorPathTool` im `ToolManager` (Registrierungs-Slot 10).
pub const TOOL_INDEX_COLOR_PATH: usize = 10;

impl GroupKind {
    /// Gibt den Tool-Index im ToolManager fuer dieses Segment zurueck.
    ///
    /// Muss mit der Registrierungsreihenfolge in `ToolManager::new()` uebereinstimmen —
    /// abgesichert durch den Unit-Test `tool_index_stimmt_mit_tool_manager_reihenfolge_ueberein`.
    /// Gibt `None` fuer `Manual`-Segmente zurueck, die keinem Tool zugeordnet sind.
    pub fn tool_index(&self) -> Option<usize> {
        match self {
            GroupKind::Straight { .. } => Some(TOOL_INDEX_STRAIGHT),
            GroupKind::CurveQuad { .. } => Some(TOOL_INDEX_CURVE_QUAD),
            GroupKind::CurveCubic { .. } => Some(TOOL_INDEX_CURVE_CUBIC),
            GroupKind::Spline { .. } => Some(TOOL_INDEX_SPLINE),
            GroupKind::SmoothCurve { .. } => Some(TOOL_INDEX_SMOOTH_CURVE),
            GroupKind::Bypass { .. } => Some(TOOL_INDEX_BYPASS),
            GroupKind::Parking { .. } => Some(TOOL_INDEX_PARKING),
            GroupKind::FieldBoundary { .. } => Some(TOOL_INDEX_FIELD_BOUNDARY),
            GroupKind::RouteOffset { .. } => Some(TOOL_INDEX_ROUTE_OFFSET),
            GroupKind::Manual { .. } => None,
        }
    }

    /// Gibt `true` zurueck wenn das Segment von einem Route-Tool erstellt wurde.
    pub fn is_tool_backed(&self) -> bool {
        !matches!(self, GroupKind::Manual { .. })
    }
}

/// Ein gespeichertes Segment (fertig erstellte Line).
#[derive(Debug, Clone)]
pub struct GroupRecord {
    /// Eindeutige Registry-ID (nicht identisch mit Node-IDs)
    pub id: u64,
    /// IDs aller neu erstellten Nodes dieses Segments
    pub node_ids: Vec<u64>,
    /// Start-Anker (ExistingNode oder NewPosition)
    pub start_anchor: ToolAnchor,
    /// End-Anker (ExistingNode oder NewPosition)
    pub end_anchor: ToolAnchor,
    /// Tool-spezifische Parameter
    pub kind: GroupKind,
    /// Original-Positionen der Nodes zum Zeitpunkt der Erstellung.
    /// Index-Reihenfolge entspricht `node_ids`; wird fuer Validitaetsprüfung genutzt.
    pub original_positions: Vec<Vec2>,
    /// IDs der Nodes mit Map-Markern (fuer Cleanup bei Edit).
    /// Leer bei Tools ohne Marker.
    pub marker_node_ids: Vec<u64>,
    /// Ob das Segment gesperrt ist (true = alle Nodes bewegen sich gemeinsam)
    pub locked: bool,
    /// Explizit gesetzte Einfahrt-Node-ID (None = kein Einfahrts-Icon).
    pub entry_node_id: Option<u64>,
    /// Explizit gesetzte Ausfahrt-Node-ID (None = kein Ausfahrts-Icon).
    pub exit_node_id: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tools::ToolManager;

    /// Stellt sicher, dass `tool_index()` mit der Registrierungsreihenfolge
    /// in `ToolManager::new()` uebereinstimmt. Bricht sofort beim Umbenennen
    /// oder Umsortieren der Tools.
    #[test]
    fn tool_index_stimmt_mit_tool_manager_reihenfolge_ueberein() {
        let manager = ToolManager::new();
        let names: Vec<&str> = manager.tool_names().into_iter().map(|(_, n)| n).collect();
        assert_eq!(
            names[TOOL_INDEX_STRAIGHT], "Gerade Strecke",
            "TOOL_INDEX_STRAIGHT zeigt nicht auf StraightLineTool"
        );
        assert_eq!(
            names[TOOL_INDEX_CURVE_QUAD], "Bézier Grad 2",
            "TOOL_INDEX_CURVE_QUAD zeigt nicht auf CurveTool(Grad 2)"
        );
        assert_eq!(
            names[TOOL_INDEX_CURVE_CUBIC], "Bézier Grad 3",
            "TOOL_INDEX_CURVE_CUBIC zeigt nicht auf CurveTool(Grad 3)"
        );
        assert_eq!(
            names[TOOL_INDEX_SPLINE], "Spline",
            "TOOL_INDEX_SPLINE zeigt nicht auf SplineTool"
        );
        assert_eq!(
            names[TOOL_INDEX_SMOOTH_CURVE], "Geglättete Kurve",
            "TOOL_INDEX_SMOOTH_CURVE zeigt nicht auf SmoothCurveTool"
        );
        assert_eq!(
            names[TOOL_INDEX_BYPASS], "Ausweichstrecke",
            "TOOL_INDEX_BYPASS zeigt nicht auf BypassTool"
        );
        assert_eq!(
            names[TOOL_INDEX_PARKING], "Parkplatz",
            "TOOL_INDEX_PARKING zeigt nicht auf ParkingTool"
        );
        assert_eq!(
            names[TOOL_INDEX_FIELD_BOUNDARY], "Feld erkennen",
            "TOOL_INDEX_FIELD_BOUNDARY zeigt nicht auf FieldBoundaryTool"
        );
    }

    /// Stellt sicher, dass `Manual`-Segmente keinen Tool-Index haben und nicht tool-backed sind.
    #[test]
    fn manual_segment_hat_keinen_tool_index() {
        use crate::core::{ConnectionDirection, ConnectionPriority};
        let kind = GroupKind::Manual {
            base: GroupBase {
                direction: ConnectionDirection::Dual,
                priority: ConnectionPriority::Regular,
                max_segment_length: 10.0,
            },
        };
        assert_eq!(kind.tool_index(), None);
        assert!(!kind.is_tool_backed());
    }
}
