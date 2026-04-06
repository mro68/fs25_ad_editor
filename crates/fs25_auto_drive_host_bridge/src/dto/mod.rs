use fs25_auto_drive_engine::shared::EditorOptions;
use serde::{Deserialize, Serialize};

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

/// Stabile Pointer-Button-Klassifikation fuer den Viewport-Input-Vertrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostPointerButton {
    /// Primaere Pointer-Taste.
    Primary,
    /// Mittlere Pointer-Taste.
    Middle,
    /// Sekundaere Pointer-Taste.
    Secondary,
}

/// Stabile Tap-Klassifikation fuer den Viewport-Input-Vertrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTapKind {
    /// Einfacher Tap bzw. einzelner Klick.
    Single,
    /// Doppelter Tap bzw. Doppelklick.
    Double,
}

/// Host-neutrale Modifiers fuer Viewport-Input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInputModifiers {
    /// Shift-Modifizierer.
    pub shift: bool,
    /// Alt-/Option-Modifizierer.
    pub alt: bool,
    /// Plattformneutraler Command-Modifizierer (`Ctrl` bzw. `Cmd`).
    pub command: bool,
}

/// Batch von host-neutralen Viewport-Input-Events fuer die kanonische Session-Surface.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HostViewportInputBatch {
    /// In Reihenfolge empfangene Viewport-Input-Events.
    pub events: Vec<HostViewportInputEvent>,
}

/// Kleines host-neutrales Viewport-Input-Event fuer die Bridge-Surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostViewportInputEvent {
    /// Aktualisiert die bekannte Viewport-Groesse der Session.
    Resize {
        /// Neue Viewport-Groesse in Pixeln [width, height].
        size_px: [f32; 2],
    },
    /// Einzelner Tap bzw. Klick an Bildschirmposition.
    Tap {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Art des Taps.
        tap_kind: HostTapKind,
        /// Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: [f32; 2],
        /// Aktive Modifiers zum Zeitpunkt des Taps.
        modifiers: HostInputModifiers,
    },
    /// Start eines Drags an Bildschirmposition.
    DragStart {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: [f32; 2],
        /// Aktive Modifiers zum Zeitpunkt des Starts.
        modifiers: HostInputModifiers,
    },
    /// Delta-Update eines laufenden Drags.
    DragUpdate {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Aktuelle Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: [f32; 2],
        /// Delta in Bildschirm-Pixeln seit dem letzten Update.
        delta_px: [f32; 2],
    },
    /// Ende eines laufenden Drags.
    DragEnd {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Optionale finale Bildschirmposition relativ zum Viewport.
        screen_pos: Option<[f32; 2]>,
    },
    /// Scroll-Ereignis an optionaler Bildschirmposition.
    Scroll {
        /// Optionale Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: Option<[f32; 2]>,
        /// Geglaettete Scroll-Differenz fuer Zoom-Interpretation.
        smooth_delta_y: f32,
        /// Rohes Scroll-Delta fuer spaetere Tick-basierte Erweiterungen.
        raw_delta_y: f32,
        /// Aktive Modifiers zum Zeitpunkt des Scrollens.
        modifiers: HostInputModifiers,
    },
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
    /// Fordert einen Background-Map-Auswahldialog an.
    RequestBackgroundMapSelection,
    /// Fordert den ZIP-Auswahldialog fuer die Overview-Generierung an.
    GenerateOverview,
    /// Fordert einen Curseplay-Import-Dialog an.
    CurseplayImport,
    /// Fordert einen Curseplay-Export-Dialog an.
    CurseplayExport,
    /// Setzt die Kamera auf den Standardzustand zurueck.
    ResetCamera,
    /// Passt den Viewport auf die komplette Karte ein.
    ZoomToFit,
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
    /// Oeffnet den Optionen-Dialog.
    OpenOptionsDialog,
    /// Schliesst den Optionen-Dialog.
    CloseOptionsDialog,
    /// Fuehrt den letzten Undo-faehigen Schritt rueckgaengig aus.
    Undo,
    /// Stellt den letzten Undo-Schritt wieder her.
    Redo,
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

