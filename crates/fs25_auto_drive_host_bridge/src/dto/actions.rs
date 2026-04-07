//! Stabiler Aktions-Satz fuer die kanonische Session-Surface der Host-Bridge.

use fs25_auto_drive_engine::app::ui_contract::RouteToolPanelAction;
use fs25_auto_drive_engine::shared::EditorOptions;
use serde::{Deserialize, Serialize};

use super::dialogs::HostDialogResult;
use super::input::HostViewportInputBatch;
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
