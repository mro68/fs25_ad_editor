//! Stabiler Aktions-Satz fuer die kanonische Session-Surface der Host-Bridge.

use fs25_auto_drive_engine::app::ui_contract::RouteToolPanelAction;
use fs25_auto_drive_engine::shared::{EditorOptions, RenderQuality};
use serde::{Deserialize, Serialize};

use super::dialogs::HostDialogResult;
use super::input::HostViewportInputBatch;
use super::node_details::HostNodeFlag;
use super::route_tool::{HostDefaultConnectionDirection, HostDefaultConnectionPriority};

/// Stabiler Tool-Identifier fuer Host-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostActiveTool {
    /// Standard: Nodes selektieren und verschieben.
    Select,
    /// Verbindungen zwischen Nodes erstellen.
    Connect,
    /// Neue Nodes auf der Karte platzieren.
    AddNode,
    /// Route-Tools (Linie, Parkplatz, Kurve, ...).
    Route,
}

/// Host-neutrale Tangentenquelle fuer Route-Tool-Aktionen und Read-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostTangentSource {
    /// Kein Tangenten-Vorschlag.
    None,
    /// Tangente aus bestehender Verbindung.
    Connection {
        /// ID des Nachbar-Nodes der Verbindung.
        neighbor_id: u64,
        /// Winkel der Verbindung in Radiant.
        angle: f32,
    },
}

/// Explizite Route-Tool-Action-Familie auf der kanonischen Session-Surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostRouteToolAction {
    /// Route-Tool ueber stabile Tool-ID auswaehlen.
    SelectTool {
        /// Ziel-Tool.
        tool: super::route_tool::HostRouteToolId,
    },
    /// Route-Tool mit vordefinierten Start-/Endankern aktivieren.
    SelectToolWithAnchors {
        /// Ziel-Tool.
        tool: super::route_tool::HostRouteToolId,
        /// Start-Node-ID.
        start_node_id: u64,
        /// End-Node-ID.
        end_node_id: u64,
    },
    /// Semantische Route-Tool-Panel-Aktion.
    PanelAction {
        /// Panel-Aktion des aktiven Route-Tools.
        action: RouteToolPanelAction,
    },
    /// Route-Tool-Ausfuehrung anfordern.
    Execute,
    /// Route-Tool abbrechen.
    Cancel,
    /// Route-Tool mit aktueller Konfiguration neu berechnen.
    Recreate,
    /// Tangenten-Auswahl fuer Start/Ende setzen.
    ApplyTangent {
        /// Start-Tangente.
        start: HostTangentSource,
        /// End-Tangente.
        end: HostTangentSource,
    },
    /// Klick im Route-Tool-Viewport.
    Click {
        /// Weltposition des Klicks.
        world_pos: [f32; 2],
        /// Plattformneutraler Command-Modifizierer (`Ctrl`/`Cmd`).
        ctrl: bool,
    },
    /// Tool-Lasso als geschlossenes Polygon abschliessen.
    LassoCompleted {
        /// Polygonpunkte in Weltkoordinaten.
        polygon: Vec<[f32; 2]>,
    },
    /// Drag auf Route-Tool-Steuerpunkt starten.
    DragStart {
        /// Weltposition des Starts.
        world_pos: [f32; 2],
    },
    /// Drag auf Route-Tool-Steuerpunkt aktualisieren.
    DragUpdate {
        /// Weltposition des Updates.
        world_pos: [f32; 2],
    },
    /// Drag auf Route-Tool-Steuerpunkt beenden.
    DragEnd,
    /// Route-Tool-Rotation via Alt+Scroll.
    ScrollRotate {
        /// Rotationsdelta.
        delta: f32,
    },
    /// Node-Anzahl im aktiven Route-Tool erhoehen.
    IncreaseNodeCount,
    /// Node-Anzahl im aktiven Route-Tool verringern.
    DecreaseNodeCount,
    /// Segmentlaenge im aktiven Route-Tool erhoehen.
    IncreaseSegmentLength,
    /// Segmentlaenge im aktiven Route-Tool verringern.
    DecreaseSegmentLength,
}

