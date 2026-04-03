//! UI/System-Intents als nicht-mutierende Eingabeebene.

use super::super::state::EditorTool;
use super::AppEventFeature;
use crate::app::tool_contract::{RouteToolId, TangentSource};
use crate::app::ui_contract::RouteToolPanelAction;
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
use crate::shared::EditorOptions;
use crate::shared::RenderQuality;

/// App-Intent und App-Command Events.
/// Intents sind Eingaben aus UI/System ohne direkte Mutationslogik.
#[derive(Debug, Clone)]
pub enum AppIntent {
    /// Datei oeffnen (zeigt Dateidialog)
    OpenFileRequested,
    /// Datei speichern (unter aktuellem Pfad oder mit Dialog)
    SaveRequested,
    /// Datei unter neuem Pfad speichern
    SaveAsRequested,
    /// Anwendung beenden
    ExitRequested,
    /// Heightmap-Auswahldialog oeffnen
    HeightmapSelectionRequested,
    /// Background-Map-Auswahldialog oeffnen
    BackgroundMapSelectionRequested,
    /// Heightmap entfernen
    HeightmapCleared,
    /// Heightmap-Warnung bestaetigt (Speichern fortsetzen)
    HeightmapWarningConfirmed,
    /// Heightmap-Warnung abgebrochen
    HeightmapWarningCancelled,
    /// Kamera auf Standard zuruecksetzen
    ResetCameraRequested,
    /// Stufenweise hineinzoomen
    ZoomInRequested,
    /// Stufenweise herauszoomen
    ZoomOutRequested,
    /// Viewport-Groesse hat sich geaendert
    ViewportResized { size: [f32; 2] },
    /// Kamera um Delta verschieben (Welt-Einheiten)
    CameraPan { delta: glam::Vec2 },
    /// Kamera zoomen (optional auf einen Fokuspunkt)
    CameraZoom {
        factor: f32,
        focus_world: Option<glam::Vec2>,
    },
    /// Kamera auf einen bestimmten Node zentrieren (Zoom beibehalten)
    CenterOnNodeRequested { node_id: u64 },
    /// Node per Klick selektieren (Nearest-Node-Pick)
    NodePickRequested {
        world_pos: glam::Vec2,
        additive: bool,
        extend_path: bool,
    },
    /// Segment zwischen Kreuzungen per Doppelklick selektieren
    NodeSegmentBetweenIntersectionsRequested {
        world_pos: glam::Vec2,
        additive: bool,
    },
    /// Nodes innerhalb eines Rechtecks selektieren (Shift + Drag)
    SelectNodesInRectRequested {
        min: glam::Vec2,
        max: glam::Vec2,
        additive: bool,
    },
    /// Nodes innerhalb eines Lasso-Polygons selektieren (Alt + Drag)
    SelectNodesInLassoRequested {
        polygon: Vec<glam::Vec2>,
        additive: bool,
    },

    /// Move-Lifecycle Start: Drag-Verschieben selektierter Nodes beginnen
    BeginMoveSelectedNodesRequested,
    /// Move-Lifecycle Update: Selektierte Nodes um Delta verschieben
    MoveSelectedNodesRequested { delta_world: glam::Vec2 },
    /// Move-Lifecycle Ende: Drag-Verschieben abgeschlossen
    EndMoveSelectedNodesRequested,

    /// Rotation-Lifecycle Start: Undo-Snapshot aufnehmen
    BeginRotateSelectedNodesRequested,
    /// Rotation-Lifecycle Update: Selektierte Nodes um Delta-Winkel (Radiant) rotieren
    RotateSelectedNodesRequested { delta_angle: f32 },
    /// Rotation-Lifecycle Ende: Spatial-Index rebuild ausloesen
    EndRotateSelectedNodesRequested,

