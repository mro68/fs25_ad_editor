use super::super::state::EditorTool;
use crate::app::tools::common::TangentSource;
use crate::core::{ConnectionDirection, ConnectionPriority};
use crate::shared::EditorOptions;
use crate::shared::RenderQuality;

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
    /// Route-Tool mit vordefinierten Start/End-Nodes aktivieren und Klicks simulieren
    RouteToolWithAnchors {
        index: usize,
        start_node_id: u64,
        end_node_id: u64,
    },
    /// Route-Tool: Strecke neu berechnen (Config geändert)
    RouteToolRecreate,
    /// Route-Tool: Node-Anzahl erhöhen
    IncreaseRouteToolNodeCount,
    /// Route-Tool: Node-Anzahl verringern
    DecreaseRouteToolNodeCount,
    /// Route-Tool: Minimalabstand um 0.25m erhöhen
    IncreaseRouteToolSegmentLength,
    /// Route-Tool: Minimalabstand um 0.25m verringern
    DecreaseRouteToolSegmentLength,
    /// Route-Tool: Tangenten-Auswahl anwenden und ggf. Recreate triggern
    RouteToolApplyTangent {
        start: TangentSource,
        end: TangentSource,
    },

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
    /// Übersichtskarten-Options-Dialog mit ZIP-Pfad öffnen
    OpenOverviewOptionsDialog { path: String },
    /// Übersichtskarte generieren (mit Layer-Optionen aus Dialog)
    GenerateOverviewWithOptions,
    /// Übersichtskarten-Options-Dialog schließen
    CloseOverviewOptionsDialog,
    /// Post-Load-Dialog schließen
    DismissPostLoadDialog,
    /// Background-Map als overview.jpg im XML-Verzeichnis speichern
    SaveBackgroundAsOverview { path: String },
    /// overview.jpg-Speichern-Dialog schließen
    DismissSaveOverviewDialog,
    /// Selektierte Nodes-Kette als gleichmäßig verteilte Wegpunkte neu berechnen (Distanzen)
    ResamplePath,
    /// Streckenteilung-Panel aktivieren
    StreckenteilungAktivieren,
    /// Alles in den Viewport einpassen (Zoom-to-fit)
    ZoomToFit,
    /// Selektierte Nodes duplizieren (mit Versatz)
    DuplicateSelectedNodes,
    /// Auswahl invertieren
    InvertSelection,
}