/// Serialisierbarer Snapshot der aktuellen Auswahl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSelectionSnapshot {
    /// Aktuell selektierte Node-IDs in stabiler Reihenfolge.
    pub selected_node_ids: Vec<u64>,
}

/// Serialisierbarer Snapshot des aktuellen Viewports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportSnapshot {
    /// Kameraposition in Weltkoordinaten.
    pub camera_position: [f32; 2],
    /// Zoom-Faktor des aktuellen Frames.
    pub zoom: f32,
}

/// Stabile Render-Klassifikation eines Nodes fuer host-neutrale Geometry-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostViewportNodeKind {
    /// Standard-Node ohne besondere Warn- oder Subprio-Faerbung.
    Regular,
    /// Subpriorisierter Node.
    SubPrio,
    /// Warn-Node.
    Warning,
}

/// Stabile Richtungsklassifikation einer Verbindung fuer Geometry-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostViewportConnectionDirection {
    /// Pfeil in Start-zu-Ende-Richtung.
    Regular,
    /// Bidirektionale Verbindung ohne Pfeil.
    Dual,
    /// Pfeil entgegengesetzt zur Start-zu-Ende-Geometrie.
    Reverse,
}

/// Stabile Prioritaetsklassifikation einer Verbindung fuer Geometry-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostViewportConnectionPriority {
    /// Normale Verbindung.
    Regular,
    /// Subpriorisierte Verbindung.
    SubPriority,
}

/// Host-neutraler Node-Eintrag fuer einen minimalen Viewport-Geometry-Snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportNodeSnapshot {
    /// Stabile Node-ID.
    pub id: u64,
    /// Weltposition des Nodes.
    pub position: [f32; 2],
    /// Render-Klassifikation fuer die host-seitige Darstellung.
    pub kind: HostViewportNodeKind,
    /// Gibt an, ob der Node auch bei Decimation sichtbar bleiben soll.
    pub preserve_when_decimating: bool,
    /// Ob der Node aktuell selektiert ist.
    pub selected: bool,
    /// Ob der Node aktuell ausgeblendet ist.
    pub hidden: bool,
    /// Ob der Node aktuell gedimmt ist.
    pub dimmed: bool,
}

/// Host-neutrale Verbindung fuer einen minimalen Viewport-Geometry-Snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportConnectionSnapshot {
    /// Start-Node-ID.
    pub start_id: u64,
    /// End-Node-ID.
    pub end_id: u64,
    /// Weltposition des Startpunkts.
    pub start_position: [f32; 2],
    /// Weltposition des Endpunkts.
    pub end_position: [f32; 2],
    /// Richtungsklassifikation der Verbindung.
    pub direction: HostViewportConnectionDirection,
    /// Prioritaetsklassifikation der Verbindung.
    pub priority: HostViewportConnectionPriority,
    /// Ob die Verbindung ueber Hidden-Nodes aktuell ausgeblendet ist.
    pub hidden: bool,
    /// Ob die Verbindung ueber gedimmte Nodes aktuell gedimmt ist.
    pub dimmed: bool,
}

/// Host-neutraler Marker-Eintrag fuer einen minimalen Viewport-Geometry-Snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportMarkerSnapshot {
    /// Weltposition des Markers.
    pub position: [f32; 2],
}

