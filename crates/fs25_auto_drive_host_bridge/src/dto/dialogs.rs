//! Dialog-DTOs fuer die Host-Bridge.

use fs25_auto_drive_engine::app::OverviewSourceContext;
use fs25_auto_drive_engine::shared::OverviewLayerOptions;
use fs25_map_overview::FieldDetectionSource;
use serde::{Deserialize, Serialize};

/// Stabile Art eines Host-Datei-/Pfad-Dialogs fuer die Bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostDialogRequestKind {
    /// AutoDrive-XML laden.
    OpenFile,
    /// AutoDrive-XML speichern.
    SaveFile,
    /// Heightmap-Bild auswaehlen.
    Heightmap,
    /// Hintergrundbild oder ZIP auswaehlen.
    BackgroundMap,
    /// Map-Mod-ZIP fuer Overview-Generierung auswaehlen.
    OverviewZip,
    /// Curseplay-Datei importieren.
    CurseplayImport,
    /// Curseplay-Datei exportieren.
    CurseplayExport,
}

/// Serialisierbare Dialog-Anforderung fuer Hosts ohne direkten Engine-State-Zugriff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostDialogRequest {
    /// Semantische Bedeutung der Anfrage.
    pub kind: HostDialogRequestKind,
    /// Optionaler Dateiname fuer Save-Dialoge.
    pub suggested_file_name: Option<String>,
}

/// Serialisierbare Rueckmeldung eines Hosts zu einer Dialog-Anforderung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum HostDialogResult {
    /// Host-Dialog wurde ohne Auswahl geschlossen.
    Cancelled {
        /// Semantische Art der beantworteten Anfrage.
        kind: HostDialogRequestKind,
    },
    /// Host hat einen Pfad ausgewaehlt.
    PathSelected {
        /// Semantische Art der beantworteten Anfrage.
        kind: HostDialogRequestKind,
        /// Gewaehlter Pfad.
        path: String,
    },
}

/// Stabile Feldquellen fuer die Uebersichtskarten-Generierung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostFieldDetectionSource {
    /// Felder aus der Map-ZIP ableiten.
    FromZip,
    /// Felder aus `infoLayer_fieldType.grle` des Savegames ableiten.
    FieldTypeGrle,
    /// Felder aus `densityMap_ground.gdm` des Savegames ableiten.
    GroundGdm,
    /// Felder aus `densityMap_fruits.gdm` des Savegames ableiten.
    FruitsGdm,
}

impl From<FieldDetectionSource> for HostFieldDetectionSource {
    fn from(source: FieldDetectionSource) -> Self {
        match source {
            FieldDetectionSource::FromZip => Self::FromZip,
            FieldDetectionSource::FieldTypeGrle => Self::FieldTypeGrle,
            FieldDetectionSource::GroundGdm => Self::GroundGdm,
            FieldDetectionSource::FruitsGdm => Self::FruitsGdm,
        }
    }
}

/// Stabiler Kontext fuer den wiederverwendeten Overview-Source-Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostOverviewSourceContext {
    /// Dialog wurde nach dem Laden einer XML mit Auto-Erkennung geoeffnet.
    PostLoadDetected,
    /// Dialog wurde manuell ueber das Menue geoeffnet.
    ManualMenu,
}

impl From<OverviewSourceContext> for HostOverviewSourceContext {
    fn from(context: OverviewSourceContext) -> Self {
        match context {
            OverviewSourceContext::PostLoadDetected => Self::PostLoadDetected,
            OverviewSourceContext::ManualMenu => Self::ManualMenu,
        }
    }
}

/// Host-neutrale Layer-Auswahl fuer die Uebersichtskarten-Generierung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostOverviewLayersSnapshot {
    /// Hillshade-Schattierung aktiv.
    pub hillshade: bool,
    /// Farmland-Grenzen aktiv.
    pub farmlands: bool,
    /// Farmland-ID-Nummern aktiv.
    pub farmland_ids: bool,
    /// POI-Marker aktiv.
    pub pois: bool,
    /// Legende aktiv.
    pub legend: bool,
}

impl From<&OverviewLayerOptions> for HostOverviewLayersSnapshot {
    fn from(layers: &OverviewLayerOptions) -> Self {
        Self {
            hillshade: layers.hillshade,
            farmlands: layers.farmlands,
            farmland_ids: layers.farmland_ids,
            pois: layers.pois,
            legend: layers.legend,
        }
    }
}

/// Snapshot der Heightmap-Warnung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostHeightmapWarningDialogSnapshot {
    /// Ob die Warnung aktuell sichtbar ist.
    pub visible: bool,
    /// Ob die Warnung fuer den aktuellen Save-Vorgang bereits bestaetigt wurde.
    pub confirmed_for_current_save: bool,
}

/// Snapshot des Marker-Dialogs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostMarkerDialogSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Node-ID des bearbeiteten Markers.
    pub node_id: Option<u64>,
    /// Aktueller Marker-Name im Dialog.
    pub name: String,
    /// Aktuelle Marker-Gruppe im Dialog.
    pub group: String,
    /// `true`, wenn ein neuer Marker angelegt wird.
    pub is_new: bool,
}