    /// Render-Qualitaetsstufe aendern
    RenderQualityChanged { quality: RenderQuality },
    /// Datei wurde im Dialog ausgewaehlt (Laden)
    FileSelected { path: String },
    /// Speicherpfad wurde im Dialog ausgewaehlt
    SaveFilePathSelected { path: String },
    /// Heightmap-Datei wurde im Dialog ausgewaehlt
    HeightmapSelected { path: String },
    /// Background-Map auswaehlen
    BackgroundMapSelected {
        path: String,
        crop_size: Option<u32>,
    },
    /// Background-Sichtbarkeit umschalten
    ToggleBackgroundVisibility,
    /// Background-Ausdehnung skalieren (Faktor relativ, z.B. 2.0 = verdoppeln)
    ScaleBackground { factor: f32 },
    /// Undo: Letzte Aktion rueckgaengig machen
    UndoRequested,
    /// Redo: Rueckgaengig gemachte Aktion wiederherstellen
    RedoRequested,

    /// Editor-Werkzeug wechseln
    SetEditorToolRequested { tool: EditorTool },
    /// Neuen Node an Weltposition hinzufuegen
    AddNodeRequested { world_pos: glam::Vec2 },
    /// Selektierte Nodes loeschen
    DeleteSelectedRequested,
    /// Connect-Tool: Node angeklickt (Source oder Target)
    ConnectToolNodeClicked { world_pos: glam::Vec2 },
    /// Verbindung zwischen zwei Nodes erstellen (via Shortcut/Panel)
    AddConnectionRequested {
        from_id: u64,
        to_id: u64,
        direction: ConnectionDirection,
        priority: ConnectionPriority,
    },
    /// Alle Verbindungen zwischen zwei Nodes entfernen
    RemoveConnectionBetweenRequested { node_a: u64, node_b: u64 },
    /// Richtung einer Verbindung aendern
    SetConnectionDirectionRequested {
        start_id: u64,
        end_id: u64,
        direction: ConnectionDirection,
    },
    /// Prioritaet einer Verbindung aendern
    SetConnectionPriorityRequested {
        start_id: u64,
        end_id: u64,
        priority: ConnectionPriority,
    },
    /// Node-Flag aendern (Regular, SubPrio, etc.)
    NodeFlagChangeRequested { node_id: u64, flag: NodeFlag },
    /// Standard-Richtung fuer neue Verbindungen aendern
    SetDefaultDirectionRequested { direction: ConnectionDirection },
    /// Standard-Strassenart fuer neue Verbindungen aendern
    SetDefaultPriorityRequested { priority: ConnectionPriority },
    /// Richtung aller Verbindungen zwischen selektierten Nodes aendern
    SetAllConnectionsDirectionBetweenSelectedRequested { direction: ConnectionDirection },
    /// Alle Verbindungen zwischen selektierten Nodes trennen
    RemoveAllConnectionsBetweenSelectedRequested,
    /// Richtung aller Verbindungen zwischen selektierten Nodes invertieren (start↔end tauschen)
    InvertAllConnectionsBetweenSelectedRequested,
    /// Prioritaet aller Verbindungen zwischen selektierten Nodes aendern
    SetAllConnectionsPriorityBetweenSelectedRequested { priority: ConnectionPriority },
    /// Zwei selektierte Nodes verbinden (mit Standard-Richtung/Prioritaet)
    ConnectSelectedNodesRequested,
    /// Map-Marker fuer einen Node erstellen
    CreateMarkerRequested { node_id: u64 },
    /// Map-Marker fuer einen Node entfernen
    RemoveMarkerRequested { node_id: u64 },
    /// Map-Marker bearbeiten (Dialog oeffnen)
    EditMarkerRequested { node_id: u64 },
    /// Marker-Dialog bestaetigt (erstellen oder aktualisieren)
    MarkerDialogConfirmed {
        node_id: u64,
        name: String,
        group: String,
        /// true = neu erstellen, false = bestehenden aktualisieren
        is_new: bool,
    },
    /// Marker-Dialog abgebrochen
    MarkerDialogCancelled,
    /// Selektion aufheben
    ClearSelectionRequested,
    /// Alle Nodes selektieren
    SelectAllRequested,
    /// Duplikat-Bereinigung bestaetigt
    DeduplicateConfirmed,
    /// Duplikat-Bereinigung abgelehnt
    DeduplicateCancelled,
    /// Options-Dialog oeffnen
    OpenOptionsDialogRequested,
    /// Options-Dialog schliessen
    CloseOptionsDialogRequested,
    /// Optionen wurden geaendert (sofortige Anwendung)
    OptionsChanged { options: Box<EditorOptions> },
    /// Optionen auf Standardwerte zuruecksetzen
    ResetOptionsRequested,
    /// Command-Palette oeffnen/schliessen
    CommandPaletteToggled,
    /// Schwebendes Menue an der Mausposition oeffnen/schliessen
    ToggleFloatingMenu {
        kind: crate::app::state::FloatingMenuKind,
    },