/// Minimaler, serialisierbarer Viewport-Geometry-Snapshot fuer Transport-Adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportGeometrySnapshot {
    /// Ob aktuell eine Karte im Render-Snapshot vorhanden ist.
    pub has_map: bool,
    /// Viewport-Groesse in Pixeln [width, height].
    pub viewport_size: [f32; 2],
    /// Kameraposition in Weltkoordinaten.
    pub camera_position: [f32; 2],
    /// Zoom-Faktor des Frames.
    pub zoom: f32,
    /// Welt-Einheiten pro Pixel im aktuellen Frame.
    pub world_per_pixel: f32,
    /// Gibt an, ob fuer den Frame ein Hintergrund-Asset vorhanden ist.
    pub has_background: bool,
    /// Gibt an, ob der Hintergrund in diesem Frame sichtbar ist.
    pub background_visible: bool,
    /// Read-only Node-Snapshot fuer den aktuellen Frame.
    pub nodes: Vec<HostViewportNodeSnapshot>,
    /// Read-only Verbindungs-Snapshot fuer den aktuellen Frame.
    pub connections: Vec<HostViewportConnectionSnapshot>,
    /// Read-only Marker-Snapshot fuer den aktuellen Frame.
    pub markers: Vec<HostViewportMarkerSnapshot>,
}

/// Kleine, serialisierbare Session-Zusammenfassung fuer Host-Frontends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostSessionSnapshot {
    /// Ob aktuell eine Karte geladen ist.
    pub has_map: bool,
    /// Anzahl der Nodes der geladenen Karte.
    pub node_count: usize,
    /// Anzahl der Verbindungen der geladenen Karte.
    pub connection_count: usize,
    /// Aktives Editor-Tool als stabiler, expliziter Identifier.
    pub active_tool: HostActiveTool,
    /// Letzte Statusmeldung der Session.
    pub status_message: Option<String>,
    /// Ob die Command-Palette sichtbar ist.
    pub show_command_palette: bool,
    /// Ob der Options-Dialog sichtbar ist.
    pub show_options_dialog: bool,
    /// Gibt an, ob ein Undo-Schritt verfuegbar ist.
    pub can_undo: bool,
    /// Gibt an, ob ein Redo-Schritt verfuegbar ist.
    pub can_redo: bool,
    /// Anzahl aktuell ausstehender Dialog-Anforderungen.
    pub pending_dialog_request_count: usize,
    /// Read-only Snapshot der aktuellen Auswahl.
    pub selection: HostSelectionSnapshot,
    /// Read-only Snapshot des aktuellen Viewports.
    pub viewport: HostViewportSnapshot,
}

/// Host-neutrale Richtung fuer Verbindungs-Defaults im Chrome-Snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostDefaultConnectionDirection {
    /// Standardrichtung Start -> Ende.
    Regular,
    /// Bidirektionaler Standard.
    Dual,
    /// Umgekehrte Standardrichtung Ende -> Start.
    Reverse,
}

/// Host-neutrale Prioritaet fuer Verbindungs-Defaults im Chrome-Snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostDefaultConnectionPriority {
    /// Normale Verbindung.
    Regular,
    /// Subpriorisierte Verbindung.
    SubPriority,
}

/// Stabile Route-Tool-ID fuer host-neutrale Chrome-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolId {
    /// Gerade Strecke.
    Straight,
    /// Quadratische Bézier-Kurve.
    CurveQuad,
    /// Kubische Bézier-Kurve.
    CurveCubic,
    /// Catmull-Rom-Spline.
    Spline,
    /// Ausweichstrecke.
    Bypass,
    /// Geglaettete Kurve.
    SmoothCurve,
    /// Parkplatz-Generator.
    Parking,
    /// Feldgrenzen-Analyse.
    FieldBoundary,
    /// Feldweg-Analyse.
    FieldPath,
    /// Strecken-Versatz.
    RouteOffset,
    /// Farb-Pfad-Analyse.
    ColorPath,
}

/// Stabile Route-Tool-Gruppe fuer host-neutrale Chrome-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolGroup {
    /// Grundlegende Streckenwerkzeuge.
    Basics,
    /// Abschnitts- und Generator-Werkzeuge.
    Section,
    /// Analyse-Werkzeuge.
    Analysis,
}

/// Stabile Route-Tool-Surface fuer host-neutrale Chrome-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolSurface {
    /// Schwebendes Floating-Menue.
    FloatingMenu,
    /// Defaults-Panel in der Sidebar.
    DefaultsPanel,
    /// Hauptmenue.
    MainMenu,
    /// Command Palette.
    CommandPalette,
}

