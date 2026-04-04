//! Tool-spezifische Edit-Payloads ausserhalb der Registry.

use crate::app::tool_contract::{TangentSource, ToolAnchor};
use crate::app::tools::parking::ParkingConfig;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Gemeinsame Route-Basisdaten fuer persistierbare Tools.
#[derive(Debug, Clone)]
pub struct ToolRouteBase {
    /// Verbindungsrichtung neuer Verbindungen.
    pub direction: ConnectionDirection,
    /// Prioritaet neuer Verbindungen.
    pub priority: ConnectionPriority,
    /// Maximale Segmentlaenge fuer Resampling.
    pub max_segment_length: f32,
}

/// Start- und Endanker eines persistierbaren Tools.
#[derive(Debug, Clone, Copy)]
pub struct ToolEditAnchors {
    /// Startanker des Tools.
    pub start: ToolAnchor,
    /// Endanker des Tools.
    pub end: ToolAnchor,
}

impl ToolEditAnchors {
    /// Liefert die zu schuetzenden ExistingNode-Anker-IDs.
    pub fn protected_node_ids(self) -> Vec<u64> {
        [self.start, self.end]
            .into_iter()
            .filter_map(|anchor| match anchor {
                ToolAnchor::ExistingNode(node_id, _) => Some(node_id),
                ToolAnchor::NewPosition(_) => None,
            })
            .collect()
    }
}

/// Gruppenneutrale Default-Werte fuer einen Session-Record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupRecordDefaults {
    /// Ob die Gruppe initial gesperrt angelegt wird.
    pub locked: bool,
    /// Optionaler Entry-Node des Session-Records.
    pub entry_node_id: Option<u64>,
    /// Optionaler Exit-Node des Session-Records.
    pub exit_node_id: Option<u64>,
}

/// Tool-spezifischer Edit-Snapshot fuer gruppenbasierte Route-Tools.
#[derive(Debug, Clone)]
pub enum RouteToolEditPayload {
    /// Persistenzdaten fuer das Gerade-Strecke-Tool.
    Straight {
        /// Start- und Endanker der Strecke.
        anchors: ToolEditAnchors,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer die quadratische Kurve.
    CurveQuad {
        /// Start- und Endanker der Kurve.
        anchors: ToolEditAnchors,
        /// Erster Kontrollpunkt.
        cp1: Vec2,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer die kubische Kurve.
    CurveCubic {
        /// Start- und Endanker der Kurve.
        anchors: ToolEditAnchors,
        /// Erster Kontrollpunkt.
        cp1: Vec2,
        /// Zweiter Kontrollpunkt.
        cp2: Vec2,
        /// Gewaehlte Tangente am Start.
        tangent_start: TangentSource,
        /// Gewaehlte Tangente am Ende.
        tangent_end: TangentSource,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer das Spline-Tool.
    Spline {
        /// Vollstaendige Spline-Ankerliste.
        anchors: Vec<ToolAnchor>,
        /// Gewaehlte Tangente am Start.
        tangent_start: TangentSource,
        /// Gewaehlte Tangente am Ende.
        tangent_end: TangentSource,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer geglaettete Kurven.
    SmoothCurve {
        /// Start- und Endanker der Kurve.
        anchors: ToolEditAnchors,
        /// Kontrollpunkte des geglaetteten Pfads.
        control_nodes: Vec<Vec2>,
        /// Maximaler Winkel pro Segment in Grad.
        max_angle_deg: f32,
        /// Minimalabstand fuer Kontrollpunkte.
        min_distance: f32,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer die Ausweichstrecke.
    Bypass {
        /// Geordnete Kettenpositionen der Quelle.
        chain_positions: Vec<Vec2>,
        /// Start-ID der Quellkette.
        chain_start_id: u64,
        /// End-ID der Quellkette.
        chain_end_id: u64,
        /// Seitlicher Versatz.
        offset: f32,
        /// Abtastabstand der erzeugten Strecke.
        base_spacing: f32,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer das Parkplatz-Tool.
    Parking {
        /// Ursprung des Layouts.
        origin: Vec2,
        /// Rotationswinkel des Layouts.
        angle: f32,
        /// Parkplatz-Konfiguration.
        config: ParkingConfig,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer die Feldgrenz-Erkennung.
    FieldBoundary {
        /// ID des bearbeiteten Feldes.
        field_id: u32,
        /// Node-Abstand entlang des Rings.
        node_spacing: f32,
        /// Offset des Rings.
        offset: f32,
        /// Toleranz fuer Geradenbegradigung.
        straighten_tolerance: f32,
        /// Optionaler Eckenschwellwert.
        corner_angle_threshold: Option<f32>,
        /// Optionaler Verrundungsradius.
        corner_rounding_radius: Option<f32>,
        /// Optionaler Maximalwinkel fuer Verrundungsboegen.
        corner_rounding_max_angle_deg: Option<f32>,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
    /// Persistenzdaten fuer das Strecken-Versatz-Tool.
    RouteOffset {
        /// Geordnete Kettenpositionen der Quelle.
        chain_positions: Vec<Vec2>,
        /// Start-ID der Quellkette.
        chain_start_id: u64,
        /// End-ID der Quellkette.
        chain_end_id: u64,
        /// Linker Versatz in Metern.
        offset_left: f32,
        /// Rechter Versatz in Metern.
        offset_right: f32,
        /// Ob die Originalstrecke erhalten bleibt.
        keep_original: bool,
        /// Abtastabstand der neuen Strecke.
        base_spacing: f32,
        /// Gemeinsame Routing-Basiswerte.
        base: ToolRouteBase,
    },
}

impl RouteToolEditPayload {
    /// Liefert die ExistingNode-Anker, die beim Tool-Edit nicht geloescht werden duerfen.
    pub fn protected_anchor_ids(&self) -> Vec<u64> {
        match self {
            Self::Straight { anchors, .. }
            | Self::CurveQuad { anchors, .. }
            | Self::CurveCubic { anchors, .. }
            | Self::SmoothCurve { anchors, .. } => anchors.protected_node_ids(),
            Self::Spline { anchors, .. } => match (anchors.first(), anchors.last()) {
                (Some(start), Some(end)) => ToolEditAnchors {
                    start: *start,
                    end: *end,
                }
                .protected_node_ids(),
                _ => Vec::new(),
            },
            Self::Bypass {
                chain_start_id,
                chain_end_id,
                ..
            }
            | Self::RouteOffset {
                chain_start_id,
                chain_end_id,
                ..
            } => vec![*chain_start_id, *chain_end_id],
            Self::Parking { .. } | Self::FieldBoundary { .. } => Vec::new(),
        }
    }

    /// Liefert gruppenneutrale Default-Werte fuer den Session-Record.
    pub fn group_record_defaults(&self, node_ids: &[u64]) -> GroupRecordDefaults {
        match self {
            Self::Parking { .. } => GroupRecordDefaults {
                locked: true,
                entry_node_id: node_ids.get(6).copied(),
                exit_node_id: node_ids.last().copied(),
            },
            _ => GroupRecordDefaults {
                locked: true,
                entry_node_id: None,
                exit_node_id: None,
            },
        }
    }
}
