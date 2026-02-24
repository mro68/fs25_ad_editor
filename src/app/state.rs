//! Application State — zentrale Datenhaltung.

use super::history::Snapshot;
use super::segment_registry::SegmentRegistry;
use super::tools::ToolManager;
use super::CommandLog;
use crate::core::Camera2D;
use crate::core::{BackgroundMap, ConnectionDirection, ConnectionPriority, RoadMap};
use crate::shared::{EditorOptions, OverviewLayerOptions, RenderQuality};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

/// Aktives Editor-Werkzeug
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTool {
    /// Standard: Nodes selektieren und verschieben
    #[default]
    Select,
    /// Verbindungen zwischen Nodes erstellen
    Connect,
    /// Neue Nodes auf der Karte platzieren
    AddNode,
    /// Route-Tools (Linie, Parkplatz, Kurve, …)
    Route,
}

/// Zustand des aktuellen Editor-Werkzeugs
pub struct EditorToolState {
    /// Aktives Werkzeug
    pub active_tool: EditorTool,
    /// Quell-Node für Connect-Tool (wartet auf Ziel)
    pub connect_source_node: Option<u64>,
    /// Standard-Richtung für neue Verbindungen
    pub default_direction: ConnectionDirection,
    /// Standard-Straßenart für neue Verbindungen
    pub default_priority: ConnectionPriority,
    /// Route-Tool-Manager (Linie, Parkplatz, Kurve, …)
    pub tool_manager: ToolManager,
}

impl Default for EditorToolState {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorToolState {
    /// Erstellt den Standard-Werkzeugzustand (Select-Tool aktiv).
    pub fn new() -> Self {
        Self {
            active_tool: EditorTool::Select,
            connect_source_node: None,
            default_direction: ConnectionDirection::Regular,
            default_priority: ConnectionPriority::Regular,
            tool_manager: ToolManager::new(),
        }
    }
}

/// Auswahlbezogener Anwendungszustand
#[derive(Clone, Default)]
pub struct SelectionState {
    /// Menge der aktuell selektierten Node-IDs (Arc für O(1)-Clone in RenderScene)
    pub selected_node_ids: Arc<HashSet<u64>>,
    /// Letzter selektierter Node als Anker für additive Bereichsselektion
    pub selection_anchor_node_id: Option<u64>,
}

impl SelectionState {
    /// Erstellt einen leeren Selektionszustand.
    pub fn new() -> Self {
        Self {
            selected_node_ids: Arc::new(HashSet::new()),
            selection_anchor_node_id: None,
        }
    }

