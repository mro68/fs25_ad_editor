//! Editing-DTOs fuer Properties-, Gruppen-Edit- und Streckenteilungs-Snapshots.

use fs25_auto_drive_engine::shared::RenderQuality;
use serde::{Deserialize, Serialize};

use super::route_tool::HostRouteToolId;

/// Host-neutrale Konfigurationsart der Streckenteilung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostResampleMode {
    /// Gleiche Aufteilung ueber feste Distanz.
    Distance,
    /// Gleiche Aufteilung ueber feste Node-Anzahl.
    Count,
}

/// Host-neutrale Laufzeitoptionen fuer editing-nahe Panels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostEditingOptionsSnapshot {
    /// Aktuelle Render-Qualitaetsstufe.
    pub render_quality: RenderQuality,
    /// Ob die Hintergrundkarte aktuell sichtbar ist.
    pub background_visible: bool,
    /// Aktueller Skalierungsfaktor der Hintergrundkarte.
    pub background_scale: f32,
    /// Ob Boundary-Icons an allen Gruppen-Grenzknoten angezeigt werden sollen.
    pub show_all_group_boundaries: bool,
    /// Ob die Segment-Selektion an Kreuzungen stoppt.
    pub segment_stop_at_junction: bool,
    /// Maximale Winkelabweichung fuer die Segment-Selektion in Grad.
    pub segment_max_angle_deg: f32,
    /// Schrittweite fuer wheel-basierte Distanz-Eingaben in Metern.
    pub mouse_wheel_distance_step_m: f32,
}

/// Host-neutrale Kurzinfo ueber eine fuer die aktuelle Selektion bearbeitbare Gruppe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostEditableGroupSummary {
    /// Record-ID der Gruppe in der Registry.
    pub record_id: u64,
    /// Anzahl der zur Gruppe gehoerenden Nodes.
    pub node_count: usize,
    /// Aktueller Lock-Zustand der Gruppe.
    pub locked: bool,
    /// Optionales persistiertes Route-Tool der Gruppe.
    pub tool_id: Option<HostRouteToolId>,
    /// Gibt an, ob fuer die Gruppe ein Tool-Edit-Snapshot existiert.
    pub has_tool_edit: bool,
    /// Explizit gesetzter Einfahrts-Node der Gruppe.
    pub entry_node_id: Option<u64>,
    /// Explizit gesetzter Ausfahrts-Node der Gruppe.
    pub exit_node_id: Option<u64>,
}

/// Host-neutraler Boundary-Kandidat fuer die Gruppenbearbeitung.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostGroupBoundaryCandidateSnapshot {
    /// ID des Kandidaten-Nodes.
    pub node_id: u64,
    /// Optionale Weltposition des Kandidaten.
    pub position: Option<[f32; 2]>,
    /// Ob der Node mindestens eine eingehende Verbindung von ausserhalb hat.
    pub has_external_incoming: bool,
    /// Ob der Node mindestens eine ausgehende Verbindung nach ausserhalb hat.
    pub has_external_outgoing: bool,
}

/// Host-neutraler Snapshot des aktiven Gruppen-Edit-Modus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostGroupEditSnapshot {
    /// Record-ID der aktuell bearbeiteten Gruppe.
    pub record_id: u64,
    /// Aktueller Lock-Zustand des Records waehrend des Edits.
    pub locked: bool,
    /// Lock-Zustand des Records vor Start des Edits.
    pub was_locked_before_edit: bool,
    /// Anzahl der Nodes im aktuellen Gruppen-Record.
    pub node_count: usize,
    /// Optionales persistiertes Route-Tool der Gruppe.
    pub tool_id: Option<HostRouteToolId>,
    /// Gibt an, ob fuer die Gruppe ein Tool-Edit-Snapshot existiert.
    pub has_tool_edit: bool,
    /// Explizit gesetzter Einfahrts-Node der Gruppe.
    pub entry_node_id: Option<u64>,
    /// Explizit gesetzter Ausfahrts-Node der Gruppe.
    pub exit_node_id: Option<u64>,
    /// Boundary-relevante Kandidaten fuer Einfahrt/Ausfahrt.
    pub boundary_candidates: Vec<HostGroupBoundaryCandidateSnapshot>,
}

