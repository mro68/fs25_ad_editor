use crate::shared::OverviewLayerOptions;
use std::path::PathBuf;

/// Zustand eines schwebenden Kontextmenues.
#[derive(Debug, Clone, Copy)]
pub struct FloatingMenuState {
    /// Art des aktuell geoeffneten Menues.
    pub kind: FloatingMenuKind,
    /// Bildschirmposition des Menues.
    pub pos: egui::Pos2,
}

/// Typ des schwebenden Kontextmenues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatingMenuKind {
    /// Werkzeug-Menue (Select/Connect/AddNode).
    Tools,
    /// Basis-Menue (Gerade/Kurve/Constraint).
    Basics,
    /// Menue fuer Abschnittswerkzeuge.
    SectionTools,
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

/// Zustand des Duplikat-Bestaetigungsdialogs
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

/// Zustand des Uebersichtskarten-Options-Dialogs
#[derive(Default)]
pub struct OverviewOptionsDialogState {
    /// Ob der Dialog sichtbar ist
    pub visible: bool,
    /// ZIP-Pfad der gewaehlten Map-Mod-Datei
    pub zip_path: String,
    /// Layer-Optionen (Arbeitskopie fuer den Dialog)
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
    /// overview.png wurde automatisch als Hintergrund geladen
    pub overview_loaded: bool,
    /// Gefundene passende ZIP-Dateien im Mods-Verzeichnis
    pub matching_zips: Vec<PathBuf>,
    /// Index des vom User ausgewaehlten ZIPs (Default: 0)
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
            overview_loaded: false,
            matching_zips: Vec::new(),
            selected_zip_index: 0,
            map_name: String::new(),
        }
    }
}

/// Dialog-State fuer "Als overview.jpg speichern"-Abfrage nach ZIP-Extraktion.
#[derive(Default)]
pub struct SaveOverviewDialogState {
    /// Ob der Dialog sichtbar ist
    pub visible: bool,
    /// Ziel-Pfad: overview.jpg im XML-Verzeichnis
    pub target_path: String,
    /// true wenn eine bestehende overview.jpg ueberschrieben wuerde
    pub is_overwrite: bool,
}

/// Zustand des Segment-Einstellungs-Popups (erscheint nach Doppelklick auf einen Segment-Node).
#[derive(Debug, Clone)]
pub struct SegmentSettingsPopupState {
    /// Ob das Popup sichtbar ist.
    pub visible: bool,
    /// Welt-Position des Doppelklicks (fuer Neu-Selektion bei Parameteraenderung).
    pub world_pos: glam::Vec2,
}

impl Default for SegmentSettingsPopupState {
    fn default() -> Self {
        Self {
            visible: false,
            world_pos: glam::Vec2::ZERO,
        }
    }
}

/// Einstellungen fuer den "Alle Felder nachzeichnen"-Dialog.
#[derive(Debug, Clone)]
pub struct TraceAllFieldsDialogState {
    /// Ob der Dialog sichtbar ist.
    pub visible: bool,
    /// Abstand zwischen generierten Wegpunkten in Welteinheiten (Meter).
    pub spacing: f32,
    /// Versatz vom Feldrand nach innen (positiv = nach innen, negativ = nach aussen).
    pub offset: f32,
    /// Begradigung: Douglas-Peucker-Toleranz in Welteinheiten (0 = kein).
    pub tolerance: f32,
}

impl Default for TraceAllFieldsDialogState {
    fn default() -> Self {
        Self {
            visible: false,
            spacing: 10.0,
            offset: 0.0,
            tolerance: 0.0,
        }
    }
}

/// Konfiguration fuer das Distanzen-Neuverteilen-Feature im Eigenschaften-Bereich.
#[derive(Debug, Clone)]
pub struct DistanzenState {
    /// true = nach Anzahl, false = nach Abstand
    pub by_count: bool,
    /// Gewuenschte Anzahl an Waypoints (bei `by_count = true`)
    pub count: u32,
    /// Maximaler Abstand zwischen Waypoints in Welteinheiten (bei `by_count = false`)
    pub distance: f32,
    /// Berechnete Streckenlaenge der aktuellen Selektion (fuer wechselseitige Berechnung)
    pub path_length: f32,
    /// Vorschau-Modus aktiv (Spline-Preview wird im Viewport gezeichnet)
    pub active: bool,
    /// Originale Strecke waehrend der Vorschau ausblenden
    pub hide_original: bool,
    /// Vorschau-Positionen (berechnete Resample-Punkte fuer Overlay)
    pub preview_positions: Vec<glam::Vec2>,
    /// Signatur der letzten Eingaben fuer Preview-Recompute (0 = ungueltig).
    pub preview_cache_signature: u64,
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
            preview_cache_signature: 0,
        }
    }
}

