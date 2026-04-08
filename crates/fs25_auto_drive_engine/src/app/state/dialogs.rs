use crate::app::ui_contract::DialogRequest;
use crate::shared::{
    DedupDialogState, DistanzenState, FloatingMenuState, GroupSettingsPopupState,
    MarkerDialogState, OverviewOptionsDialogState, PostLoadDialogState, SaveOverviewDialogState,
    TraceAllFieldsDialogState,
};

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

/// UI-bezogener Anwendungszustand der Engine.
///
/// Enthält fachliche Felder (Dialog-Queues, Dateipfade, Status, Workflow-Flags)
/// sowie Sichtbarkeits-Requests für Chrome-Dialoge (werden von der HostBridge
/// per Drain in `HostLocalDialogState` gespiegelt).
#[derive(Default)]
pub struct EngineUiState {
    /// Ausstehende host-native Dialog-Anforderungen (Datei-/Pfad-Dialoge).
    pub dialog_requests: Vec<DialogRequest>,
    /// Ob die Command-Palette angezeigt werden soll
    pub show_command_palette: bool,
    /// Ob der Optionen-Dialog angezeigt wird.
    pub show_options_dialog: bool,
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
    pub group_settings_popup: GroupSettingsPopupState,
    /// Bestaetigungsdialog zum Aufloesen einer Gruppe.
    pub confirm_dissolve_group_id: Option<u64>,
}

impl EngineUiState {
    /// Erstellt den Standard-UI-Zustand (alle Dialoge geschlossen).
    pub fn new() -> Self {
        Self {
            dialog_requests: Vec::new(),
            show_command_palette: false,
            show_options_dialog: false,
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
            group_settings_popup: GroupSettingsPopupState::default(),
            confirm_dissolve_group_id: None,
        }
    }

    /// Wartet eine neue host-native Dialog-Anforderung ein.
    pub fn request_dialog(&mut self, request: DialogRequest) {
        self.dialog_requests.push(request);
    }

    /// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen.
    pub fn take_dialog_requests(&mut self) -> Vec<DialogRequest> {
        std::mem::take(&mut self.dialog_requests)
    }
}
