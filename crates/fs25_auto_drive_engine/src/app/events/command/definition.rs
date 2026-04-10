use crate::app::events::AppEventFeature;
use crate::app::state::EditorTool;
use crate::app::tool_contract::{RouteToolId, TangentSource};
use crate::app::ui_contract::RouteToolPanelAction;
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
use crate::shared::EditorOptions;
use crate::shared::RenderQuality;

/// Commands sind mutierende Schritte, die zentral ausgefuehrt werden.
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Editor-Werkzeug wechseln
    SetEditorTool { tool: EditorTool },
    /// Neuen Node an Weltposition hinzufuegen
    AddNodeAtPosition { world_pos: glam::Vec2 },
    /// Selektierte Nodes loeschen
    DeleteSelectedNodes,
    /// Connect-Tool: Node anwaehlen (Source oder Target)
    ConnectToolPickNode {
        world_pos: glam::Vec2,
        max_distance: f32,
    },
    /// Verbindung zwischen zwei Nodes erstellen
    AddConnection {
        from_id: u64,
        to_id: u64,
        direction: ConnectionDirection,
        priority: ConnectionPriority,
    },
    /// Alle Verbindungen zwischen zwei Nodes entfernen
    RemoveConnectionBetween { node_a: u64, node_b: u64 },
    /// Richtung einer Verbindung aendern
    SetConnectionDirection {
        start_id: u64,
        end_id: u64,
        direction: ConnectionDirection,
    },
    /// Prioritaet einer Verbindung aendern
    SetConnectionPriority {
        start_id: u64,
        end_id: u64,
        priority: ConnectionPriority,
    },
    /// Setzt das Flag eines Nodes
    SetNodeFlag { node_id: u64, flag: NodeFlag },
    /// Standard-Richtung fuer neue Verbindungen setzen
    SetDefaultDirection { direction: ConnectionDirection },
    /// Standard-Prioritaet fuer neue Verbindungen setzen
    SetDefaultPriority { priority: ConnectionPriority },
    /// Bulk: Richtung aller Verbindungen zwischen Selektion aendern
    SetAllConnectionsDirectionBetweenSelected { direction: ConnectionDirection },
    /// Bulk: Alle Verbindungen zwischen Selektion entfernen
    RemoveAllConnectionsBetweenSelected,
    /// Bulk: Richtung aller Verbindungen zwischen Selektion invertieren
    InvertAllConnectionsBetweenSelected,
    /// Bulk: Prioritaet aller Verbindungen zwischen Selektion aendern
    SetAllConnectionsPriorityBetweenSelected { priority: ConnectionPriority },
    /// Zwei selektierte Nodes mit Standard-Einstellungen verbinden
    ConnectSelectedNodes,
    /// Datei-Oeffnen-Dialog anfordern
    RequestOpenFileDialog,
    /// Datei-Speichern-Dialog anfordern
    RequestSaveFileDialog,
    /// Anwendung beenden
    RequestExit,
    /// Heightmap-Dialog anfordern
    RequestHeightmapDialog,
    /// Background-Map-Dialog anfordern
    RequestBackgroundMapDialog,
    /// Heightmap entfernen
    ClearHeightmap,
    /// Speichern nach Heightmap-Warnung bestaetigen
    ConfirmAndSaveFile,
    /// Kamera auf Standard zuruecksetzen
    ResetCamera,
    /// Stufenweise hineinzoomen
    ZoomIn,
    /// Stufenweise herauszoomen
    ZoomOut,
    /// Viewport-Groesse setzen
    SetViewportSize { size: [f32; 2] },
    /// Kamera um Delta verschieben
    PanCamera { delta: glam::Vec2 },
    /// Kamera zoomen (optional auf Fokuspunkt)
    ZoomCamera {
        factor: f32,
        focus_world: Option<glam::Vec2>,
    },
    /// Kamera auf Node zentrieren (Zoom beibehalten)
    CenterOnNode { node_id: u64 },
    /// Naechsten Node zur Position selektieren
    SelectNearestNode {
        world_pos: glam::Vec2,
        max_distance: f32,
        additive: bool,
        extend_path: bool,
    },
    /// Segment zwischen Kreuzungen selektieren
    SelectSegmentBetweenNearestIntersections {
        world_pos: glam::Vec2,
        max_distance: f32,
        additive: bool,
        stop_at_junction: bool,
        max_angle_deg: f32,
    },
    /// Alle Nodes einer Gruppe selektieren (identifiziert ueber Naehe zu world_pos)
    SelectGroupByNearestNode {
        world_pos: glam::Vec2,
        max_distance: f32,
        additive: bool,
    },
    /// Nodes innerhalb eines Rechtecks selektieren
    SelectNodesInRect {
        min: glam::Vec2,
        max: glam::Vec2,
        additive: bool,
    },
    /// Nodes innerhalb eines Lasso-Polygons selektieren
    SelectNodesInLasso {
        polygon: Vec<glam::Vec2>,
        additive: bool,
    },
    /// Selektierte Nodes um Delta verschieben
    MoveSelectedNodes { delta_world: glam::Vec2 },
    /// Rotation-Lifecycle: Starten (Undo-Snapshot aufnehmen)
    BeginRotateSelectedNodes,
    /// Rotation-Lifecycle: Selektierte Nodes um Delta-Winkel (Radiant) rotieren
    RotateSelectedNodes { delta_angle: f32 },
    /// Rotation-Lifecycle: Beenden (Spatial-Index rebuild anstoßen)
    EndRotateSelectedNodes,
    /// Render-Qualitaet setzen
    SetRenderQuality { quality: RenderQuality },
    /// XML-Datei laden
    LoadFile { path: String },
    /// Datei speichern (None = aktueller Pfad, Some(p) = neuer Pfad)
    SaveFile { path: Option<String> },
    /// Heightmap setzen
    SetHeightmap { path: String },
    /// Background-Map laden
    LoadBackgroundMap {
        path: String,
        crop_size: Option<u32>,
    },
    /// Background-Sichtbarkeit umschalten
    ToggleBackgroundVisibility,
    /// Background-Ausdehnung skalieren (Faktor relativ)
    ScaleBackground { factor: f32 },
    /// Heightmap-Warnung schliessen
    DismissHeightmapWarning,
    /// Move-Lifecycle: Verschieben starten (Undo-Snapshot)
    BeginMoveSelectedNodes,
    /// Move-Lifecycle: Verschieben beenden
    EndMoveSelectedNodes,
    /// Undo: Letzte Aktion rueckgaengig machen
    Undo,
    /// Redo: Rueckgaengig gemachte Aktion wiederherstellen
    Redo,
    /// Map-Marker erstellen
    CreateMarker {
        node_id: u64,
        name: String,
        group: String,
    },
    /// Map-Marker entfernen
    RemoveMarker { node_id: u64 },
    /// Marker-Dialog oeffnen (neu oder bearbeiten)
    OpenMarkerDialog { node_id: u64, is_new: bool },
    /// Marker aktualisieren
    UpdateMarker {
        node_id: u64,
        name: String,
        group: String,
    },
    /// Marker-Dialog schliessen
    CloseMarkerDialog,
    /// Duplikat-Bereinigung durchfuehren
    DeduplicateNodes,
    /// Duplikat-Dialog schliessen (ohne Bereinigung)
    DismissDeduplicateDialog,
    /// Options-Dialog oeffnen
    OpenOptionsDialog,
    /// Options-Dialog schliessen
    CloseOptionsDialog,
    /// Optionen anwenden und speichern
    ApplyOptions { options: Box<EditorOptions> },
    /// Optionen auf Standardwerte zuruecksetzen
    ResetOptions,
    /// Command-Palette ein-/ausblenden
    ToggleCommandPalette,
    /// Selektion aufheben
    ClearSelection,
    /// Alle Nodes selektieren
    SelectAllNodes,
    /// Route-Tool: Viewport-Klick verarbeiten
    RouteToolClick { world_pos: glam::Vec2, ctrl: bool },
    /// Route-Tool: Ergebnis anwenden
    RouteToolExecute,
    /// Route-Tool: Abbrechen
    RouteToolCancel,
    /// Route-Tool per stabiler Tool-ID aktivieren.
    SelectRouteTool { tool_id: RouteToolId },
    /// Route-Tool mit vordefinierten Start/End-Nodes aktivieren und Klicks simulieren
    RouteToolWithAnchors {
        tool_id: RouteToolId,
        start_node_id: u64,
        end_node_id: u64,
    },
    /// Route-Tool: Strecke neu berechnen (Config geaendert)
    RouteToolRecreate,
    /// Route-Tool: Semantische Panel-Aktion anwenden.
    RouteToolPanelAction { action: RouteToolPanelAction },
    /// Route-Tool: Node-Anzahl erhoehen
    IncreaseRouteToolNodeCount,
    /// Route-Tool: Node-Anzahl verringern
    DecreaseRouteToolNodeCount,
    /// Route-Tool: Minimalabstand um 0.25m erhoehen
    IncreaseRouteToolSegmentLength,
    /// Route-Tool: Minimalabstand um 0.25m verringern
    DecreaseRouteToolSegmentLength,
    /// Route-Tool: Tangenten-Auswahl anwenden und ggf. Recreate triggern
    RouteToolApplyTangent {
        start: TangentSource,
        end: TangentSource,
    },
    /// Route-Tool: Lasso-Polygon an das aktive Route-Tool weiterleiten
    RouteToolLassoCompleted { polygon: Vec<glam::Vec2> },
    /// Route-Tool: Drag auf Steuerpunkt/Anker starten
    RouteToolDragStart { world_pos: glam::Vec2 },
    /// Route-Tool: Drag-Position aktualisieren
    RouteToolDragUpdate { world_pos: glam::Vec2 },
    /// Route-Tool: Drag beenden
    RouteToolDragEnd,
    /// Route-Tool: Scroll-Rotation anwenden
    RouteToolRotate { delta: f32 },
    /// Segment nachtraeglich bearbeiten
    EditGroup { record_id: u64 },
    /// Gruppen-Edit-Modus nicht-destruktiv starten
    GroupEditStart { record_id: u64 },
    /// Gruppen-Edit uebernehmen (Aenderungen persistieren)
    GroupEditApply,
    /// Gruppen-Edit abbrechen (Undo zum Snapshot)
    GroupEditCancel,
    /// Atomar: Gruppen-Edit aufraumen → Undo → Tool-Edit starten
    BeginToolEditFromGroup { record_id: u64 },
    /// ZIP-Archiv oeffnen und Bilddateien im Browser anzeigen
    BrowseZipBackground { path: String },
    /// Bilddatei aus ZIP als Background-Map laden
    LoadBackgroundFromZip {
        zip_path: String,
        entry_name: String,
        crop_size: Option<u32>,
    },
    /// ZIP-Browser-Dialog schliessen
    CloseZipBrowser,
    /// Wiederverwendbaren Overview-Source-Dialog oeffnen
    OpenOverviewSourceDialog,
    /// Nativen Uebersichtskarten-ZIP-Dialog anfordern
    RequestOverviewDialog,
    /// Uebersichtskarten-Options-Dialog mit ZIP-Pfad oeffnen
    OpenOverviewOptionsDialog { path: String },
    /// Uebersichtskarte generieren (mit Layer-Optionen aus Dialog)
    GenerateOverviewWithOptions,
    /// Uebersichtskarten-Options-Dialog schliessen
    CloseOverviewOptionsDialog,
    /// Post-Load-Dialog schliessen
    DismissPostLoadDialog,
    /// Background-Map als overview.png im XML-Verzeichnis speichern
    SaveBackgroundAsOverview { path: String },
    /// overview.png-Speichern-Dialog schliessen
    DismissSaveOverviewDialog,
    /// Selektierte Nodes-Kette als gleichmaessig verteilte Wegpunkte neu berechnen (Distanzen)
    ResamplePath,
    /// Streckenteilung-Panel aktivieren
    StreckenteilungAktivieren,
    /// Alles in den Viewport einpassen (Zoom-to-fit)
    ZoomToFit,
    /// Kamera auf die Bounding Box der Selektion zoomen
    ZoomToSelectionBounds,
    /// Auswahl invertieren
    InvertSelection,
    /// Selektion in die Zwischenablage kopieren
    CopySelection,
    /// Einfuegen-Vorschau starten
    StartPastePreview,
    /// Einfuegen-Vorschau: Position aktualisieren
    UpdatePastePreview { world_pos: glam::Vec2 },
    /// Einfuegen an aktueller Vorschauposition bestaetigen
    ConfirmPaste,
    /// Einfuegen-Vorschau abbrechen
    CancelPastePreview,
    /// Segment-Lock umschalten (gesperrt ↔ entsperrt)
    ToggleGroupLock { segment_id: u64 },
    /// Segment aufloesen (Segment-Record entfernen, Nodes beibehalten)
    DissolveGroup { segment_id: u64 },
    /// Dialog zum Bestaetigen des Aufloesens oeffnen
    OpenDissolveConfirmDialog { segment_id: u64 },
    /// Selektierte zusammenhaengende Nodes als neues Segment in der Registry speichern
    GroupSelectionAsGroup,
    /// Selektierte Nodes aus ihren zugehoerigen Gruppen entfernen
    RemoveSelectedNodesFromGroups,
    /// Einfahrt/Ausfahrt-Nodes einer Gruppe setzen
    SetGroupBoundaryNodes {
        record_id: u64,
        entry_node_id: Option<u64>,
        exit_node_id: Option<u64>,
    },
    /// Einstellungsdialog "Alle Felder nachzeichnen" oeffnen
    OpenTraceAllFieldsDialog,
    /// Einstellungsdialog "Alle Felder nachzeichnen" schliessen (Abbruch)
    CloseTraceAllFieldsDialog,
    /// Alle Farmland-Polygone als Wegpunkt-Ring nachzeichnen (Batch-Operation)
    TraceAllFields {
        spacing: f32,
        offset: f32,
        tolerance: f32,
        corner_angle: Option<f32>,
        corner_rounding_radius: Option<f32>,
        corner_rounding_max_angle_deg: Option<f32>,
    },
    /// Curseplay-Import-Dateidialog anfordern
    RequestCurseplayImportDialog,
    /// Curseplay-Datei importieren (Nodes + Ring-Verbindungen anlegen)
    ImportCurseplay { path: String },
    /// Curseplay-Export-Dateidialog anfordern
    RequestCurseplayExportDialog,
    /// Selektierte Strecke als Curseplay-XML exportieren
    ExportCurseplay { path: String },
    /// Segment-Einstellungs-Popup oeffnen oder aktualisieren
    OpenGroupSettingsPopup { world_pos: glam::Vec2 },
}