    /// Route-Tool: Viewport-Klick
    RouteToolClicked { world_pos: glam::Vec2, ctrl: bool },
    /// Route-Tool: Ausfuehrung bestaetigt (Enter)
    RouteToolExecuteRequested,
    /// Route-Tool: Abbrechen (Escape)
    RouteToolCancelled,
    /// Route-Tool auswaehlen (ueber stabile Tool-ID im Katalog)
    SelectRouteToolRequested { tool_id: RouteToolId },
    /// Route-Tool mit vordefinierten Start/End-Nodes aktivieren (Kontextmenue bei 2 selektierten Nodes)
    RouteToolWithAnchorsRequested {
        tool_id: RouteToolId,
        start_node_id: u64,
        end_node_id: u64,
    },
    /// Route-Tool: Konfiguration geaendert (Distanz/Anzahl) → Strecke neu berechnen
    RouteToolConfigChanged,
    /// Route-Tool: Semantische Panel-Aktion aus dem schwebenden Panel.
    RouteToolPanelActionRequested { action: RouteToolPanelAction },
    /// Route-Tool: Tangenten-Auswahl aus dem Kontextmenue aendern
    RouteToolTangentSelected {
        start: TangentSource,
        end: TangentSource,
    },

    /// Route-Tool: Lasso-Polygon abgeschlossen (Alt+Drag bei Tools die `needs_lasso_input()` setzen)
    RouteToolLassoCompleted { polygon: Vec<glam::Vec2> },
    /// Route-Tool: Drag auf Steuerpunkt/Anker gestartet
    RouteToolDragStarted { world_pos: glam::Vec2 },
    /// Route-Tool: Drag-Position aktualisiert
    RouteToolDragUpdated { world_pos: glam::Vec2 },
    /// Route-Tool: Drag beendet (Punkt loslassen)
    RouteToolDragEnded,
    /// Route-Tool: Alt+Scroll-Rotation
    RouteToolScrollRotated { delta: f32 },
    /// Segment nachtraeglich bearbeiten (Nodes loeschen + Tool laden)
    EditGroupRequested { record_id: u64 },
    /// Gruppen-Bearbeitung nicht-destruktiv starten (Select-Tool-Modus)
    GroupEditStartRequested { record_id: u64 },
    /// Gruppen-Bearbeitung abschliessen (Aenderungen uebernehmen)
    GroupEditApplyRequested,
    /// Gruppen-Bearbeitung abbrechen (Undo zum Snapshot vor Edit-Start)
    GroupEditCancelRequested,
    /// Aus Gruppen-Edit heraus das Tool-Edit starten (destruktiv/regenerativ)
    GroupEditToolRequested { record_id: u64 },
    /// ZIP-Datei wurde als Background-Map gewaehlt → Browser oeffnen
    ZipBackgroundBrowseRequested { path: String },
    /// Bilddatei aus ZIP-Browser gewaehlt
    ZipBackgroundFileSelected {
        zip_path: String,
        entry_name: String,
    },
    /// ZIP-Browser geschlossen (ohne Auswahl)
    ZipBrowserCancelled,
    /// Uebersichtskarte aus Map-Mod-ZIP generieren (oeffnet Dateidialog)
    GenerateOverviewRequested,
    /// ZIP fuer Uebersichtskarte gewaehlt → Options-Dialog anzeigen
    GenerateOverviewFromZip { path: String },
    /// Uebersichtskarten-Options-Dialog bestaetigt (generieren)
    OverviewOptionsConfirmed,
    /// Uebersichtskarten-Options-Dialog abgebrochen
    OverviewOptionsCancelled,
    /// Post-Load-Dialog: Uebersichtskarte generieren (ZIP-Pfad ausgewaehlt)
    PostLoadGenerateOverview { zip_path: String },
    /// Post-Load-Dialog: geschlossen ohne Aktion
    PostLoadDialogDismissed,
    /// Benutzer hat bestaetigt: Background als overview.png speichern
    SaveBackgroundAsOverviewConfirmed,
    /// Benutzer hat abgelehnt: overview.png nicht speichern
    SaveBackgroundAsOverviewDismissed,
    /// Selektierte Nodes-Kette als gleichmaessig verteilte Wegpunkte neu berechnen (Distanzen)
    ResamplePathRequested,
    /// Streckenteilung-Panel aktivieren (z.B. per Kontextmenue)
    StreckenteilungAktivieren,
    /// Alles in den Viewport einpassen (Zoom-to-fit)
    ZoomToFitRequested,
    /// Viewport auf die Grenzen der aktuellen Selektion einpassen
    ZoomToSelectionBoundsRequested,
    /// Auswahl invertieren (selektierte abwaehlen, nicht-selektierte waehlen)
    InvertSelectionRequested,
    /// Route-Tool: Strecke neu berechnen mit aktuellem Config (nach Parameter-Aenderung)
    RouteToolRecreateRequested,
    /// Route-Tool: Node-Anzahl erhoehen (Pfeiltaste oben)
    IncreaseRouteToolNodeCount,
    /// Route-Tool: Node-Anzahl verringern (Pfeiltaste unten)
    DecreaseRouteToolNodeCount,
    /// Route-Tool: Minimalabstand um 0.25m erhoehen (Pfeiltaste rechts)
    IncreaseRouteToolSegmentLength,
    /// Route-Tool: Minimalabstand um 0.25m verringern (Pfeiltaste links)
    DecreaseRouteToolSegmentLength,