/// Snapshot des Dedup-Bestaetigungsdialogs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostDedupDialogSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Anzahl erkannter Duplikate.
    pub duplicate_count: u32,
    /// Anzahl betroffener Positions-Gruppen.
    pub group_count: u32,
}

/// Serialisierbarer ZIP-Browser-Eintrag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostZipImageEntrySnapshot {
    /// Dateiname des ZIP-Eintrags.
    pub name: String,
    /// Unkomprimierte Dateigroesse in Bytes.
    pub size: u64,
}

/// Snapshot des ZIP-Browsers fuer die Background-Auswahl.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostZipBrowserSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Pfad der geoeffneten ZIP-Datei.
    pub zip_path: String,
    /// Verfuegbare Bilddateien im Archiv.
    pub entries: Vec<HostZipImageEntrySnapshot>,
    /// Aktuell selektierter Eintrag.
    pub selected_entry_index: Option<usize>,
    /// Ob auf Overview-Dateien gefiltert wird.
    pub filter_overview: bool,
}

/// Snapshot des Overview-Options-Dialogs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostOverviewOptionsDialogSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Pfad der gewaehlten Map-ZIP.
    pub zip_path: String,
    /// Aktuelle Layer-Auswahl.
    pub layers: HostOverviewLayersSnapshot,
    /// Aktuelle Feldquelle.
    pub field_detection_source: HostFieldDetectionSource,
    /// Verfuegbare Feldquellen fuer diesen Dialog.
    pub available_sources: Vec<HostFieldDetectionSource>,
}

/// Snapshot des wiederverwendeten Post-Load-/Overview-Source-Dialogs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostPostLoadDialogSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Kontext, aus dem der Dialog geoeffnet wurde.
    pub context: HostOverviewSourceContext,
    /// Ob eine Heightmap automatisch erkannt wurde.
    pub heightmap_set: bool,
    /// Pfad zur automatisch erkannten Heightmap.
    pub heightmap_path: Option<String>,
    /// Ob bereits ein Hintergrundbild automatisch geladen wurde.
    pub overview_loaded: bool,
    /// Kandidaten-ZIP-Dateien fuer die Uebersichtskarte.
    pub matching_zip_paths: Vec<String>,
    /// Aktuell selektierter ZIP-Index.
    pub selected_zip_index: usize,
    /// Anzeigename der Karte.
    pub map_name: String,
}

/// Snapshot des Dialogs zum Speichern als `overview.png`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSaveOverviewDialogSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Zielpfad der zu speichernden `overview.png`.
    pub target_path: String,
    /// Ob eine bestehende Datei ueberschrieben wuerde.
    pub is_overwrite: bool,
}

/// Snapshot des Dialogs fuer "Alle Felder nachzeichnen".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostTraceAllFieldsDialogSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Abstand zwischen generierten Wegpunkten.
    pub spacing: f32,
    /// Versatz vom Feldrand.
    pub offset: f32,
    /// Douglas-Peucker-Toleranz.
    pub tolerance: f32,
    /// Ob Ecken-Erkennung aktiv ist.
    pub corner_detection_enabled: bool,
    /// Winkel-Schwelle fuer Ecken-Erkennung.
    pub corner_angle_threshold_deg: f32,
    /// Ob Eckenverrundung aktiv ist.
    pub corner_rounding_enabled: bool,
    /// Radius fuer Eckenverrundung.
    pub corner_rounding_radius: f32,
    /// Maximale Winkelabweichung fuer die Verrundung.
    pub corner_rounding_max_angle_deg: f32,
}

/// Snapshot des Gruppen-Einstellungs-Popups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostGroupSettingsDialogSnapshot {
    /// Ob das Popup aktuell sichtbar ist.
    pub visible: bool,
    /// Weltposition, an der das Popup geoeffnet wurde.
    pub world_pos: [f32; 2],
    /// Ob die Segment-Selektion an Kreuzungen stoppt.
    pub segment_stop_at_junction: bool,
    /// Maximale Winkelabweichung fuer die Segment-Selektion.
    pub segment_max_angle_deg: f32,
}

/// Snapshot des Bestaetigungsdialogs zum Aufloesen einer Gruppe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostConfirmDissolveDialogSnapshot {
    /// Ob der Dialog aktuell sichtbar ist.
    pub visible: bool,
    /// Segment-ID der zu bestaetigenden Aufloesung.
    pub segment_id: Option<u64>,
}

