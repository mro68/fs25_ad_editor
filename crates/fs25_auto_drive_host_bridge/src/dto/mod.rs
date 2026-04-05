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

/// Explizite Host-Aktionen fuer die gemeinsame Bridge-Session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineActiveTool = HostActiveTool;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogRequestKind = HostDialogRequestKind;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogRequest = HostDialogRequest;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineDialogResult = HostDialogResult;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSessionAction = HostSessionAction;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSelectionSnapshot = HostSelectionSnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineViewportSnapshot = HostViewportSnapshot;

/// Kompatibilitaetsalias fuer bestehende Flutter-/FFI-Call-Sites.
pub type EngineSessionSnapshot = HostSessionSnapshot;
