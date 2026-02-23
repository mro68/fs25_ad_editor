//! AppIntent- und AppCommand-Enums für den Intent/Command-Datenfluss.

use super::state::EditorTool;
use crate::core::{ConnectionDirection, ConnectionPriority};
use crate::shared::EditorOptions;
use crate::shared::RenderQuality;

/// App-Intent und App-Command Events.
/// Intents sind Eingaben aus UI/System ohne direkte Mutationslogik.
#[derive(Debug, Clone)]
pub enum AppIntent {
    /// Datei öffnen (zeigt Dateidialog)
    OpenFileRequested,
    /// Datei speichern (unter aktuellem Pfad oder mit Dialog)
    SaveRequested,
    /// Datei unter neuem Pfad speichern
    SaveAsRequested,
    /// Anwendung beenden
    ExitRequested,
    /// Heightmap-Auswahldialog öffnen
    HeightmapSelectionRequested,
    /// Background-Map-Auswahldialog öffnen
    BackgroundMapSelectionRequested,
    /// Heightmap entfernen
    HeightmapCleared,
    /// Heightmap-Warnung bestätigt (Speichern fortsetzen)
    HeightmapWarningConfirmed,
    /// Heightmap-Warnung abgebrochen
    HeightmapWarningCancelled,
    /// Kamera auf Standard zurücksetzen
    ResetCameraRequested,
    /// Stufenweise hineinzoomen
    ZoomInRequested,
    /// Stufenweise herauszoomen
    ZoomOutRequested,
    /// Viewport-Größe hat sich geändert
    ViewportResized { size: [f32; 2] },
    /// Kamera um Delta verschieben (Welt-Einheiten)
    CameraPan { delta: glam::Vec2 },
    /// Kamera zoomen (optional auf einen Fokuspunkt)
    CameraZoom {
        factor: f32,
        focus_world: Option<glam::Vec2>,
    },
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

    /// Render-Qualitätsstufe ändern
    RenderQualityChanged { quality: RenderQuality },
    /// Datei wurde im Dialog ausgewählt (Laden)
    FileSelected { path: String },
    /// Speicherpfad wurde im Dialog ausgewählt
    SaveFilePathSelected { path: String },
    /// Heightmap-Datei wurde im Dialog ausgewählt
    HeightmapSelected { path: String },
    /// Background-Map auswählen
    BackgroundMapSelected {
        path: String,
        crop_size: Option<u32>,
    },
    /// Background-Opacity ändern
    SetBackgroundOpacity { opacity: f32 },
    /// Background-Sichtbarkeit umschalten
    ToggleBackgroundVisibility,
    /// Background-Ausdehnung skalieren (Faktor relativ, z.B. 2.0 = verdoppeln)
    ScaleBackground { factor: f32 },
    /// Undo: Letzte Aktion rückgängig machen
    UndoRequested,
    /// Redo: Rückgängig gemachte Aktion wiederherstellen
    RedoRequested,

    /// Editor-Werkzeug wechseln
    SetEditorToolRequested { tool: EditorTool },
    /// Neuen Node an Weltposition hinzufügen
    AddNodeRequested { world_pos: glam::Vec2 },
    /// Selektierte Nodes löschen
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
    /// Richtung einer Verbindung ändern
    SetConnectionDirectionRequested {
        start_id: u64,
        end_id: u64,
        direction: ConnectionDirection,
    },
    /// Priorität einer Verbindung ändern
    SetConnectionPriorityRequested {
        start_id: u64,
        end_id: u64,
        priority: ConnectionPriority,
    },
    /// Standard-Richtung für neue Verbindungen ändern
    SetDefaultDirectionRequested { direction: ConnectionDirection },
    /// Standard-Straßenart für neue Verbindungen ändern
    SetDefaultPriorityRequested { priority: ConnectionPriority },
    /// Richtung aller Verbindungen zwischen selektierten Nodes ändern
    SetAllConnectionsDirectionBetweenSelectedRequested { direction: ConnectionDirection },
    /// Alle Verbindungen zwischen selektierten Nodes trennen
    RemoveAllConnectionsBetweenSelectedRequested,
    /// Richtung aller Verbindungen zwischen selektierten Nodes invertieren (start↔end tauschen)
    InvertAllConnectionsBetweenSelectedRequested,
    /// Priorität aller Verbindungen zwischen selektierten Nodes ändern
    SetAllConnectionsPriorityBetweenSelectedRequested { priority: ConnectionPriority },
    /// Zwei selektierte Nodes verbinden (mit Standard-Richtung/Priorität)
    ConnectSelectedNodesRequested,
    /// Map-Marker für einen Node erstellen
    CreateMarkerRequested { node_id: u64 },
    /// Map-Marker für einen Node entfernen
    RemoveMarkerRequested { node_id: u64 },
    /// Map-Marker bearbeiten (Dialog öffnen)
    EditMarkerRequested { node_id: u64 },
    /// Marker-Dialog bestätigt (erstellen oder aktualisieren)
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
    /// Duplikat-Bereinigung bestätigt
    DeduplicateConfirmed,
    /// Duplikat-Bereinigung abgelehnt
    DeduplicateCancelled,
    /// Options-Dialog öffnen
    OpenOptionsDialogRequested,
    /// Options-Dialog schließen
    CloseOptionsDialogRequested,
    /// Optionen wurden geändert (sofortige Anwendung)
    OptionsChanged { options: EditorOptions },
    /// Optionen auf Standardwerte zurücksetzen
    ResetOptionsRequested,