/// Host-neutraler Snapshot der aktuellen Streckenteilungs-Konfiguration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostResampleEditSnapshot {
    /// Ob der Resample-Vorschau-Modus aktiv ist.
    pub active: bool,
    /// Ob die aktuelle Selektion als zusammenhaengende Kette resamplebar ist.
    pub can_resample_current_selection: bool,
    /// Anzahl der aktuell selektierten Nodes.
    pub selected_node_count: usize,
    /// Aktive Konfigurationsart der Streckenteilung.
    pub mode: HostResampleMode,
    /// Gewuenschter Maximalabstand zwischen Punkten.
    pub distance: f32,
    /// Gewuenschte Gesamtanzahl von Punkten.
    pub count: u32,
    /// Berechnete Laenge der aktuellen Kette in Metern.
    pub path_length: f32,
    /// Ob die Originalstrecke waehrend der Vorschau ausgeblendet werden soll.
    pub hide_original: bool,
    /// Anzahl der aktuell berechneten Vorschau-Punkte.
    pub preview_count: usize,
}

/// Host-neutraler Sammelsnapshot fuer editing-nahe Panelzustandsdaten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostEditingSnapshot {
    /// Bearbeitbare Gruppen fuer die aktuelle Selektion.
    pub editable_groups: Vec<HostEditableGroupSummary>,
    /// Aktiver Gruppen-Edit-Snapshot, falls ein Edit-Modus laeuft.
    pub group_edit: Option<HostGroupEditSnapshot>,
    /// Aktuelle Streckenteilungs-Konfiguration.
    pub resample: HostResampleEditSnapshot,
    /// Editing-relevante host-neutrale Laufzeitoptionen.
    pub options: HostEditingOptionsSnapshot,
}

#[cfg(test)]
mod tests {
    use super::{
        HostEditableGroupSummary, HostEditingOptionsSnapshot, HostEditingSnapshot,
        HostGroupBoundaryCandidateSnapshot, HostGroupEditSnapshot, HostResampleEditSnapshot,
        HostResampleMode,
    };
    use crate::dto::HostRouteToolId;
    use fs25_auto_drive_engine::shared::RenderQuality;

    #[test]
    fn editing_snapshot_roundtrips_via_serde_json() {
        let snapshot = HostEditingSnapshot {
            editable_groups: vec![HostEditableGroupSummary {
                record_id: 7,
                node_count: 4,
                locked: true,
                tool_id: Some(HostRouteToolId::Straight),
                has_tool_edit: true,
                entry_node_id: Some(11),
                exit_node_id: Some(14),
            }],
            group_edit: Some(HostGroupEditSnapshot {
                record_id: 7,
                locked: false,
                was_locked_before_edit: true,
                node_count: 4,
                tool_id: Some(HostRouteToolId::Straight),
                has_tool_edit: true,
                entry_node_id: Some(11),
                exit_node_id: Some(14),
                boundary_candidates: vec![HostGroupBoundaryCandidateSnapshot {
                    node_id: 11,
                    position: Some([10.0, 20.0]),
                    has_external_incoming: true,
                    has_external_outgoing: false,
                }],
            }),
            resample: HostResampleEditSnapshot {
                active: true,
                can_resample_current_selection: true,
                selected_node_count: 4,
                mode: HostResampleMode::Count,
                distance: 6.5,
                count: 9,
                path_length: 42.0,
                hide_original: true,
                preview_count: 9,
            },
            options: HostEditingOptionsSnapshot {
                render_quality: RenderQuality::Medium,
                background_visible: true,
                background_scale: 1.25,
                show_all_group_boundaries: true,
                segment_stop_at_junction: false,
                segment_max_angle_deg: 37.5,
                mouse_wheel_distance_step_m: 0.5,
            },
        };

        let json =
            serde_json::to_string(&snapshot).expect("Editing-Snapshot muss serialisierbar sein");
        let parsed: HostEditingSnapshot =
            serde_json::from_str(&json).expect("Editing-Snapshot muss parsebar bleiben");

        assert_eq!(parsed, snapshot);
        assert!(json.contains("editable_groups"));
        assert!(json.contains("count"));
    }
}