/// Stabile Icon-Klassifikation fuer Route-Tool-Eintraege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolIconKey {
    /// Icon fuer Gerade Strecke.
    Straight,
    /// Icon fuer Bézier Grad 2.
    CurveQuad,
    /// Icon fuer Bézier Grad 3.
    CurveCubic,
    /// Icon fuer Spline.
    Spline,
    /// Icon fuer Ausweichstrecke.
    Bypass,
    /// Icon fuer Geglaettete Kurve.
    SmoothCurve,
    /// Icon fuer Parkplatz.
    Parking,
    /// Icon fuer Feldgrenze.
    FieldBoundary,
    /// Icon fuer Feldweg.
    FieldPath,
    /// Icon fuer Streckenversatz.
    RouteOffset,
    /// Icon fuer Farbpfad.
    ColorPath,
}

/// Stabile Deaktivierungsgruende fuer Route-Tool-Eintraege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolDisabledReason {
    /// Farmland-Daten fehlen.
    MissingFarmland,
    /// Hintergrundbild fehlt.
    MissingBackground,
    /// Geordnete Ketten-Selektion fehlt.
    MissingOrderedChain,
}

/// Host-neutraler Route-Tool-Eintrag fuer Menues und Panels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRouteToolEntrySnapshot {
    /// Surface, fuer die der Eintrag aufgeloest wurde.
    pub surface: HostRouteToolSurface,
    /// Anzeigegruppe des Eintrags.
    pub group: HostRouteToolGroup,
    /// Stabile Tool-ID.
    pub tool: HostRouteToolId,
    /// Kanonischer Slot des Tools im Katalog.
    pub slot: usize,
    /// Stabile Icon-Klassifikation des Eintrags.
    pub icon_key: HostRouteToolIconKey,
    /// Gibt an, ob der Eintrag aktuell aktivierbar ist.
    pub enabled: bool,
    /// Optionaler Deaktivierungsgrund.
    pub disabled_reason: Option<HostRouteToolDisabledReason>,
}

/// Zuletzt gewaehlte Route-Tools je Gruppe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRouteToolSelectionSnapshot {
    /// Zuletzt gewaehltes Tool in der Gruppe `Basics`.
    pub basics: HostRouteToolId,
    /// Zuletzt gewaehltes Tool in der Gruppe `Section`.
    pub section: HostRouteToolId,
    /// Zuletzt gewaehltes Tool in der Gruppe `Analysis`.
    pub analysis: HostRouteToolId,
}