    /// Route-Tool: Viewport-Klick
    RouteToolClicked { world_pos: glam::Vec2, ctrl: bool },
    /// Route-Tool: Ausführung bestätigt (Enter)
    RouteToolExecuteRequested,
    /// Route-Tool: Abbrechen (Escape)
    RouteToolCancelled,
    /// Route-Tool auswählen (per Index im ToolManager)
    SelectRouteToolRequested { index: usize },
    /// Route-Tool: Konfiguration geändert (Distanz/Anzahl) → Strecke neu berechnen
    RouteToolConfigChanged,

    /// Route-Tool: Drag auf Steuerpunkt/Anker gestartet
    RouteToolDragStarted { world_pos: glam::Vec2 },
    /// Route-Tool: Drag-Position aktualisiert
    RouteToolDragUpdated { world_pos: glam::Vec2 },
    /// Route-Tool: Drag beendet (Punkt loslassen)
    RouteToolDragEnded,
    /// Segment nachträglich bearbeiten (Nodes löschen + Tool laden)
    EditSegmentRequested { record_id: u64 },
    /// ZIP-Datei wurde als Background-Map gewählt → Browser öffnen
    ZipBackgroundBrowseRequested { path: String },
    /// Bilddatei aus ZIP-Browser gewählt
    ZipBackgroundFileSelected {
        zip_path: String,
        entry_name: String,
    },
    /// ZIP-Browser geschlossen (ohne Auswahl)
    ZipBrowserCancelled,
    /// Übersichtskarte aus Map-Mod-ZIP generieren (öffnet Dateidialog)
    GenerateOverviewRequested,
    /// Übersichtskarte aus gewähltem ZIP generieren
    GenerateOverviewFromZip { path: String },
}