/// Explizite Host-Aktionen fuer die gemeinsame Bridge-Session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostSessionAction {
    /// Fordert den Host auf, einen Open-File-Dialog zu starten.
    OpenFile,
    /// Fordert Speichern unter dem aktuellen Pfad an.
    Save,
    /// Fordert einen Save-As-Dialog an.
    SaveAs,
    /// Fordert einen Heightmap-Auswahldialog an.
    RequestHeightmapSelection,
    /// Entfernt die aktuell geladene Heightmap.
    ClearHeightmap,
    /// Bestaetigt die Heightmap-Warnung und setzt den Ladepfad fort.
    ConfirmHeightmapWarning,
    /// Bricht die Heightmap-Warnung ohne Folgemutation ab.
    CancelHeightmapWarning,
    /// Fordert einen Background-Map-Auswahldialog an.
    RequestBackgroundMapSelection,
    /// Fordert den ZIP-Auswahldialog fuer die Overview-Generierung an.
    GenerateOverview,
    /// Fordert aus dem Overview-Source-Dialog den nativen ZIP-Picker an.
    BrowseOverviewZip,
    /// Uebernimmt ein ZIP direkt fuer die Overview-Generierung.
    GenerateOverviewFromZip {
        /// Pfad zur gewaehlten ZIP-Datei.
        path: String,
    },
    /// Uebernimmt eine Bilddatei aus dem ZIP-Browser als Background-Map.
    SelectZipBackgroundFile {
        /// Pfad zur ZIP-Datei.
        zip_path: String,
        /// Name des gewaelten ZIP-Eintrags.
        entry_name: String,
    },
    /// Schliesst den ZIP-Browser ohne Auswahl.
    CancelZipBrowser,
    /// Bestaetigt den Overview-Options-Dialog.
    ConfirmOverviewOptions,
    /// Bricht den Overview-Options-Dialog ab.
    CancelOverviewOptions,
    /// Schliesst den Post-Load-/Overview-Source-Dialog ohne Aktion.
    DismissPostLoadDialog,
    /// Bestaetigt das Speichern des Backgrounds als Overview.
    ConfirmSaveBackgroundAsOverview,
    /// Lehnt das Speichern des Backgrounds als Overview ab.
    DismissSaveBackgroundAsOverview,
    /// Bestaetigt die Duplikat-Bereinigung nach dem Laden.
    ConfirmDeduplication,
    /// Lehnt die Duplikat-Bereinigung nach dem Laden ab.
    CancelDeduplication,
    /// Fordert einen Curseplay-Import-Dialog an.
    CurseplayImport,
    /// Fordert einen Curseplay-Export-Dialog an.
    CurseplayExport,
    /// Setzt die Kamera auf den Standardzustand zurueck.
    ResetCamera,
    /// Zoomt eine Stufe hinein.
    ZoomIn,
    /// Zoomt eine Stufe heraus.
    ZoomOut,
    /// Passt den Viewport auf die komplette Karte ein.
    ZoomToFit,
    /// Zentriert die Kamera auf einen bestimmten Node.
    CenterOnNode {
        /// ID des Ziel-Nodes.
        node_id: u64,
    },
    /// Aendert die Render-Qualitaetsstufe.
    SetRenderQuality {
        /// Ziel-Qualitaet fuer das Rendering.
        quality: RenderQuality,
    },
    /// Schaltet die Sichtbarkeit der Background-Map um.
    ToggleBackgroundVisibility,
    /// Skaliert die Background-Map relativ zur aktuellen Groesse.
    ScaleBackground {
        /// Relativer Skalierungsfaktor.
        factor: f32,
    },
    /// Passt den Viewport auf die aktuelle Selektion ein.
    ZoomToSelectionBounds,
    /// Beendet die Anwendung.
    Exit,
    /// Schaltet die Command-Palette um.
    ToggleCommandPalette,
    /// Wechselt das aktive Editor-Tool.
    SetEditorTool {
        /// Ziel-Tool als stabiler Bridge-Identifier.
        tool: HostActiveTool,
    },
    /// Fuehrt eine explizite Route-Tool-Aktion aus.
    RouteTool {
        /// Semantische Route-Tool-Aktion.
        action: HostRouteToolAction,
    },
    /// Setzt die Default-Richtung fuer neue Verbindungen.
    SetDefaultDirection {
        /// Neue Standardrichtung.
        direction: HostDefaultConnectionDirection,
    },
    /// Setzt die Default-Prioritaet fuer neue Verbindungen.
    SetDefaultPriority {
        /// Neue Standard-Prioritaet.
        priority: HostDefaultConnectionPriority,
    },
    /// Erstellt eine einzelne Verbindung zwischen zwei Nodes.
    AddConnection {
        /// Start-Node-ID der Verbindung.
        from_id: u64,
        /// Ziel-Node-ID der Verbindung.
        to_id: u64,
        /// Richtung der neuen Verbindung.
        direction: HostDefaultConnectionDirection,
        /// Prioritaet der neuen Verbindung.
        priority: HostDefaultConnectionPriority,
    },
    /// Entfernt alle Verbindungen zwischen genau zwei Nodes.
    RemoveConnectionBetween {
        /// Erste Node-ID des Paares.
        node_a: u64,
        /// Zweite Node-ID des Paares.
        node_b: u64,
    },
    /// Aendert die Richtung einer einzelnen Verbindung.
    SetConnectionDirection {
        /// Start-Node-ID der Verbindung.
        start_id: u64,
        /// End-Node-ID der Verbindung.
        end_id: u64,
        /// Neue Richtung der Verbindung.
        direction: HostDefaultConnectionDirection,
    },
    /// Aendert die Prioritaet einer einzelnen Verbindung.
    SetConnectionPriority {
        /// Start-Node-ID der Verbindung.
        start_id: u64,
        /// End-Node-ID der Verbindung.
        end_id: u64,
        /// Neue Prioritaet der Verbindung.
        priority: HostDefaultConnectionPriority,
    },
    /// Verbindet die aktuell selektierten Nodes mit den Standard-Defaults.
    ConnectSelectedNodes,
    /// Setzt die Richtung aller Verbindungen zwischen den selektierten Nodes.
    SetAllConnectionsDirectionBetweenSelected {
        /// Neue Richtung fuer alle betroffenen Verbindungen.
        direction: HostDefaultConnectionDirection,
    },
    /// Invertiert alle Verbindungen zwischen den selektierten Nodes.
    InvertAllConnectionsBetweenSelected,
    /// Setzt die Prioritaet aller Verbindungen zwischen den selektierten Nodes.
    SetAllConnectionsPriorityBetweenSelected {
        /// Neue Prioritaet fuer alle betroffenen Verbindungen.
        priority: HostDefaultConnectionPriority,
    },
    /// Entfernt alle Verbindungen zwischen den selektierten Nodes.
    RemoveAllConnectionsBetweenSelected,
    /// Uebernimmt geaenderte Editor-Optionen.
    ApplyOptions {
        /// Vollstaendige Optionen-Payload.
        options: Box<EditorOptions>,
    },
    /// Setzt die Editor-Optionen auf Standardwerte zurueck.
    ResetOptions,
    /// Oeffnet den Optionen-Dialog.
    OpenOptionsDialog,
    /// Schliesst den Optionen-Dialog.
    CloseOptionsDialog,
    /// Fuehrt den letzten Undo-faehigen Schritt rueckgaengig aus.
    Undo,
    /// Stellt den letzten Undo-Schritt wieder her.
    Redo,
    /// Fragt die Detail-Informationen eines einzelnen Nodes fuer Legacy-JSON-Reads ab.
    QueryNodeDetails {
        /// ID des abzufragenden Nodes.
        node_id: u64,
    },
    /// Aendert den Node-Flag eines Nodes.
    SetNodeFlag {
        /// ID des Nodes.
        node_id: u64,
        /// Neuer Flag-Wert.
        flag: HostNodeFlag,
    },
    /// Oeffnet den Marker-Erstellen-Dialog fuer einen Node.
    OpenCreateMarkerDialog {
        /// Node-ID.
        node_id: u64,
    },
    /// Oeffnet den Marker-Bearbeiten-Dialog fuer einen Node.
    OpenEditMarkerDialog {
        /// Node-ID.
        node_id: u64,
    },
    /// Bricht den aktuell offenen Marker-Dialog ab.
    CancelMarkerDialog,
    /// Erstellt einen neuen Marker am angegebenen Node.
    CreateMarker {
        /// Node-ID.
        node_id: u64,
        /// Name des Markers.
        name: String,
        /// Gruppe des Markers.
        group: String,
    },
    /// Aktualisiert Name und Gruppe eines bestehenden Markers.
    UpdateMarker {
        /// Node-ID des Markers.
        node_id: u64,
        /// Neuer Name.
        name: String,
        /// Neue Gruppe.
        group: String,
    },
    /// Entfernt den Marker am angegebenen Node.
    RemoveMarker {
        /// Node-ID.
        node_id: u64,
    },
    /// Loescht alle aktuell selektierten Nodes.
    DeleteSelected,
    /// Selektiert alle Nodes der aktuellen Karte.
    SelectAll,
    /// Invertiert die aktuelle Auswahl.
    InvertSelection,
    /// Hebt die aktuelle Selektion auf.
    ClearSelection,
    /// Aktiviert das Streckenteilungs-/Resample-Panel.
    StartResampleSelection,
    /// Wendet die aktuelle Streckenteilungs-Konfiguration auf die Selektion an.
    ApplyCurrentResample,
    /// Startet den nicht-destruktiven Gruppen-Edit-Modus fuer einen Record.
    StartGroupEdit {
        /// ID des zu bearbeitenden Gruppen-Records.
        record_id: u64,
    },
    /// Uebernimmt die aktuelle Gruppen-Bearbeitung.
    ApplyGroupEdit,
    /// Bricht die aktuelle Gruppen-Bearbeitung ab.
    CancelGroupEdit,
    /// Wechselt aus dem Gruppen-Edit in den zugehoerigen Tool-Edit-Modus.
    OpenGroupEditTool {
        /// ID des zugehoerigen Gruppen-Records.
        record_id: u64,
    },
    /// Speichert die aktuelle Selektion als neue Gruppe.
    GroupSelectionAsGroup,
    /// Entfernt die selektierten Nodes aus ihrer Gruppe.
    RemoveSelectedNodesFromGroup,
    /// Setzt Einfahrts-/Ausfahrts-Nodes einer Gruppe.
    SetGroupBoundaryNodes {
        /// ID des Gruppen-Records.
        record_id: u64,
        /// Node-ID der Einfahrt.
        entry_node_id: Option<u64>,
        /// Node-ID der Ausfahrt.
        exit_node_id: Option<u64>,
    },
    /// Selektiert das Segment zwischen Kreuzungen an einer Weltposition neu.
    RecomputeNodeSegmentSelection {
        /// Weltposition der Anfrage.
        world_pos: [f32; 2],
        /// Erweitert eine bestehende Selektion statt sie zu ersetzen.
        additive: bool,
    },
    /// Schaltet den Lock-Status einer Gruppe um.
    ToggleGroupLock {
        /// ID des Segment-/Gruppen-Records.
        segment_id: u64,
    },
    /// Fordert das Aufloesen einer Gruppe an.
    DissolveGroup {
        /// ID des Segment-/Gruppen-Records.
        segment_id: u64,
    },
    /// Bestaetigt das Aufloesen einer Gruppe.
    ConfirmDissolveGroup {
        /// ID des Segment-/Gruppen-Records.
        segment_id: u64,
    },
    /// Kopiert die aktuelle Selektion in die Zwischenablage.
    CopySelection,
    /// Startet den Paste-Modus mit Vorschau.
    PasteStart,
    /// Bestaetigt die Paste-Operation an der aktuellen Position.
    PasteConfirm,
    /// Bricht den Paste-Modus ab.
    PasteCancel,
    /// Oeffnet den Dialog fuer das Nachzeichnen aller Felder.
    OpenTraceAllFieldsDialog,
    /// Bestaetigt das Nachzeichnen aller Felder mit den aktuellen Dialogwerten.
    ConfirmTraceAllFields {
        /// Abstand zwischen Pfadpunkten.
        spacing: f32,
        /// Zusatzausdehnung relativ zur Feldkante.
        offset: f32,
        /// Toleranz fuer die Erkennung.
        tolerance: f32,
        /// Optionaler Eckwinkel fuer Spezialbehandlung.
        corner_angle: Option<f32>,
        /// Optionaler Radius fuer das Abrunden von Ecken.
        corner_rounding_radius: Option<f32>,
        /// Optionaler Maximalwinkel fuer Eckrundungen.
        corner_rounding_max_angle_deg: Option<f32>,
    },
    /// Bricht den Dialog fuer das Nachzeichnen aller Felder ab.
    CancelTraceAllFields,
    /// Reicht einen Batch aus host-neutralen Viewport-Input-Events in die Session.
    SubmitViewportInput {
        /// Sequenzieller Batch von Resize-, Pointer- und Scroll-Events.
        batch: HostViewportInputBatch,
    },
    /// Uebergibt ein host-seitiges Dialog-Ergebnis an die Engine.
    SubmitDialogResult {
        /// Semantisches Ergebnis einer zuvor angeforderten Dialog-Interaktion.
        result: HostDialogResult,
    },
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::shared::RenderQuality;
    use serde_json::json;

    use super::HostSessionAction;
    use crate::dto::{
        HostDefaultConnectionDirection, HostDefaultConnectionPriority,
    };

    #[test]
    fn host_session_action_connection_family_roundtrips_json() {
        let cases = vec![
            (
                HostSessionAction::AddConnection {
                    from_id: 1,
                    to_id: 2,
                    direction: HostDefaultConnectionDirection::Dual,
                    priority: HostDefaultConnectionPriority::SubPriority,
                },
                json!({
                    "kind": "add_connection",
                    "from_id": 1,
                    "to_id": 2,
                    "direction": "dual",
                    "priority": "sub_priority"
                }),
            ),
            (
                HostSessionAction::RemoveConnectionBetween {
                    node_a: 3,
                    node_b: 4,
                },
                json!({
                    "kind": "remove_connection_between",
                    "node_a": 3,
                    "node_b": 4
                }),
            ),
            (
                HostSessionAction::SetConnectionDirection {
                    start_id: 5,
                    end_id: 6,
                    direction: HostDefaultConnectionDirection::Reverse,
                },
                json!({
                    "kind": "set_connection_direction",
                    "start_id": 5,
                    "end_id": 6,
                    "direction": "reverse"
                }),
            ),
            (
                HostSessionAction::SetConnectionPriority {
                    start_id: 7,
                    end_id: 8,
                    priority: HostDefaultConnectionPriority::Regular,
                },
                json!({
                    "kind": "set_connection_priority",
                    "start_id": 7,
                    "end_id": 8,
                    "priority": "regular"
                }),
            ),
            (
                HostSessionAction::ConnectSelectedNodes,
                json!({ "kind": "connect_selected_nodes" }),
            ),
            (
                HostSessionAction::SetAllConnectionsDirectionBetweenSelected {
                    direction: HostDefaultConnectionDirection::Regular,
                },
                json!({
                    "kind": "set_all_connections_direction_between_selected",
                    "direction": "regular"
                }),
            ),
            (
                HostSessionAction::InvertAllConnectionsBetweenSelected,
                json!({ "kind": "invert_all_connections_between_selected" }),
            ),
            (
                HostSessionAction::SetAllConnectionsPriorityBetweenSelected {
                    priority: HostDefaultConnectionPriority::SubPriority,
                },
                json!({
                    "kind": "set_all_connections_priority_between_selected",
                    "priority": "sub_priority"
                }),
            ),
            (
                HostSessionAction::RemoveAllConnectionsBetweenSelected,
                json!({ "kind": "remove_all_connections_between_selected" }),
            ),
        ];

        for (action, expected_json) in cases {
            let payload = serde_json::to_value(&action)
                .expect("Connection-HostAction muss als JSON serialisierbar sein");
            assert_eq!(payload, expected_json);

            let parsed: HostSessionAction = serde_json::from_value(payload)
                .expect("Connection-HostAction muss aus JSON zuruecklesbar sein");
            assert_eq!(parsed, action);
        }
    }

    #[test]
    fn host_session_action_parity_gap_families_roundtrip_json() {
        let cases = vec![
            (
                HostSessionAction::ClearHeightmap,
                json!({ "kind": "clear_heightmap" }),
            ),
            (
                HostSessionAction::ConfirmHeightmapWarning,
                json!({ "kind": "confirm_heightmap_warning" }),
            ),
            (
                HostSessionAction::GenerateOverviewFromZip {
                    path: "/tmp/source.zip".to_string(),
                },
                json!({
                    "kind": "generate_overview_from_zip",
                    "path": "/tmp/source.zip"
                }),
            ),
            (
                HostSessionAction::SelectZipBackgroundFile {
                    zip_path: "/tmp/background.zip".to_string(),
                    entry_name: "overview.png".to_string(),
                },
                json!({
                    "kind": "select_zip_background_file",
                    "zip_path": "/tmp/background.zip",
                    "entry_name": "overview.png"
                }),
            ),
            (
                HostSessionAction::ZoomIn,
                json!({ "kind": "zoom_in" }),
            ),
            (
                HostSessionAction::CenterOnNode { node_id: 42 },
                json!({ "kind": "center_on_node", "node_id": 42 }),
            ),
            (
                HostSessionAction::SetRenderQuality {
                    quality: RenderQuality::Medium,
                },
                json!({ "kind": "set_render_quality", "quality": "Medium" }),
            ),
            (
                HostSessionAction::ScaleBackground { factor: 2.0 },
                json!({ "kind": "scale_background", "factor": 2.0 }),
            ),
            (
                HostSessionAction::OpenCreateMarkerDialog { node_id: 7 },
                json!({ "kind": "open_create_marker_dialog", "node_id": 7 }),
            ),
            (
                HostSessionAction::CancelMarkerDialog,
                json!({ "kind": "cancel_marker_dialog" }),
            ),
            (
                HostSessionAction::InvertSelection,
                json!({ "kind": "invert_selection" }),
            ),
            (
                HostSessionAction::StartGroupEdit { record_id: 5 },
                json!({ "kind": "start_group_edit", "record_id": 5 }),
            ),
            (
                HostSessionAction::SetGroupBoundaryNodes {
                    record_id: 9,
                    entry_node_id: Some(1),
                    exit_node_id: None,
                },
                json!({
                    "kind": "set_group_boundary_nodes",
                    "record_id": 9,
                    "entry_node_id": 1,
                    "exit_node_id": null
                }),
            ),
            (
                HostSessionAction::RecomputeNodeSegmentSelection {
                    world_pos: [1.5, -2.5],
                    additive: true,
                },
                json!({
                    "kind": "recompute_node_segment_selection",
                    "world_pos": [1.5, -2.5],
                    "additive": true
                }),
            ),
            (
                HostSessionAction::ConfirmDissolveGroup { segment_id: 11 },
                json!({ "kind": "confirm_dissolve_group", "segment_id": 11 }),
            ),
            (
                HostSessionAction::ConfirmTraceAllFields {
                    spacing: 8.0,
                    offset: 0.5,
                    tolerance: 1.25,
                    corner_angle: Some(30.0),
                    corner_rounding_radius: None,
                    corner_rounding_max_angle_deg: Some(75.0),
                },
                json!({
                    "kind": "confirm_trace_all_fields",
                    "spacing": 8.0,
                    "offset": 0.5,
                    "tolerance": 1.25,
                    "corner_angle": 30.0,
                    "corner_rounding_radius": null,
                    "corner_rounding_max_angle_deg": 75.0
                }),
            ),
            (
                HostSessionAction::ConfirmDeduplication,
                json!({ "kind": "confirm_deduplication" }),
            ),
        ];

        for (action, expected_json) in cases {
            let payload = serde_json::to_value(&action)
                .expect("Parity-HostAction muss als JSON serialisierbar sein");
            assert_eq!(payload, expected_json);

            let parsed: HostSessionAction = serde_json::from_value(payload)
                .expect("Parity-HostAction muss aus JSON zuruecklesbar sein");
            assert_eq!(parsed, action);
        }
    }
}