impl AppCommand {
    /// Ordnet einen Command einem internen Feature-Slice fuer Controller-Dispatch und Tests zu.
    pub(crate) fn feature(&self) -> AppEventFeature {
        match self {
            Self::RequestOpenFileDialog
            | Self::RequestSaveFileDialog
            | Self::ConfirmAndSaveFile
            | Self::LoadFile { .. }
            | Self::SaveFile { .. }
            | Self::ClearHeightmap
            | Self::SetHeightmap { .. }
            | Self::DeduplicateNodes => AppEventFeature::FileIo,
            Self::ResetCamera
            | Self::ZoomIn
            | Self::ZoomOut
            | Self::SetViewportSize { .. }
            | Self::PanCamera { .. }
            | Self::ZoomCamera { .. }
            | Self::CenterOnNode { .. }
            | Self::SetRenderQuality { .. }
            | Self::LoadBackgroundMap { .. }
            | Self::ToggleBackgroundVisibility
            | Self::ScaleBackground { .. }
            | Self::BrowseZipBackground { .. }
            | Self::LoadBackgroundFromZip { .. }
            | Self::GenerateOverviewWithOptions
            | Self::SaveBackgroundAsOverview { .. }
            | Self::ZoomToFit
            | Self::ZoomToSelectionBounds => AppEventFeature::View,
            Self::SelectNearestNode { .. }
            | Self::SelectSegmentBetweenNearestIntersections { .. }
            | Self::SelectGroupByNearestNode { .. }
            | Self::SelectNodesInRect { .. }
            | Self::SelectNodesInLasso { .. }
            | Self::MoveSelectedNodes { .. }
            | Self::BeginMoveSelectedNodes
            | Self::EndMoveSelectedNodes
            | Self::BeginRotateSelectedNodes
            | Self::RotateSelectedNodes { .. }
            | Self::EndRotateSelectedNodes
            | Self::ClearSelection
            | Self::SelectAllNodes
            | Self::InvertSelection => AppEventFeature::Selection,
            Self::SetEditorTool { .. }
            | Self::AddNodeAtPosition { .. }
            | Self::DeleteSelectedNodes
            | Self::ConnectToolPickNode { .. }
            | Self::AddConnection { .. }
            | Self::RemoveConnectionBetween { .. }
            | Self::SetConnectionDirection { .. }
            | Self::SetConnectionPriority { .. }
            | Self::SetNodeFlag { .. }
            | Self::SetDefaultDirection { .. }
            | Self::SetDefaultPriority { .. }
            | Self::SetAllConnectionsDirectionBetweenSelected { .. }
            | Self::RemoveAllConnectionsBetweenSelected
            | Self::InvertAllConnectionsBetweenSelected
            | Self::SetAllConnectionsPriorityBetweenSelected { .. }
            | Self::ConnectSelectedNodes
            | Self::CreateMarker { .. }
            | Self::RemoveMarker { .. }
            | Self::OpenMarkerDialog { .. }
            | Self::UpdateMarker { .. }
            | Self::ResamplePath
            | Self::StreckenteilungAktivieren
            | Self::CopySelection
            | Self::StartPastePreview
            | Self::UpdatePastePreview { .. }
            | Self::ConfirmPaste
            | Self::CancelPastePreview
            | Self::TraceAllFields { .. }
            | Self::ImportCurseplay { .. }
            | Self::ExportCurseplay { .. } => AppEventFeature::Editing,
            Self::RouteToolClick { .. }
            | Self::RouteToolExecute
            | Self::RouteToolCancel
            | Self::SelectRouteTool { .. }
            | Self::RouteToolWithAnchors { .. }
            | Self::RouteToolRecreate
            | Self::RouteToolPanelAction { .. }
            | Self::IncreaseRouteToolNodeCount
            | Self::DecreaseRouteToolNodeCount
            | Self::IncreaseRouteToolSegmentLength
            | Self::DecreaseRouteToolSegmentLength
            | Self::RouteToolApplyTangent { .. }
            | Self::RouteToolLassoCompleted { .. }
            | Self::RouteToolDragStart { .. }
            | Self::RouteToolDragUpdate { .. }
            | Self::RouteToolDragEnd
            | Self::RouteToolRotate { .. } => AppEventFeature::RouteTool,
            Self::EditGroup { .. }
            | Self::ToggleGroupLock { .. }
            | Self::DissolveGroup { .. }
            | Self::OpenDissolveConfirmDialog { .. }
            | Self::GroupSelectionAsGroup
            | Self::RemoveSelectedNodesFromGroups
            | Self::SetGroupBoundaryNodes { .. }
            | Self::GroupEditStart { .. }
            | Self::GroupEditApply
            | Self::GroupEditCancel
            | Self::BeginToolEditFromGroup { .. }
            | Self::OpenGroupSettingsPopup { .. } => AppEventFeature::Group,
            Self::RequestExit
            | Self::RequestHeightmapDialog
            | Self::RequestBackgroundMapDialog
            | Self::DismissHeightmapWarning
            | Self::CloseMarkerDialog
            | Self::OpenOptionsDialog
            | Self::CloseOptionsDialog
            | Self::ApplyOptions { .. }
            | Self::ResetOptions
            | Self::ToggleCommandPalette
            | Self::DismissDeduplicateDialog
            | Self::CloseZipBrowser
            | Self::OpenOverviewSourceDialog
            | Self::RequestOverviewDialog
            | Self::OpenOverviewOptionsDialog { .. }
            | Self::CloseOverviewOptionsDialog
            | Self::DismissPostLoadDialog
            | Self::DismissSaveOverviewDialog
            | Self::OpenTraceAllFieldsDialog
            | Self::CloseTraceAllFieldsDialog
            | Self::RequestCurseplayImportDialog
            | Self::RequestCurseplayExportDialog => AppEventFeature::Dialog,
            Self::Undo | Self::Redo => AppEventFeature::History,
        }
    }
}