    // ── Copy/Paste ────────────────────────────────────────────────────
    /// Selektion in die Zwischenablage kopieren
    CopySelectionRequested,
    /// Einfuegen-Vorschau starten (Clipboard → Vorschau auf Karte)
    PasteStartRequested,
    /// Einfuegen-Vorschau: Mauszeiger hat sich bewegt → Vorschau aktualisieren
    PastePreviewMoved { world_pos: glam::Vec2 },
    /// Einfuegen an aktueller Vorschauposition bestaetigen
    PasteConfirmRequested,
    /// Einfuegen-Vorschau abbrechen (Escape)
    PasteCancelled,

    // ── Segment-Lock ──────────────────────────────────────────────────
    /// Segment-Lock umschalten (gesperrt ↔ entsperrt)
    ToggleGroupLockRequested { segment_id: u64 },
    /// Segment aufloesen (Segment-Record entfernen, Nodes beibehalten)
    DissolveGroupRequested { segment_id: u64 },
    /// Bestaetigung: Gruppe aufloesen (nach Dialog-Bestaetigung)
    DissolveGroupConfirmed { segment_id: u64 },
    /// Selektierte zusammenhaengende Nodes als neues Segment in der Registry speichern
    GroupSelectionAsGroupRequested,
    /// Selektierte Nodes aus ihrer Gruppe entfernen (Nodes bleiben in RoadMap erhalten)
    RemoveSelectedNodesFromGroupRequested,
    /// Einfahrt/Ausfahrt-Nodes einer Gruppe setzen
    SetGroupBoundaryNodes {
        record_id: u64,
        entry_node_id: Option<u64>,
        exit_node_id: Option<u64>,
    },

