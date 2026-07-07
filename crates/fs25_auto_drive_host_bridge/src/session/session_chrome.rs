//! Chrome-/Dialog-Gruppe der `HostBridgeSession`: host-lokale Panel-, Dialog-
//! und Floating-Menu-Seams. Reine interne Aufteilung — die oeffentliche
//! Session-Surface bleibt unveraendert.

use fs25_auto_drive_engine::app::{FloatingMenuKind, FloatingMenuState};
use glam::Vec2;

use super::snapshots::{
    map_host_field_detection_source_to_engine, map_host_overview_layers_to_engine,
};
use super::{
    HostBridgeSession, HostDialogUiState, HostLocalDialogState, HostPanelPropertiesState,
    HostViewportInputContext,
};
use crate::dto::HostOverviewOptionsDialogSnapshot;

impl HostBridgeSession {
    /// Liefert eine read-only Referenz auf den host-lokalen Chrome-/Dialog-Zustand.
    pub fn chrome_state(&self) -> &HostLocalDialogState {
        &self.chrome_state
    }

    /// Liefert eine mutable Referenz auf den host-lokalen Chrome-/Dialog-Zustand.
    ///
    /// Aenderungen ueber diesen Accessor setzen automatisch `chrome_dirty` im
    /// `HostLocalDialogState`. Der Session-Snapshot wird *nicht* automatisch
    /// als dirty markiert — bei Bedarf `mark_snapshot_dirty()` aufrufen.
    pub fn chrome_state_mut(&mut self) -> &mut HostLocalDialogState {
        &mut self.chrome_state
    }

    /// Liefert den schmalen Properties-/Edit-Panel-Zugriff.
    ///
    /// Diese Seams kapseln die verbleibenden host-lokalen UI-Mutationen
    /// (`distanzen`, `options`) bei gleichzeitig read-only Zugriff auf
    /// Selektions-/Gruppen-/Karteninformationen. Der Zugriff bleibt bewusst
    /// Snapshot-transparent, weil diese lokalen Felder nicht Teil des kleinen
    /// `HostSessionSnapshot` sind.
    pub fn panel_properties_state_mut(&mut self) -> HostPanelPropertiesState<'_> {
        let state = &mut self.state;

        HostPanelPropertiesState {
            road_map: state.road_map.as_deref(),
            selected_node_ids: &state.selection.selected_node_ids,
            default_direction: state.editor.default_direction,
            default_priority: state.editor.default_priority,
            active_tool: state.editor.active_tool,
            group_registry: &state.group_registry,
            tool_edit_store: &state.tool_edit_store,
            group_editing: state.group_editing.as_ref(),
            distanzen: &mut state.ui.distanzen,
            options: &mut state.options,
        }
    }

    /// Liefert den schmalen Dialogzugriff fuer host-lokale Modalfenster.
    ///
    /// Das `ui`-Feld zeigt nun auf `chrome_state` (statt `state.ui`), sodass
    /// Dialog-Mutationen durch das Frontend direkt im host-lokalen State landen.
    /// Der Accessor invalidiert den Session-Snapshot nicht automatisch; falls
    /// Snapshot-relevante Felder geaendert werden, muss __mark_snapshot_dirty()__
    /// explizit aufgerufen werden.
    pub fn dialog_ui_state_mut(&mut self) -> HostDialogUiState<'_> {
        HostDialogUiState {
            road_map: self.state.road_map.as_deref(),
            ui: &mut self.chrome_state,
            options: &mut self.state.options,
        }
    }

    /// Aktualisiert den host-lokalen Draft des Overview-Options-Dialogs aus einem DTO-Snapshot.
    ///
    /// Serialisierbare Hosts spiegeln damit lokal editierte Dialogwerte in die
    /// Session zurueck, bevor sie `OverviewOptionsConfirmed` ausloesen. Der
    /// Engine-Dialogzustand bleibt bis zur Bestaetigung unveraendert.
    pub fn update_overview_options_dialog(&mut self, snapshot: HostOverviewOptionsDialogSnapshot) {
        self.chrome_state.overview_options_dialog.visible = snapshot.visible;
        self.chrome_state.overview_options_dialog.zip_path = snapshot.zip_path;
        self.chrome_state.overview_options_dialog.layers =
            map_host_overview_layers_to_engine(&snapshot.layers);
        self.chrome_state
            .overview_options_dialog
            .field_detection_source =
            map_host_field_detection_source_to_engine(snapshot.field_detection_source);
        self.chrome_state.overview_options_dialog.available_sources = snapshot
            .available_sources
            .into_iter()
            .map(map_host_field_detection_source_to_engine)
            .collect();
        self.chrome_state.mark_dirty();
        self.snapshot_dirty = true;
    }

    /// Liefert den schmalen Viewport-Input-Zugriff fuer Host-Event-Sammler.
    ///
    /// Der Zugriff bleibt bewusst Snapshot-transparent, weil der lokale
    /// Distanzzustand nicht im kleinen `HostSessionSnapshot` gespiegelt wird.
    pub fn viewport_input_context_mut(&mut self) -> HostViewportInputContext<'_> {
        let state = &mut self.state;
        let farmland_available = state
            .farmland_polygons_arc()
            .is_some_and(|polygons| !polygons.is_empty());

        HostViewportInputContext {
            paste_preview_active: state.paste_preview_pos.is_some(),
            camera: &state.view.camera,
            road_map: state.road_map.as_deref(),
            selected_node_ids: &state.selection.selected_node_ids,
            active_tool: state.editor.active_tool,
            default_direction: state.editor.default_direction,
            default_priority: state.editor.default_priority,
            options: &state.options,
            clipboard_has_nodes: !state.clipboard.nodes.is_empty(),
            farmland_available,
            group_editing_active: state.group_editing.is_some(),
            group_registry: &state.group_registry,
            distanzen: &mut state.ui.distanzen,
        }
    }

    /// Schliesst das host-lokale Floating-Menue explizit.
    pub fn clear_floating_menu(&mut self) {
        self.chrome_state.floating_menu = None;
        self.chrome_state.mark_dirty();
        self.snapshot_dirty = true;
    }

    /// Schaltet das host-lokale Floating-Menue fuer den angegebenen Menue-Typ um.
    ///
    /// `pointer_pos` beschreibt die aktuelle Pointer-Position in Host-Pixeln.
    /// Ist keine Position verfuegbar, wird bei Aktivierung kein Menue geoeffnet.
    pub fn toggle_floating_menu(&mut self, kind: FloatingMenuKind, pointer_pos: Option<Vec2>) {
        let next_menu = match self.chrome_state.floating_menu {
            Some(existing) if existing.kind == kind => None,
            Some(_) | None => pointer_pos.map(|pos| FloatingMenuState { kind, pos }),
        };

        self.chrome_state.floating_menu = next_menu;
        self.chrome_state.mark_dirty();
        self.snapshot_dirty = true;
    }

    /// Setzt die aktuelle Statusmeldung explizit.
    pub fn set_status_message(&mut self, message: Option<String>) {
        self.state.ui.status_message = message;
        self.snapshot_dirty = true;
    }
}