/// Host-neutraler Read-Snapshot fuer Chrome-nahe Menues und Panels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostChromeSnapshot {
    /// Letzte Statusmeldung der Session.
    pub status_message: Option<String>,
    /// Ob die Command-Palette sichtbar ist.
    pub show_command_palette: bool,
    /// Ob der Optionen-Dialog sichtbar ist.
    pub show_options_dialog: bool,
    /// Ob aktuell eine Karte geladen ist.
    pub has_map: bool,
    /// Ob aktuell mindestens ein Node selektiert ist.
    pub has_selection: bool,
    /// Ob die Zwischenablage Node-Daten enthaelt.
    pub has_clipboard: bool,
    /// Gibt an, ob ein Undo-Schritt verfuegbar ist.
    pub can_undo: bool,
    /// Gibt an, ob ein Redo-Schritt verfuegbar ist.
    pub can_redo: bool,
    /// Aktives Editor-Tool als stabiler Identifier.
    pub active_tool: HostActiveTool,
    /// Aktives Route-Tool im Route-Modus.
    pub active_route_tool: Option<HostRouteToolId>,
    /// Aktuelle Verbindungs-Default-Richtung.
    pub default_direction: HostDefaultConnectionDirection,
    /// Aktuelle Verbindungs-Default-Prioritaet.
    pub default_priority: HostDefaultConnectionPriority,
    /// Zuletzt gewaehlte Route-Tools je Gruppe.
    pub route_tool_memory: HostRouteToolSelectionSnapshot,
    /// Vollstaendige Laufzeitoptionen fuer host-neutrale Panels.
    pub options: EditorOptions,
    /// Aufgeloeste Route-Tool-Eintraege fuer Menues und Panels.
    pub route_tool_entries: Vec<HostRouteToolEntrySnapshot>,
}

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineActiveTool = HostActiveTool;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogRequestKind = HostDialogRequestKind;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogRequest = HostDialogRequest;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogResult = HostDialogResult;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EnginePointerButton = HostPointerButton;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineTapKind = HostTapKind;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineInputModifiers = HostInputModifiers;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportInputBatch = HostViewportInputBatch;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportInputEvent = HostViewportInputEvent;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSessionAction = HostSessionAction;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSelectionSnapshot = HostSelectionSnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportSnapshot = HostViewportSnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportGeometrySnapshot = HostViewportGeometrySnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSessionSnapshot = HostSessionSnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDefaultConnectionDirection = HostDefaultConnectionDirection;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDefaultConnectionPriority = HostDefaultConnectionPriority;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolId = HostRouteToolId;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolGroup = HostRouteToolGroup;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolSurface = HostRouteToolSurface;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolIconKey = HostRouteToolIconKey;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolDisabledReason = HostRouteToolDisabledReason;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolEntrySnapshot = HostRouteToolEntrySnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineRouteToolSelectionSnapshot = HostRouteToolSelectionSnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineChromeSnapshot = HostChromeSnapshot;

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::shared::EditorOptions;
    use serde_json::json;

    use super::{
        EngineActiveTool, EngineChromeSnapshot, EngineDialogRequestKind, EngineDialogResult,
        EngineInputModifiers, EnginePointerButton, EngineSessionAction, EngineSessionSnapshot,
        EngineTapKind, EngineViewportGeometrySnapshot, EngineViewportInputBatch,
        EngineViewportInputEvent, HostActiveTool, HostChromeSnapshot,
        HostDefaultConnectionDirection, HostDefaultConnectionPriority, HostDialogResult,
        HostInputModifiers, HostPointerButton, HostRouteToolDisabledReason,
        HostRouteToolEntrySnapshot, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId,
        HostRouteToolSelectionSnapshot, HostRouteToolSurface, HostSelectionSnapshot,
        HostSessionAction, HostSessionSnapshot, HostTapKind, HostViewportConnectionDirection,
        HostViewportConnectionPriority, HostViewportConnectionSnapshot,
        HostViewportGeometrySnapshot, HostViewportInputBatch, HostViewportInputEvent,
        HostViewportMarkerSnapshot, HostViewportNodeKind, HostViewportNodeSnapshot,
        HostViewportSnapshot,
    };

    #[test]
    fn engine_session_action_alias_uses_stable_host_json_contract() {
        let action = EngineSessionAction::SetEditorTool {
            tool: EngineActiveTool::Route,
        };

        let payload = serde_json::to_value(&action)
            .expect("SetEditorTool muss als stabiles Host-JSON serialisierbar sein");
        assert_eq!(
            payload,
            json!({ "kind": "set_editor_tool", "tool": "route" })
        );

        let parsed: HostSessionAction = serde_json::from_value(payload)
            .expect("Alias-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostSessionAction::SetEditorTool {
                tool: HostActiveTool::Route,
            }
        );
    }

    #[test]
    fn engine_dialog_result_alias_roundtrips_with_host_json_shape() {
        let result = EngineDialogResult::PathSelected {
            kind: EngineDialogRequestKind::BackgroundMap,
            path: "/tmp/overview.zip".to_string(),
        };

        let payload = serde_json::to_value(&result)
            .expect("Dialog-Ergebnis muss als stabiles Host-JSON serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "status": "path_selected",
                "kind": "background_map",
                "path": "/tmp/overview.zip"
            })
        );

        let parsed: HostDialogResult = serde_json::from_value(payload)
            .expect("Alias-Dialog-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostDialogResult::PathSelected {
                kind: EngineDialogRequestKind::BackgroundMap,
                path: "/tmp/overview.zip".to_string(),
            }
        );
    }

    #[test]
    fn engine_session_snapshot_alias_roundtrips_without_schema_drift() {
        let host_snapshot = HostSessionSnapshot {
            has_map: true,
            node_count: 7,
            connection_count: 9,
            active_tool: HostActiveTool::Connect,
            status_message: Some("bereit".to_string()),
            show_command_palette: true,
            show_options_dialog: false,
            can_undo: true,
            can_redo: false,
            pending_dialog_request_count: 2,
            selection: HostSelectionSnapshot {
                selected_node_ids: vec![11, 42],
            },
            viewport: HostViewportSnapshot {
                camera_position: [12.5, -8.0],
                zoom: 1.25,
            },
        };

        let payload = serde_json::to_value(&host_snapshot)
            .expect("HostSnapshot muss fuer den Alias-Contract serialisierbar sein");

        let alias_snapshot: EngineSessionSnapshot = serde_json::from_value(payload.clone())
            .expect("EngineSessionSnapshot-Alias muss kanonisches Host-JSON lesen koennen");
        let canonical_snapshot: HostSessionSnapshot = serde_json::from_value(payload)
            .expect("HostSessionSnapshot muss das gleiche JSON lesen koennen");

        assert_eq!(alias_snapshot, host_snapshot);
        assert_eq!(canonical_snapshot, host_snapshot);
    }

    #[test]
    fn engine_chrome_snapshot_alias_roundtrips_with_route_tool_metadata() {
        let host_snapshot = HostChromeSnapshot {
            status_message: Some("bereit".to_string()),
            show_command_palette: true,
            show_options_dialog: false,
            has_map: true,
            has_selection: true,
            has_clipboard: false,
            can_undo: true,
            can_redo: false,
            active_tool: HostActiveTool::Route,
            active_route_tool: Some(HostRouteToolId::CurveCubic),
            default_direction: HostDefaultConnectionDirection::Dual,
            default_priority: HostDefaultConnectionPriority::SubPriority,
            route_tool_memory: HostRouteToolSelectionSnapshot {
                basics: HostRouteToolId::CurveCubic,
                section: HostRouteToolId::Bypass,
                analysis: HostRouteToolId::FieldBoundary,
            },
            options: EditorOptions::default(),
            route_tool_entries: vec![
                HostRouteToolEntrySnapshot {
                    surface: HostRouteToolSurface::DefaultsPanel,
                    group: HostRouteToolGroup::Basics,
                    tool: HostRouteToolId::CurveCubic,
                    slot: 2,
                    icon_key: HostRouteToolIconKey::CurveCubic,
                    enabled: true,
                    disabled_reason: None,
                },
                HostRouteToolEntrySnapshot {
                    surface: HostRouteToolSurface::MainMenu,
                    group: HostRouteToolGroup::Analysis,
                    tool: HostRouteToolId::FieldPath,
                    slot: 8,
                    icon_key: HostRouteToolIconKey::FieldPath,
                    enabled: false,
                    disabled_reason: Some(HostRouteToolDisabledReason::MissingFarmland),
                },
            ],
        };

        let payload = serde_json::to_value(&host_snapshot)
            .expect("Chrome-Snapshot muss fuer den Alias-Contract serialisierbar sein");
        let payload_obj = payload
            .as_object()
            .expect("Chrome-Snapshot muss als JSON-Objekt serialisiert werden");
        assert_eq!(payload_obj.get("active_tool"), Some(&json!("route")));
        assert_eq!(
            payload_obj.get("active_route_tool"),
            Some(&json!("curve_cubic"))
        );
        assert_eq!(payload_obj.get("default_direction"), Some(&json!("dual")));
        assert_eq!(
            payload_obj.get("default_priority"),
            Some(&json!("sub_priority"))
        );

        let route_tool_entries = payload_obj
            .get("route_tool_entries")
            .and_then(|entries| entries.as_array())
            .expect("Route-Tool-Eintraege muessen als JSON-Array serialisiert werden");
        assert_eq!(route_tool_entries.len(), 2);
        assert_eq!(
            route_tool_entries[0],
            json!({
                "surface": "defaults_panel",
                "group": "basics",
                "tool": "curve_cubic",
                "slot": 2,
                "icon_key": "curve_cubic",
                "enabled": true,
                "disabled_reason": null
            })
        );
        assert_eq!(
            route_tool_entries[1],
            json!({
                "surface": "main_menu",
                "group": "analysis",
                "tool": "field_path",
                "slot": 8,
                "icon_key": "field_path",
                "enabled": false,
                "disabled_reason": "missing_farmland"
            })
        );

        let options = payload_obj
            .get("options")
            .and_then(|options| options.as_object())
            .expect("Optionen muessen als JSON-Objekt serialisiert werden");
        assert_eq!(options.get("language"), Some(&json!("De")));

        let alias_snapshot: EngineChromeSnapshot = serde_json::from_value(payload.clone())
            .expect("EngineChromeSnapshot-Alias muss kanonisches Host-JSON lesen koennen");
        let canonical_snapshot: HostChromeSnapshot = serde_json::from_value(payload)
            .expect("HostChromeSnapshot muss das gleiche JSON lesen koennen");

        assert_eq!(alias_snapshot.route_tool_entries.len(), 2);
        assert_eq!(canonical_snapshot.route_tool_entries.len(), 2);
        assert!(alias_snapshot.show_command_palette);
        assert_eq!(
            canonical_snapshot.default_direction,
            HostDefaultConnectionDirection::Dual
        );
        assert_eq!(
            alias_snapshot.options.language,
            host_snapshot.options.language
        );
    }

    #[test]
    fn viewport_input_batch_roundtrips_with_stable_json_shape() {
        let batch = HostViewportInputBatch {
            events: vec![
                HostViewportInputEvent::Resize {
                    size_px: [1280.0, 720.0],
                },
                HostViewportInputEvent::Tap {
                    button: HostPointerButton::Primary,
                    tap_kind: HostTapKind::Single,
                    screen_pos: [32.0, 48.0],
                    modifiers: HostInputModifiers {
                        shift: true,
                        alt: false,
                        command: true,
                    },
                },
                HostViewportInputEvent::Tap {
                    button: HostPointerButton::Primary,
                    tap_kind: HostTapKind::Double,
                    screen_pos: [64.0, 96.0],
                    modifiers: HostInputModifiers::default(),
                },
                HostViewportInputEvent::DragEnd {
                    button: HostPointerButton::Secondary,
                    screen_pos: None,
                },
                HostViewportInputEvent::Scroll {
                    screen_pos: Some([300.0, 200.0]),
                    smooth_delta_y: 12.0,
                    raw_delta_y: 1.0,
                    modifiers: HostInputModifiers::default(),
                },
            ],
        };

        let payload = serde_json::to_value(&batch)
            .expect("Viewport-Input-Batch muss stabil serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "events": [
                    { "kind": "resize", "size_px": [1280.0, 720.0] },
                    {
                        "kind": "tap",
                        "button": "primary",
                        "tap_kind": "single",
                        "screen_pos": [32.0, 48.0],
                        "modifiers": {
                            "shift": true,
                            "alt": false,
                            "command": true
                        }
                    },
                    {
                        "kind": "tap",
                        "button": "primary",
                        "tap_kind": "double",
                        "screen_pos": [64.0, 96.0],
                        "modifiers": {
                            "shift": false,
                            "alt": false,
                            "command": false
                        }
                    },
                    {
                        "kind": "drag_end",
                        "button": "secondary",
                        "screen_pos": null
                    },
                    {
                        "kind": "scroll",
                        "screen_pos": [300.0, 200.0],
                        "smooth_delta_y": 12.0,
                        "raw_delta_y": 1.0,
                        "modifiers": {
                            "shift": false,
                            "alt": false,
                            "command": false
                        }
                    }
                ]
            })
        );

        let parsed: HostViewportInputBatch =
            serde_json::from_value(payload).expect("Viewport-Input-Batch muss wieder lesbar sein");
        assert_eq!(parsed, batch);
    }

    #[test]
    fn engine_viewport_input_alias_roundtrips_with_canonical_host_contract() {
        let action = EngineSessionAction::SubmitViewportInput {
            batch: EngineViewportInputBatch {
                events: vec![EngineViewportInputEvent::Tap {
                    button: EnginePointerButton::Primary,
                    tap_kind: EngineTapKind::Double,
                    screen_pos: [10.0, 20.0],
                    modifiers: EngineInputModifiers {
                        shift: false,
                        alt: false,
                        command: true,
                    },
                }],
            },
        };

        let payload = serde_json::to_value(&action)
            .expect("Viewport-Input-Alias muss stabil serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "kind": "submit_viewport_input",
                "batch": {
                    "events": [{
                        "kind": "tap",
                        "button": "primary",
                        "tap_kind": "double",
                        "screen_pos": [10.0, 20.0],
                        "modifiers": {
                            "shift": false,
                            "alt": false,
                            "command": true
                        }
                    }]
                }
            })
        );

        let parsed: HostSessionAction = serde_json::from_value(payload)
            .expect("Alias-JSON muss in den kanonischen Host-Typ zuruecklesbar sein");
        assert_eq!(
            parsed,
            HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![HostViewportInputEvent::Tap {
                        button: HostPointerButton::Primary,
                        tap_kind: HostTapKind::Double,
                        screen_pos: [10.0, 20.0],
                        modifiers: HostInputModifiers {
                            shift: false,
                            alt: false,
                            command: true,
                        },
                    }],
                },
            }
        );
    }

    #[test]
    fn engine_viewport_geometry_snapshot_alias_roundtrips_without_schema_drift() {
        let host_snapshot = HostViewportGeometrySnapshot {
            has_map: true,
            viewport_size: [1280.0, 720.0],
            camera_position: [32.0, -16.0],
            zoom: 1.5,
            world_per_pixel: 0.75,
            has_background: true,
            background_visible: true,
            nodes: vec![HostViewportNodeSnapshot {
                id: 7,
                position: [10.0, 20.0],
                kind: HostViewportNodeKind::Warning,
                preserve_when_decimating: true,
                selected: true,
                hidden: false,
                dimmed: false,
            }],
            connections: vec![HostViewportConnectionSnapshot {
                start_id: 7,
                end_id: 8,
                start_position: [10.0, 20.0],
                end_position: [15.0, 25.0],
                direction: HostViewportConnectionDirection::Dual,
                priority: HostViewportConnectionPriority::SubPriority,
                hidden: false,
                dimmed: true,
            }],
            markers: vec![HostViewportMarkerSnapshot {
                position: [12.0, 18.0],
            }],
        };

        let payload = serde_json::to_value(&host_snapshot)
            .expect("Geometry-Snapshot muss fuer den Alias-Contract serialisierbar sein");

        let alias_snapshot: EngineViewportGeometrySnapshot = serde_json::from_value(
            payload.clone(),
        )
        .expect("EngineViewportGeometrySnapshot-Alias muss kanonisches Host-JSON lesen koennen");
        let canonical_snapshot: HostViewportGeometrySnapshot = serde_json::from_value(payload)
            .expect("HostViewportGeometrySnapshot muss das gleiche JSON lesen koennen");

        assert_eq!(alias_snapshot, host_snapshot);
        assert_eq!(canonical_snapshot, host_snapshot);
    }
}
