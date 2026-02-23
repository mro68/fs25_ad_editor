use super::super::state::EditorTool;
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
    /// ZIP für Übersichtskarte gewählt → Options-Dialog anzeigen
    GenerateOverviewFromZip { path: String },
    /// Übersichtskarten-Options-Dialog bestätigt (generieren)
    OverviewOptionsConfirmed,
    /// Übersichtskarten-Options-Dialog abgebrochen
    OverviewOptionsCancelled,
    /// Post-Load-Dialog: Übersichtskarte generieren (ZIP-Pfad ausgewählt)
    PostLoadGenerateOverview { zip_path: String },
    /// Post-Load-Dialog: geschlossen ohne Aktion
    PostLoadDialogDismissed,
}