    // ── Extras ───────────────────────────────────────────────────────
    /// Alle-Felder-nachzeichnen-Einstellungsdialog oeffnen
    OpenTraceAllFieldsDialogRequested,
    /// Alle-Felder-nachzeichnen bestaetigt (nach Dialog-Eingabe)
    TraceAllFieldsConfirmed {
        /// Abstand zwischen Wegpunkten in Metern
        spacing: f32,
        /// Randversatz in Metern (positiv = nach innen)
        offset: f32,
        /// Begradigung: Douglas-Peucker-Toleranz (0 = aus)
        tolerance: f32,
        /// Winkel-Schwellwert fuer Ecken-Erkennung in Grad (None = deaktiviert)
        corner_angle: Option<f32>,
        /// Verrundungsradius fuer Ecken in Metern (None = keine Verrundung)
        corner_rounding_radius: Option<f32>,
        /// Maximale Winkelabweichung zwischen Bogenpunkten in Grad (None = 15°)
        corner_rounding_max_angle_deg: Option<f32>,
    },
    /// Alle-Felder-nachzeichnen-Dialog abgebrochen
    TraceAllFieldsCancelled,
    /// Curseplay-Import-Dialog anfordern
    CurseplayImportRequested,
    /// Curseplay-Export-Dialog anfordern
    CurseplayExportRequested,
    /// Curseplay-Importdatei wurde im Dialog ausgewaehlt
    CurseplayFileSelected { path: String },
    /// Curseplay-Exportpfad wurde im Dialog ausgewaehlt
    CurseplayExportPathSelected { path: String },
}