impl DistanzenState {
    /// Aktualisiert count aus distance (und umgekehrt) basierend auf der Streckenlaenge.
    pub fn sync_from_distance(&mut self) {
        if self.path_length > 0.0 && self.distance > 0.0 {
            self.count = ((self.path_length / self.distance).round() as u32 + 1).max(2);
        }
    }

    /// Aktualisiert distance aus count basierend auf der Streckenlaenge.
    pub fn sync_from_count(&mut self) {
        if self.path_length > 0.0 && self.count >= 2 {
            self.distance = (self.path_length / (self.count - 1) as f32).max(1.0);
        }
    }

    /// Deaktiviert den Vorschau-Modus und loescht die Vorschau-Daten.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.preview_positions.clear();
        self.preview_cache_signature = 0;
    }

    /// Gibt `true` zurueck wenn die Originalstrecke aktuell ausgeblendet werden soll.
    pub fn should_hide_original(&self) -> bool {
        self.active && self.hide_original
    }
}

/// Zustand des ZIP-Browser-Dialogs.
#[derive(Debug, Clone)]
pub struct ZipBrowserState {
    /// Pfad zur ZIP-Datei
    pub zip_path: String,
    /// Bilddateien im Archiv (mit Dateigroesse)
    pub entries: Vec<crate::core::ZipImageEntry>,
    /// Index des aktuell selektierten Eintrags
    pub selected: Option<usize>,
    /// Nur *overview*-Dateien anzeigen
    pub filter_overview: bool,
}

/// UI-bezogener Anwendungszustand
#[derive(Default)]
pub struct UiState {
    /// Ob der Open-Datei-Dialog geoeffnet werden soll
    pub show_file_dialog: bool,
    /// Ob der Save-Datei-Dialog geoeffnet werden soll
    pub show_save_file_dialog: bool,
    /// Ob der Heightmap-Auswahl-Dialog geoeffnet werden soll
    pub show_heightmap_dialog: bool,
    /// Ob der Background-Map-Auswahl-Dialog geoeffnet werden soll
    pub show_background_map_dialog: bool,
    /// Ob der Uebersichtskarten-ZIP-Auswahl-Dialog geoeffnet werden soll
    pub show_overview_dialog: bool,
    /// Ob die Command-Palette angezeigt werden soll
    pub show_command_palette: bool,
    /// Optionales schwebendes Menue an der Mausposition.
    pub floating_menu: Option<FloatingMenuState>,
    /// Ob die Heightmap-Warnung angezeigt werden soll
    pub show_heightmap_warning: bool,
    /// Ob die Heightmap-Warnung fuer diese Save-Operation bereits bestaetigt wurde
    pub heightmap_warning_confirmed: bool,
    /// Pfad fuer Save-Operation nach Heightmap-Warnung
    pub pending_save_path: Option<String>,
    /// Pfad der aktuell geladenen Datei (fuer Save ohne Dialog)
    pub current_file_path: Option<String>,
    /// Pfad der aktuell ausgewaehlten Heightmap (optional)
    pub heightmap_path: Option<String>,
    /// Marker-Bearbeiten-Dialog
    pub marker_dialog: MarkerDialogState,
    /// Temporaere Statusnachricht (z.B. Duplikat-Bereinigung)
    pub status_message: Option<String>,
    /// Duplikat-Bestaetigungsdialog
    pub dedup_dialog: DedupDialogState,
    /// ZIP-Browser-Dialog fuer Background-Map-Auswahl
    pub zip_browser: Option<ZipBrowserState>,
    /// Uebersichtskarten-Optionen-Dialog
    pub overview_options_dialog: OverviewOptionsDialogState,
    /// Post-Load-Dialog (Heightmap/ZIP-Erkennung)
    pub post_load_dialog: PostLoadDialogState,
    /// Dialog fuer "Als overview.png speichern"-Abfrage
    pub save_overview_dialog: SaveOverviewDialogState,
    /// Distanzen-Neuverteilen-Konfiguration (Eigenschaften-Panel)
    pub distanzen: DistanzenState,
    /// Dialog fuer "Alle Felder nachzeichnen"-Einstellungen
    pub trace_all_fields_dialog: TraceAllFieldsDialogState,
    /// Segment-Einstellungs-Popup (erscheint nach Doppelklick auf Segment-Node)
    pub segment_settings_popup: SegmentSettingsPopupState,
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
            show_command_palette: false,
            floating_menu: None,
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
            save_overview_dialog: SaveOverviewDialogState::default(),
            distanzen: DistanzenState::default(),
            trace_all_fields_dialog: TraceAllFieldsDialogState::default(),
            segment_settings_popup: SegmentSettingsPopupState::default(),
        }
    }
}
