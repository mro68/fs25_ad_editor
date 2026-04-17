use super::HostBridgeSession;
use fs25_auto_drive_engine::app::{ui_contract::DialogRequest, AppIntent};

impl HostBridgeSession {
    pub(super) fn reconcile_host_local_dialog_state_for_intent(&mut self, intent: &AppIntent) {
        if matches!(intent, AppIntent::OverviewOptionsConfirmed) {
            self.state.ui.overview_options_dialog =
                self.chrome_state.overview_options_dialog.clone();
        }
    }

    pub(super) fn rebuild_snapshot_if_dirty(&mut self) {
        if !self.snapshot_dirty {
            return;
        }

        self.snapshot_cache = super::snapshots::build_snapshot(
            &self.state,
            &self.chrome_state,
            self.pending_dialog_requests.len(),
        );
        self.snapshot_dirty = false;
    }

    /// Verarbeitet ausstehende Engine-Requests: Chrome-Varianten werden in
    /// `chrome_state` ausgefuehrt, `PickPath`-Varianten in `pending_dialog_requests`
    /// gepuffert (fuer spaeteres `take_dialog_requests()`).
    pub(super) fn drain_engine_requests(&mut self) {
        let requests = self.controller.take_dialog_requests(&mut self.state);
        let mut chrome_dirty = false;
        for req in requests {
            match req {
                DialogRequest::ToggleCommandPalette => {
                    self.chrome_state.show_command_palette =
                        !self.chrome_state.show_command_palette;
                    chrome_dirty = true;
                }
                DialogRequest::OpenOptionsDialog => {
                    self.chrome_state.show_options_dialog = true;
                    chrome_dirty = true;
                }
                DialogRequest::CloseOptionsDialog => {
                    self.chrome_state.show_options_dialog = false;
                    chrome_dirty = true;
                }
                DialogRequest::ShowHeightmapWarning => {
                    self.chrome_state.show_heightmap_warning = true;
                    chrome_dirty = true;
                }
                DialogRequest::DismissHeightmapWarning => {
                    self.chrome_state.show_heightmap_warning = false;
                    chrome_dirty = true;
                }
                DialogRequest::ShowDissolveGroupConfirm(id) => {
                    self.chrome_state.confirm_dissolve_group_id = Some(id);
                    chrome_dirty = true;
                }
                DialogRequest::PickPath {
                    kind,
                    suggested_file_name,
                } => {
                    self.pending_dialog_requests
                        .push(crate::dispatch::map_engine_dialog_request(
                            DialogRequest::PickPath {
                                kind,
                                suggested_file_name,
                            },
                        ));
                }
            }
        }
        if chrome_dirty {
            self.chrome_state.mark_dirty();
            self.snapshot_dirty = true;
        }
    }

    /// Spiegelt Engine-UI-Request-Flags in den host-lokalen Chrome-State.
    ///
    /// Wird nach jedem `apply_action()`/`apply_intent()` aufgerufen, damit
    /// `chrome_state` immer die aktuellen Engine-Werte fuer sichtbarkeits-relevante
    /// Felder enthaelt. Fuer Dialoge mit nutzer-mutierbaren Daten wird ein
    /// Transition-basiertes Sync verwendet: Beim Oeffen werden Daten kopiert,
    /// waehrend der Dialog offen ist wird der `chrome_state` NICHT ueberschrieben.
    pub(super) fn sync_chrome_from_engine(&mut self) {
        let ui = &self.state.ui;

        // show_command_palette, show_options_dialog, show_heightmap_warning und
        // confirm_dissolve_group_id werden nach dem Drain-Refactoring nicht mehr als
        // Flags in EngineUiState gehalten, sondern als DialogRequest-Events emittiert
        // und bereits in drain_engine_requests() direkt in chrome_state verarbeitet.
        let new_hwconf = ui.heightmap_warning_confirmed;

        let mut dirty = false;

        if self.chrome_state.heightmap_warning_confirmed != new_hwconf {
            self.chrome_state.heightmap_warning_confirmed = new_hwconf;
            dirty = true;
        }

        // Dedup-Dialog: read-only im Frontend -> immer spiegeln
        if self.chrome_state.dedup_dialog.visible != ui.dedup_dialog.visible
            || self.chrome_state.dedup_dialog.duplicate_count != ui.dedup_dialog.duplicate_count
            || self.chrome_state.dedup_dialog.group_count != ui.dedup_dialog.group_count
        {
            self.chrome_state.dedup_dialog = ui.dedup_dialog.clone();
            dirty = true;
        }

        // Save-Overview-Dialog: kein mutierbares Nutzerfeld -> immer spiegeln
        if self.chrome_state.save_overview_dialog.visible != ui.save_overview_dialog.visible {
            self.chrome_state.save_overview_dialog = ui.save_overview_dialog.clone();
            dirty = true;
        }

        // Group-Settings-Popup: einfacher Trigger - beim Oeffen/Schliessen spiegeln
        if self.chrome_state.group_settings_popup.visible != ui.group_settings_popup.visible {
            self.chrome_state.group_settings_popup = ui.group_settings_popup.clone();
            dirty = true;
        }

        // Dialoge mit Nutzer-mutierbaren Feldern: nur beim Oeffen (Transition false->true)
        // kopieren; waehrend offen NICHT ueberschreiben.
        if ui.marker_dialog.visible && !self.chrome_state.marker_dialog.visible {
            self.chrome_state.marker_dialog = ui.marker_dialog.clone();
            dirty = true;
        } else if !ui.marker_dialog.visible && self.chrome_state.marker_dialog.visible {
            self.chrome_state.marker_dialog.visible = false;
            dirty = true;
        }

        if ui.trace_all_fields_dialog.visible && !self.chrome_state.trace_all_fields_dialog.visible
        {
            self.chrome_state.trace_all_fields_dialog = ui.trace_all_fields_dialog.clone();
            dirty = true;
        } else if !ui.trace_all_fields_dialog.visible
            && self.chrome_state.trace_all_fields_dialog.visible
        {
            self.chrome_state.trace_all_fields_dialog.visible = false;
            dirty = true;
        }

        if ui.overview_options_dialog.visible && !self.chrome_state.overview_options_dialog.visible
        {
            self.chrome_state.overview_options_dialog = ui.overview_options_dialog.clone();
            dirty = true;
        } else if !ui.overview_options_dialog.visible
            && self.chrome_state.overview_options_dialog.visible
        {
            self.chrome_state.overview_options_dialog.visible = false;
            dirty = true;
        }

        if ui.post_load_dialog.visible && !self.chrome_state.post_load_dialog.visible {
            self.chrome_state.post_load_dialog = ui.post_load_dialog.clone();
            dirty = true;
        } else if !ui.post_load_dialog.visible && self.chrome_state.post_load_dialog.visible {
            self.chrome_state.post_load_dialog.visible = false;
            dirty = true;
        }

        // ZIP-Browser: Option<ZipBrowserState> - beim Oeffnen kopieren
        let engine_zip_open = ui.zip_browser.is_some();
        let chrome_zip_open = self.chrome_state.zip_browser.is_some();
        if engine_zip_open && !chrome_zip_open {
            self.chrome_state.zip_browser = ui.zip_browser.clone();
            dirty = true;
        } else if !engine_zip_open && chrome_zip_open {
            self.chrome_state.zip_browser = None;
            dirty = true;
        }

        if dirty {
            self.chrome_state.mark_dirty();
        }
    }
}