    /// Gibt eine mutable Referenz auf die HashSet zurück (CoW: klont nur wenn nötig).
    ///
    /// Alle Mutationen der Selektion gehen über diese Methode, damit der
    /// Arc-Klon in `RenderScene::build()` O(1) bleibt.
    #[inline]
    pub fn ids_mut(&mut self) -> &mut HashSet<u64> {
        Arc::make_mut(&mut self.selected_node_ids)
    }
}

/// Zustand des Marker-Bearbeiten-Dialogs
#[derive(Default)]
pub struct MarkerDialogState {
    /// Ob der Dialog sichtbar ist
    pub visible: bool,
    /// Node-ID des Markers im Dialog
    pub node_id: Option<u64>,
    /// Marker-Name im Dialog
    pub name: String,
    /// Marker-Gruppe im Dialog
    pub group: String,
    /// Neuer Marker (true) oder bestehender editieren (false)
    pub is_new: bool,
}

impl MarkerDialogState {
    /// Erstellt einen geschlossenen Marker-Dialog-Zustand.
    pub fn new() -> Self {
        Self {
            visible: false,
            node_id: None,
            name: String::new(),
            group: String::new(),
            is_new: true,
        }
    }
}

/// Zustand des Duplikat-Bestätigungsdialogs
#[derive(Default)]
pub struct DedupDialogState {
    /// Ob der Dialog sichtbar ist
    pub visible: bool,
    /// Anzahl gefundener Duplikat-Nodes
    pub duplicate_count: u32,
    /// Anzahl der Positions-Gruppen mit Duplikaten
    pub group_count: u32,
}

impl DedupDialogState {
    /// Erstellt einen geschlossenen Dedup-Dialog-Zustand.
    pub fn new() -> Self {
        Self {
            visible: false,
            duplicate_count: 0,
            group_count: 0,
        }
    }
}

/// Zustand des Übersichtskarten-Options-Dialogs
#[derive(Default)]
pub struct OverviewOptionsDialogState {
    /// Ob der Dialog sichtbar ist
    pub visible: bool,
    /// ZIP-Pfad der gewählten Map-Mod-Datei
    pub zip_path: String,
    /// Layer-Optionen (Arbeitskopie für den Dialog)
    pub layers: OverviewLayerOptions,
}

impl OverviewOptionsDialogState {
    /// Erstellt einen geschlossenen Dialog-Zustand.
    pub fn new() -> Self {
        Self {
            visible: false,
            zip_path: String::new(),
            layers: OverviewLayerOptions::default(),
        }
    }
}

/// Zustand des Post-Load-Dialogs (automatische Erkennung nach XML-Laden).
#[derive(Default)]
pub struct PostLoadDialogState {
    /// Ob der Dialog sichtbar ist
    pub visible: bool,
    /// Heightmap wurde automatisch gesetzt
    pub heightmap_set: bool,
    /// Pfad zur automatisch gesetzten Heightmap
    pub heightmap_path: Option<String>,
    /// Gefundene passende ZIP-Dateien im Mods-Verzeichnis
    pub matching_zips: Vec<PathBuf>,
    /// Index des vom User ausgewählten ZIPs (Default: 0)
    pub selected_zip_index: usize,
    /// Map-Name zur Anzeige im Dialog
    pub map_name: String,
}

impl PostLoadDialogState {
    /// Erstellt einen geschlossenen Post-Load-Dialog-Zustand.
    pub fn new() -> Self {
        Self {
            visible: false,
            heightmap_set: false,
            heightmap_path: None,
            matching_zips: Vec::new(),
            selected_zip_index: 0,
            map_name: String::new(),
        }
    }
}

/// Konfiguration für das Distanzen-Neuverteilen-Feature im Eigenschaften-Bereich.
#[derive(Debug, Clone)]
pub struct DistanzenState {
    /// true = nach Anzahl, false = nach Abstand
    pub by_count: bool,
    /// Gewünschte Anzahl an Waypoints (bei `by_count = true`)
    pub count: u32,
    /// Maximaler Abstand zwischen Waypoints in Welteinheiten (bei `by_count = false`)
    pub distance: f32,
    /// Berechnete Streckenlänge der aktuellen Selektion (für wechselseitige Berechnung)
    pub path_length: f32,
    /// Vorschau-Modus aktiv (Spline-Preview wird im Viewport gezeichnet)
    pub active: bool,
    /// Originale Strecke während der Vorschau ausblenden
    pub hide_original: bool,
    /// Vorschau-Positionen (berechnete Resample-Punkte für Overlay)
    pub preview_positions: Vec<glam::Vec2>,
}

impl Default for DistanzenState {
    fn default() -> Self {
        Self {
            by_count: false,
            count: 10,
            distance: 6.0,
            path_length: 0.0,
            active: false,
            hide_original: true,
            preview_positions: Vec::new(),
        }
    }
}

impl DistanzenState {
    /// Aktualisiert count aus distance (und umgekehrt) basierend auf der Streckenlänge.
    pub fn sync_from_distance(&mut self) {
        if self.path_length > 0.0 && self.distance > 0.0 {
            self.count = ((self.path_length / self.distance).round() as u32 + 1).max(2);
        }
    }

    /// Aktualisiert distance aus count basierend auf der Streckenlänge.
    pub fn sync_from_count(&mut self) {
        if self.path_length > 0.0 && self.count >= 2 {
            self.distance = (self.path_length / (self.count - 1) as f32).max(0.5);
        }
    }

    /// Deaktiviert den Vorschau-Modus und löscht die Vorschau-Daten.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.preview_positions.clear();
    }

    /// Gibt `true` zurück wenn die Originalstrecke aktuell ausgeblendet werden soll.
    pub fn should_hide_original(&self) -> bool {
        self.active && self.hide_original
    }
}

/// UI-bezogener Anwendungszustand
#[derive(Default)]
pub struct UiState {
    /// Ob der Open-Datei-Dialog geöffnet werden soll
    pub show_file_dialog: bool,
    /// Ob der Save-Datei-Dialog geöffnet werden soll
    pub show_save_file_dialog: bool,
    /// Ob der Heightmap-Auswahl-Dialog geöffnet werden soll
    pub show_heightmap_dialog: bool,
    /// Ob der Background-Map-Auswahl-Dialog geöffnet werden soll
    pub show_background_map_dialog: bool,
    /// Ob der Übersichtskarten-ZIP-Auswahl-Dialog geöffnet werden soll
    pub show_overview_dialog: bool,
    /// Ob die Heightmap-Warnung angezeigt werden soll
    pub show_heightmap_warning: bool,
    /// Ob die Heightmap-Warnung für diese Save-Operation bereits bestätigt wurde
    pub heightmap_warning_confirmed: bool,
    /// Pfad für Save-Operation nach Heightmap-Warnung
    pub pending_save_path: Option<String>,
    /// Pfad der aktuell geladenen Datei (für Save ohne Dialog)
    pub current_file_path: Option<String>,
    /// Pfad der aktuell ausgewählten Heightmap (optional)
    pub heightmap_path: Option<String>,
    /// Marker-Bearbeiten-Dialog
    pub marker_dialog: MarkerDialogState,
    /// Temporäre Statusnachricht (z.B. Duplikat-Bereinigung)
    pub status_message: Option<String>,
    /// Duplikat-Bestätigungsdialog
    pub dedup_dialog: DedupDialogState,
    /// ZIP-Browser-Dialog für Background-Map-Auswahl
    pub zip_browser: Option<ZipBrowserState>,
    /// Übersichtskarten-Optionen-Dialog
    pub overview_options_dialog: OverviewOptionsDialogState,
    /// Post-Load-Dialog (Heightmap/ZIP-Erkennung)
    pub post_load_dialog: PostLoadDialogState,
    /// Distanzen-Neuverteilen-Konfiguration (Eigenschaften-Panel)
    pub distanzen: DistanzenState,
}