/// Commands sind mutierende Schritte, die zentral ausgeführt werden.
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Editor-Werkzeug wechseln
    SetEditorTool { tool: EditorTool },
    /// Neuen Node an Weltposition hinzufügen
    AddNodeAtPosition { world_pos: glam::Vec2 },
    /// Selektierte Nodes löschen
    DeleteSelectedNodes,
    /// Connect-Tool: Node anwählen (Source oder Target)
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
    /// Richtung einer Verbindung ändern
    SetConnectionDirection {
        start_id: u64,
        end_id: u64,
        direction: ConnectionDirection,
    },
    /// Priorität einer Verbindung ändern
    SetConnectionPriority {
        start_id: u64,
        end_id: u64,
        priority: ConnectionPriority,
    },
    /// Standard-Richtung für neue Verbindungen setzen
    SetDefaultDirection { direction: ConnectionDirection },
    /// Standard-Priorität für neue Verbindungen setzen
    SetDefaultPriority { priority: ConnectionPriority },
    /// Bulk: Richtung aller Verbindungen zwischen Selektion ändern
    SetAllConnectionsDirectionBetweenSelected { direction: ConnectionDirection },
    /// Bulk: Alle Verbindungen zwischen Selektion entfernen
    RemoveAllConnectionsBetweenSelected,
    /// Bulk: Richtung aller Verbindungen zwischen Selektion invertieren
    InvertAllConnectionsBetweenSelected,
    /// Bulk: Priorität aller Verbindungen zwischen Selektion ändern
    SetAllConnectionsPriorityBetweenSelected { priority: ConnectionPriority },
    /// Zwei selektierte Nodes mit Standard-Einstellungen verbinden
    ConnectSelectedNodes,
    /// Datei-Öffnen-Dialog anfordern
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
    /// Speichern nach Heightmap-Warnung bestätigen
    ConfirmAndSaveFile,
    /// Kamera auf Standard zurücksetzen
    ResetCamera,
    /// Stufenweise hineinzoomen
    ZoomIn,
    /// Stufenweise herauszoomen
    ZoomOut,
    /// Viewport-Größe setzen
    SetViewportSize { size: [f32; 2] },
    /// Kamera um Delta verschieben
    PanCamera { delta: glam::Vec2 },
    /// Kamera zoomen (optional auf Fokuspunkt)
    ZoomCamera {
        factor: f32,
        focus_world: Option<glam::Vec2>,
    },
    /// Nächsten Node zur Position selektieren
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
    /// Render-Qualität setzen
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
    /// Background-Opacity ändern
    UpdateBackgroundOpacity { opacity: f32 },
    /// Background-Sichtbarkeit umschalten
    ToggleBackgroundVisibility,
    /// Background-Ausdehnung skalieren (Faktor relativ)
    ScaleBackground { factor: f32 },
    /// Heightmap-Warnung schließen
    DismissHeightmapWarning,
    /// Move-Lifecycle: Verschieben starten (Undo-Snapshot)
    BeginMoveSelectedNodes,
    /// Move-Lifecycle: Verschieben beenden
    EndMoveSelectedNodes,
    /// Undo: Letzte Aktion rückgängig machen
    Undo,
    /// Redo: Rückgängig gemachte Aktion wiederherstellen
    Redo,
    /// Map-Marker erstellen
    CreateMarker {
        node_id: u64,
        name: String,
        group: String,
    },
    /// Map-Marker entfernen
    RemoveMarker { node_id: u64 },
    /// Marker-Dialog öffnen (neu oder bearbeiten)
    OpenMarkerDialog { node_id: u64, is_new: bool },
    /// Marker aktualisieren
    UpdateMarker {
        node_id: u64,
        name: String,
        group: String,
    },
    /// Marker-Dialog schliessen
    CloseMarkerDialog,
    /// Duplikat-Bereinigung durchführen
    DeduplicateNodes,
    /// Duplikat-Dialog schließen (ohne Bereinigung)
    DismissDeduplicateDialog,
    /// Options-Dialog öffnen
    OpenOptionsDialog,
    /// Options-Dialog schliessen
    CloseOptionsDialog,
    /// Optionen anwenden und speichern
    ApplyOptions { options: EditorOptions },
    /// Optionen auf Standardwerte zurücksetzen
    ResetOptions,
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
    /// Route-Tool per Index aktivieren
    SelectRouteTool { index: usize },
    /// Route-Tool: Strecke neu berechnen (Config geändert)
    RouteToolRecreate,

    /// Route-Tool: Drag auf Steuerpunkt/Anker starten
    RouteToolDragStart { world_pos: glam::Vec2 },
    /// Route-Tool: Drag-Position aktualisieren
    RouteToolDragUpdate { world_pos: glam::Vec2 },
    /// Route-Tool: Drag beenden
    RouteToolDragEnd,
    /// Segment nachträglich bearbeiten
    EditSegment { record_id: u64 },
    /// ZIP-Archiv öffnen und Bilddateien im Browser anzeigen
    BrowseZipBackground { path: String },
    /// Bilddatei aus ZIP als Background-Map laden
    LoadBackgroundFromZip {
        zip_path: String,
        entry_name: String,
        crop_size: Option<u32>,
    },
    /// ZIP-Browser-Dialog schließen
    CloseZipBrowser,
    /// Übersichtskarten-ZIP-Dialog anfordern
    RequestOverviewDialog,
    /// Übersichtskarte aus Map-Mod-ZIP generieren und als Background laden
    GenerateOverviewFromZip { path: String },
}