impl AppIntent {
    /// Ordnet einen Intent einem internen Feature-Slice fuer Mapping und Tests zu.
    pub(crate) fn feature(&self) -> AppEventFeature {
        match self {
            Self::OpenFileRequested
            | Self::SaveRequested
            | Self::SaveAsRequested
            | Self::HeightmapSelectionRequested
            | Self::HeightmapCleared
            | Self::HeightmapWarningConfirmed
            | Self::HeightmapWarningCancelled
            | Self::FileSelected { .. }
            | Self::SaveFilePathSelected { .. }
            | Self::HeightmapSelected { .. }
            | Self::DeduplicateConfirmed
            | Self::DeduplicateCancelled => AppEventFeature::FileIo,
            Self::ResetCameraRequested
            | Self::ZoomInRequested
            | Self::ZoomOutRequested
            | Self::ViewportResized { .. }
            | Self::CameraPan { .. }
            | Self::CameraZoom { .. }
            | Self::CenterOnNodeRequested { .. }
            | Self::RenderQualityChanged { .. }
            | Self::BackgroundMapSelectionRequested
            | Self::BackgroundMapSelected { .. }
            | Self::ToggleBackgroundVisibility
            | Self::ScaleBackground { .. }
            | Self::ZipBackgroundBrowseRequested { .. }
            | Self::ZipBackgroundFileSelected { .. }
            | Self::ZipBrowserCancelled
            | Self::GenerateOverviewRequested
            | Self::GenerateOverviewFromZip { .. }
            | Self::OverviewOptionsConfirmed
            | Self::OverviewOptionsCancelled
            | Self::PostLoadGenerateOverview { .. }
            | Self::PostLoadDialogDismissed
            | Self::SaveBackgroundAsOverviewConfirmed
            | Self::SaveBackgroundAsOverviewDismissed
            | Self::ZoomToFitRequested
            | Self::ZoomToSelectionBoundsRequested => AppEventFeature::View,
            Self::NodePickRequested { .. }
            | Self::NodeSegmentBetweenIntersectionsRequested { .. }
            | Self::SelectNodesInRectRequested { .. }
            | Self::SelectNodesInLassoRequested { .. }
            | Self::BeginMoveSelectedNodesRequested
            | Self::MoveSelectedNodesRequested { .. }
            | Self::EndMoveSelectedNodesRequested
            | Self::BeginRotateSelectedNodesRequested
            | Self::RotateSelectedNodesRequested { .. }
            | Self::EndRotateSelectedNodesRequested
            | Self::ClearSelectionRequested
            | Self::SelectAllRequested
            | Self::InvertSelectionRequested => AppEventFeature::Selection,
            Self::SetEditorToolRequested { .. }
            | Self::AddNodeRequested { .. }
            | Self::DeleteSelectedRequested
            | Self::ConnectToolNodeClicked { .. }
            | Self::AddConnectionRequested { .. }
            | Self::RemoveConnectionBetweenRequested { .. }
            | Self::SetConnectionDirectionRequested { .. }
            | Self::SetConnectionPriorityRequested { .. }
            | Self::NodeFlagChangeRequested { .. }
            | Self::SetDefaultDirectionRequested { .. }
            | Self::SetDefaultPriorityRequested { .. }
            | Self::SetAllConnectionsDirectionBetweenSelectedRequested { .. }
            | Self::RemoveAllConnectionsBetweenSelectedRequested
            | Self::InvertAllConnectionsBetweenSelectedRequested
            | Self::SetAllConnectionsPriorityBetweenSelectedRequested { .. }
            | Self::ConnectSelectedNodesRequested
            | Self::CreateMarkerRequested { .. }
            | Self::RemoveMarkerRequested { .. }
            | Self::EditMarkerRequested { .. }
            | Self::MarkerDialogConfirmed { .. }
            | Self::MarkerDialogCancelled
            | Self::ResamplePathRequested
            | Self::StreckenteilungAktivieren
            | Self::CopySelectionRequested
            | Self::PasteStartRequested
            | Self::PastePreviewMoved { .. }
            | Self::PasteConfirmRequested
            | Self::PasteCancelled
            | Self::OpenTraceAllFieldsDialogRequested
            | Self::TraceAllFieldsConfirmed { .. }
            | Self::TraceAllFieldsCancelled
            | Self::CurseplayImportRequested
            | Self::CurseplayExportRequested
            | Self::CurseplayFileSelected { .. }
            | Self::CurseplayExportPathSelected { .. } => AppEventFeature::Editing,
            Self::RouteToolClicked { .. }
            | Self::RouteToolExecuteRequested
            | Self::RouteToolCancelled
            | Self::SelectRouteToolRequested { .. }
            | Self::RouteToolWithAnchorsRequested { .. }
            | Self::RouteToolConfigChanged
            | Self::RouteToolPanelActionRequested { .. }
            | Self::RouteToolTangentSelected { .. }
            | Self::RouteToolLassoCompleted { .. }
            | Self::RouteToolDragStarted { .. }
            | Self::RouteToolDragUpdated { .. }
            | Self::RouteToolDragEnded
            | Self::RouteToolScrollRotated { .. }
            | Self::RouteToolRecreateRequested
            | Self::IncreaseRouteToolNodeCount
            | Self::DecreaseRouteToolNodeCount
            | Self::IncreaseRouteToolSegmentLength
            | Self::DecreaseRouteToolSegmentLength => AppEventFeature::RouteTool,
            Self::EditGroupRequested { .. }
            | Self::GroupEditStartRequested { .. }
            | Self::GroupEditApplyRequested
            | Self::GroupEditCancelRequested
            | Self::GroupEditToolRequested { .. }
            | Self::GroupSelectionAsGroupRequested
            | Self::RemoveSelectedNodesFromGroupRequested
            | Self::SetGroupBoundaryNodes { .. }
            | Self::ToggleGroupLockRequested { .. }
            | Self::DissolveGroupRequested { .. }
            | Self::DissolveGroupConfirmed { .. } => AppEventFeature::Group,
            Self::ExitRequested
            | Self::OpenOptionsDialogRequested
            | Self::CloseOptionsDialogRequested
            | Self::OptionsChanged { .. }
            | Self::ResetOptionsRequested
            | Self::CommandPaletteToggled
            | Self::ToggleFloatingMenu { .. } => AppEventFeature::Dialog,
            Self::UndoRequested | Self::RedoRequested => AppEventFeature::History,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppIntent;
    use crate::app::events::AppEventFeature;
    use crate::app::ui_contract::{ParkingPanelAction, RouteToolPanelAction};

    #[test]
    fn classifies_editing_group_and_dialog_intents() {
        assert_eq!(
            AppIntent::PasteCancelled.feature(),
            AppEventFeature::Editing
        );
        assert_eq!(
            AppIntent::GroupEditToolRequested { record_id: 7 }.feature(),
            AppEventFeature::Group
        );
        assert_eq!(
            AppIntent::CommandPaletteToggled.feature(),
            AppEventFeature::Dialog
        );
    }

    #[test]
    fn classifies_view_and_route_tool_intents() {
        assert_eq!(
            AppIntent::GenerateOverviewRequested.feature(),
            AppEventFeature::View
        );
        assert_eq!(
            AppIntent::RouteToolPanelActionRequested {
                action: RouteToolPanelAction::Parking(ParkingPanelAction::SetNumRows(3)),
            }
            .feature(),
            AppEventFeature::RouteTool
        );
    }
}
