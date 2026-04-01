//! Typen und Konstanten fuer die Segment-Registry.
//!
//! Enthaelt alle Datentypes (`GroupBase`, `GroupKind`, `GroupRecord`)
//! sowie den expliziten Tool-Vertrag fuer `GroupRecord`.

use crate::app::tool_contract::TangentSource;
use crate::app::tools::parking::ParkingConfig;
use crate::app::tools::ToolAnchor;
use crate::app::tools::{
    route_tool_descriptor, RouteToolBackingMode, RouteToolDescriptor, RouteToolId,
};
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

/// Ein gespeichertes Segment (fertig erstellte Line).
#[derive(Debug, Clone)]
pub struct GroupRecord {
    /// Eindeutige Registry-ID (nicht identisch mit Node-IDs)
    pub id: u64,
    /// Explizite Tool-Herkunft des Records (`None` fuer manuelle Gruppen).
    pub tool_id: Option<RouteToolId>,
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

impl GroupRecord {
    /// Liefert den Descriptor des zugeordneten Route-Tools.
    pub fn tool_descriptor(&self) -> Option<&'static RouteToolDescriptor> {
        self.tool_id.map(route_tool_descriptor)
    }

    /// Liefert den Backing-Modus des zugeordneten Route-Tools.
    pub fn backing_mode(&self) -> Option<RouteToolBackingMode> {
        self.tool_descriptor()
            .map(|descriptor| descriptor.backing_mode)
    }

    /// Gibt `true` zurueck wenn der Record von einem group-backed Tool stammt.
    pub fn is_tool_backed(&self) -> bool {
        self.backing_mode()
            .is_some_and(RouteToolBackingMode::is_group_backed)
    }

    /// Gibt `true` zurueck wenn der Record ueber den Tool-Edit-Flow bearbeitbar ist.
    pub fn is_tool_editable(&self) -> bool {
        self.backing_mode()
            .is_some_and(RouteToolBackingMode::is_editable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tools::route_tool_catalog;
    use crate::app::tools::ToolManager;

    /// Stellt sicher, dass Katalog und ToolManager dieselbe Reihenfolge und Namen nutzen.
    #[test]
    fn tool_catalog_stimmt_mit_tool_manager_reihenfolge_ueberein() {
        let manager = ToolManager::new();
        let entries = manager.tool_names();
        assert_eq!(entries.len(), route_tool_catalog().len());
        for ((tool_id, runtime_name), descriptor) in entries.iter().zip(route_tool_catalog()) {
            assert_eq!(*tool_id, descriptor.id);
            assert_eq!(*runtime_name, descriptor.name);
        }
    }

    /// Stellt sicher, dass manuelle Gruppen keinen Tool-Vertrag haben.
    #[test]
    fn manual_segment_hat_keinen_tool_vertrag() {
        use crate::core::{ConnectionDirection, ConnectionPriority};
        let record = GroupRecord {
            id: 1,
            tool_id: None,
            node_ids: vec![1, 2],
            start_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            end_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            kind: GroupKind::Manual {
                base: GroupBase {
                    direction: ConnectionDirection::Dual,
                    priority: ConnectionPriority::Regular,
                    max_segment_length: 10.0,
                },
            },
            original_positions: vec![Vec2::ZERO, Vec2::X],
            marker_node_ids: Vec::new(),
            locked: false,
            entry_node_id: None,
            exit_node_id: None,
        };
        assert!(record.tool_descriptor().is_none());
        assert_eq!(record.backing_mode(), None);
        assert!(!record.is_tool_backed());
        assert!(!record.is_tool_editable());
    }
}
