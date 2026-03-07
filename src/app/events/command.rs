//! Mutierende App-Commands fuer den zentralen Controller-Dispatch.

use super::super::state::EditorTool;
use crate::app::tools::common::TangentSource;
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
    /// Route-Tool: Strecke neu berechnen (Config geaendert)
    RouteToolRecreate,
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

    /// Route-Tool: Drag auf Steuerpunkt/Anker starten
    RouteToolDragStart { world_pos: glam::Vec2 },
    /// Route-Tool: Drag-Position aktualisieren
    RouteToolDragUpdate { world_pos: glam::Vec2 },
    /// Route-Tool: Drag beenden
    RouteToolDragEnd,
    /// Route-Tool: Scroll-Rotation anwenden
    RouteToolRotate { delta: f32 },
    /// Segment nachtraeglich bearbeiten
    EditSegment { record_id: u64 },
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
    /// Uebersichtskarten-ZIP-Dialog anfordern
    RequestOverviewDialog,
    /// Uebersichtskarten-Options-Dialog mit ZIP-Pfad oeffnen
    OpenOverviewOptionsDialog { path: String },
    /// Uebersichtskarte generieren (mit Layer-Optionen aus Dialog)
    GenerateOverviewWithOptions,
    /// Uebersichtskarten-Options-Dialog schliessen
    CloseOverviewOptionsDialog,
    /// Post-Load-Dialog schliessen
    DismissPostLoadDialog,
    /// Background-Map als overview.jpg im XML-Verzeichnis speichern
    SaveBackgroundAsOverview { path: String },
    /// overview.jpg-Speichern-Dialog schliessen
    DismissSaveOverviewDialog,
    /// Selektierte Nodes-Kette als gleichmaessig verteilte Wegpunkte neu berechnen (Distanzen)
    ResamplePath,
    /// Streckenteilung-Panel aktivieren
    StreckenteilungAktivieren,
    /// Alles in den Viewport einpassen (Zoom-to-fit)
    ZoomToFit,
    /// Auswahl invertieren
    InvertSelection,

    // ── Copy/Paste ────────────────────────────────────────────────────
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

    // ── Segment-Lock ──────────────────────────────────────────────────
    /// Segment-Lock umschalten (gesperrt ↔ entsperrt)
    ToggleSegmentLock { segment_id: u64 },
    /// Segment aufloesen (Segment-Record entfernen, Nodes beibehalten)
    DissolveSegment { segment_id: u64 },

    // ── Extras ───────────────────────────────────────────────────────
    /// Alle Farmland-Polygone als Wegpunkt-Ring nachzeichnen (Batch-Operation)
    TraceAllFields,
}