/// Vollstaendiger host-neutraler Dialog-Snapshot fuer alle egui-Dialoge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostDialogSnapshot {
    /// Snapshot der Heightmap-Warnung.
    pub heightmap_warning: HostHeightmapWarningDialogSnapshot,
    /// Snapshot des Marker-Dialogs.
    pub marker_dialog: HostMarkerDialogSnapshot,
    /// Snapshot des Dedup-Dialogs.
    pub dedup_dialog: HostDedupDialogSnapshot,
    /// Snapshot des ZIP-Browsers.
    pub zip_browser: HostZipBrowserSnapshot,
    /// Snapshot des Overview-Options-Dialogs.
    pub overview_options_dialog: HostOverviewOptionsDialogSnapshot,
    /// Snapshot des Post-Load-/Overview-Source-Dialogs.
    pub post_load_dialog: HostPostLoadDialogSnapshot,
    /// Snapshot des Save-Overview-Dialogs.
    pub save_overview_dialog: HostSaveOverviewDialogSnapshot,
    /// Snapshot des Trace-All-Fields-Dialogs.
    pub trace_all_fields_dialog: HostTraceAllFieldsDialogSnapshot,
    /// Snapshot des Group-Settings-Popups.
    pub group_settings_popup: HostGroupSettingsDialogSnapshot,
    /// Snapshot des Confirm-Dissolve-Dialogs.
    pub confirm_dissolve_group: HostConfirmDissolveDialogSnapshot,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        HostConfirmDissolveDialogSnapshot, HostDedupDialogSnapshot, HostDialogSnapshot,
        HostFieldDetectionSource, HostGroupSettingsDialogSnapshot,
        HostHeightmapWarningDialogSnapshot, HostMarkerDialogSnapshot, HostOverviewLayersSnapshot,
        HostOverviewOptionsDialogSnapshot, HostOverviewSourceContext, HostPostLoadDialogSnapshot,
        HostSaveOverviewDialogSnapshot, HostTraceAllFieldsDialogSnapshot, HostZipBrowserSnapshot,
        HostZipImageEntrySnapshot,
    };

    #[test]
    fn host_dialog_snapshot_roundtrips_json() {
        let snapshot = HostDialogSnapshot {
            heightmap_warning: HostHeightmapWarningDialogSnapshot {
                visible: true,
                confirmed_for_current_save: false,
            },
            marker_dialog: HostMarkerDialogSnapshot {
                visible: true,
                node_id: Some(7),
                name: "Hof".to_string(),
                group: "All".to_string(),
                is_new: false,
            },
            dedup_dialog: HostDedupDialogSnapshot {
                visible: true,
                duplicate_count: 4,
                group_count: 2,
            },
            zip_browser: HostZipBrowserSnapshot {
                visible: true,
                zip_path: "/tmp/map.zip".to_string(),
                entries: vec![HostZipImageEntrySnapshot {
                    name: "overview.png".to_string(),
                    size: 4096,
                }],
                selected_entry_index: Some(0),
                filter_overview: true,
            },
            overview_options_dialog: HostOverviewOptionsDialogSnapshot {
                visible: true,
                zip_path: "/tmp/map.zip".to_string(),
                layers: HostOverviewLayersSnapshot {
                    hillshade: true,
                    farmlands: false,
                    farmland_ids: true,
                    pois: true,
                    legend: false,
                },
                field_detection_source: HostFieldDetectionSource::GroundGdm,
                available_sources: vec![
                    HostFieldDetectionSource::FromZip,
                    HostFieldDetectionSource::GroundGdm,
                ],
            },
            post_load_dialog: HostPostLoadDialogSnapshot {
                visible: true,
                context: HostOverviewSourceContext::PostLoadDetected,
                heightmap_set: true,
                heightmap_path: Some("/tmp/terrain.png".to_string()),
                overview_loaded: true,
                matching_zip_paths: vec!["/mods/map.zip".to_string()],
                selected_zip_index: 0,
                map_name: "Riverbend".to_string(),
            },
            save_overview_dialog: HostSaveOverviewDialogSnapshot {
                visible: true,
                target_path: "/tmp/overview.png".to_string(),
                is_overwrite: true,
            },
            trace_all_fields_dialog: HostTraceAllFieldsDialogSnapshot {
                visible: true,
                spacing: 8.5,
                offset: -1.25,
                tolerance: 0.75,
                corner_detection_enabled: true,
                corner_angle_threshold_deg: 92.0,
                corner_rounding_enabled: true,
                corner_rounding_radius: 5.0,
                corner_rounding_max_angle_deg: 15.0,
            },
            group_settings_popup: HostGroupSettingsDialogSnapshot {
                visible: true,
                world_pos: [12.0, 18.0],
                segment_stop_at_junction: true,
                segment_max_angle_deg: 37.5,
            },
            confirm_dissolve_group: HostConfirmDissolveDialogSnapshot {
                visible: true,
                segment_id: Some(19),
            },
        };

        let payload = serde_json::to_value(&snapshot)
            .expect("HostDialogSnapshot muss als JSON serialisierbar sein");
        assert_eq!(
            payload
                .get("overview_options_dialog")
                .and_then(|value| value.get("field_detection_source")),
            Some(&json!("ground_gdm"))
        );
        assert_eq!(
            payload
                .get("post_load_dialog")
                .and_then(|value| value.get("context")),
            Some(&json!("post_load_detected"))
        );

        let parsed: HostDialogSnapshot = serde_json::from_value(payload)
            .expect("HostDialogSnapshot muss aus JSON zuruecklesbar sein");

        assert_eq!(parsed, snapshot);
    }
}
