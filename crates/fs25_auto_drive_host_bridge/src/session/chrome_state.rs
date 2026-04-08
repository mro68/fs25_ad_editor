//! Chrome- und Dialog-Sichtbarkeitszustand fuer den Host.
//!
//! Dieser Modul trennt rein UI-lokale Sichtbarkeitsflags aus dem Engine-`UiState`
//! heraus. Die `HostBridgeSession` haelt `HostLocalDialogState` exklusiv und
//! spiegelt eingehende "Request"-Flags aus dem Engine-`EngineUiState` per Drain.

use fs25_auto_drive_engine::app::{
    DedupDialogState, FloatingMenuState, GroupSettingsPopupState, MarkerDialogState,
    OverviewOptionsDialogState, PostLoadDialogState, SaveOverviewDialogState,
    TraceAllFieldsDialogState, ZipBrowserState,
};

/// Host-lokaler Chrome- und Dialog-Sichtbarkeitszustand.
///
/// Enthaelt alle Sichtbarkeitsflags und transienten UI-Zustaende, die nicht
/// zur fachlichen Engine-Logik gehoeren. Wird von `HostBridgeSession` gehalten
/// und ueber `chrome_state()`/`chrome_state_mut()` zugaenglich gemacht.
///
/// Aenderungen an diesem Zustand werden ueber das `chrome_dirty`-Flag
/// signalisiert, damit Hosts wissen ob ein Repaint noetig ist.
#[derive(Default)]
pub struct HostLocalDialogState {
    /// Ob die Command-Palette angezeigt wird.
    pub show_command_palette: bool,
    /// Ob der Optionen-Dialog sichtbar ist.
    pub show_options_dialog: bool,
    /// Optionales schwebendes Kontextmenue.
    pub floating_menu: Option<FloatingMenuState>,
    /// Ob die Heightmap-Warnung sichtbar ist.
    pub show_heightmap_warning: bool,
    /// Ob die Heightmap-Warnung fuer diese Save-Operation bestaetigt wurde.
    pub heightmap_warning_confirmed: bool,
    /// Marker-Bearbeiten-Dialog.
    pub marker_dialog: MarkerDialogState,
    /// Duplikat-Bestaetigungsdialog.
    pub dedup_dialog: DedupDialogState,
    /// ZIP-Browser-Dialog fuer Background-Map-Auswahl.
    pub zip_browser: Option<ZipBrowserState>,
    /// Uebersichtskarten-Optionen-Dialog.
    pub overview_options_dialog: OverviewOptionsDialogState,
    /// Post-Load-Dialog (Heightmap/ZIP-Erkennung).
    pub post_load_dialog: PostLoadDialogState,
    /// Dialog fuer "Als overview.png speichern"-Abfrage.
    pub save_overview_dialog: SaveOverviewDialogState,
    /// Dialog fuer "Alle Felder nachzeichnen"-Einstellungen.
    pub trace_all_fields_dialog: TraceAllFieldsDialogState,
    /// Segment-Einstellungs-Popup (erscheint nach Doppelklick).
    pub group_settings_popup: GroupSettingsPopupState,
    /// Bestaetigungsdialog zum Aufloesen einer Gruppe.
    pub confirm_dissolve_group_id: Option<u64>,
    /// Internes Dirty-Flag: wird gesetzt wenn sich Chrome-Zustand geaendert hat.
    pub chrome_dirty: bool,
}

impl HostLocalDialogState {
    /// Erstellt einen geschlossenen Chrome-Zustand (alle Dialoge unsichtbar).
    pub fn new() -> Self {
        Self::default()
    }

    /// Markiert den Chrome-Zustand als geaendert.
    pub fn mark_dirty(&mut self) {
        self.chrome_dirty = true;
    }

    /// Konsumiert das Dirty-Flag und gibt zurueck ob es gesetzt war.
    pub fn take_dirty(&mut self) -> bool {
        let dirty = self.chrome_dirty;
        self.chrome_dirty = false;
        dirty
    }
}