/// Zustand des ZIP-Browser-Dialogs.
#[derive(Debug, Clone)]
pub struct ZipBrowserState {
    /// Pfad zur ZIP-Datei
    pub zip_path: String,
    /// Bilddateien im Archiv (mit Dateigröße)
    pub entries: Vec<crate::core::ZipImageEntry>,
    /// Index des aktuell selektierten Eintrags
    pub selected: Option<usize>,
    /// Nur *overview*-Dateien anzeigen
    pub filter_overview: bool,
}

impl UiState {
    /// Erstellt den Standard-UI-Zustand (alle Dialoge geschlossen).
    pub fn new() -> Self {
        Self {
            show_file_dialog: false,
            show_save_file_dialog: false,
            show_heightmap_dialog: false,
            show_background_map_dialog: false,
            show_overview_dialog: false,
            show_heightmap_warning: false,
            heightmap_warning_confirmed: false,
            pending_save_path: None,
            current_file_path: None,
            heightmap_path: None,
            marker_dialog: MarkerDialogState::new(),
            status_message: None,
            dedup_dialog: DedupDialogState::new(),
            zip_browser: None,
            overview_options_dialog: OverviewOptionsDialogState::new(),
            post_load_dialog: PostLoadDialogState::new(),
            distanzen: DistanzenState::default(),
        }
    }
}

/// View-bezogener Anwendungszustand
#[derive(Default)]
pub struct ViewState {
    /// 2D-Kamera für die Ansicht
    pub camera: Camera2D,
    /// Aktuelle Viewport-Größe in Pixel
    pub viewport_size: [f32; 2],
    /// Qualitätsstufe für Kantenglättung
    pub render_quality: RenderQuality,
    /// Background-Map (optional)
    pub background_map: Option<Arc<BackgroundMap>>,
    /// Background-Opacity (0.0 = transparent, 1.0 = opak)
    pub background_opacity: f32,
    /// Background-Sichtbarkeit
    pub background_visible: bool,
    /// Skalierungsfaktor für Background-Map-Ausdehnung (1.0 = Original)
    pub background_scale: f32,
    /// Signalisiert, dass die Background-Map neu in den GPU-Renderer hochgeladen werden muss
    pub background_dirty: bool,
}

impl ViewState {
    /// Erstellt den Standard-View-Zustand.
    pub fn new() -> Self {
        Self {
            camera: Camera2D::new(),
            viewport_size: [0.0, 0.0],
            render_quality: RenderQuality::High,
            background_map: None,
            background_opacity: 1.0,
            background_visible: true,
            background_scale: 1.0,
            background_dirty: false,
        }
    }
}

/// Hauptzustand der Anwendung
pub struct AppState {
    /// Aktuell geladene RoadMap (None = keine Datei geladen)
    pub road_map: Option<Arc<RoadMap>>,
    /// View-State
    pub view: ViewState,
    /// UI-State
    pub ui: UiState,
    /// Selection-State
    pub selection: SelectionState,
    /// Editor-Werkzeug-State
    pub editor: EditorToolState,
    /// Verlauf ausgeführter Commands
    pub command_log: CommandLog,
    /// Undo/Redo-History (Snapshot-basiert)
    pub history: super::history::EditHistory,
    /// Laufzeit-Optionen (Farben, Größen, Breiten)
    pub options: EditorOptions,
    /// Ob der Options-Dialog angezeigt wird
    pub show_options_dialog: bool,
    /// In-Session-Registry aller erstellten Segmente (für nachträgliche Bearbeitung)
    pub segment_registry: SegmentRegistry,
    /// Signalisiert dem Host (eframe), die Anwendung kontrolliert zu beenden
    pub should_exit: bool,
}

impl AppState {
    /// Erstellt einen neuen, leeren App-State
    pub fn new() -> Self {
        Self {
            road_map: None,
            view: ViewState::new(),
            ui: UiState::new(),
            selection: SelectionState::new(),
            editor: EditorToolState::new(),
            command_log: CommandLog::new(),
            history: super::history::EditHistory::new_with_capacity(200),
            options: EditorOptions::default(),
            show_options_dialog: false,
            segment_registry: SegmentRegistry::new(),
            should_exit: false,
        }
    }

    /// Gibt die Anzahl der Nodes zurück (für UI-Anzeige)
    pub fn node_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.node_count())
    }

    /// Gibt die Anzahl der Connections zurück (für UI-Anzeige)
    pub fn connection_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.connection_count())
    }

    /// Undo/Redo helpers
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Gibt zurück, ob ein Redo-Schritt verfügbar ist.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Erstellt einen Undo-Snapshot des aktuellen Zustands.
    /// Reduziert Boilerplate in mutierenden Use-Cases.
    pub fn record_undo_snapshot(&mut self) {
        let snap = Snapshot::from_state(self);
        self.history.record_snapshot(snap);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
