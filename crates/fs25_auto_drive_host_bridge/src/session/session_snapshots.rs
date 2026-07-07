//! Snapshot-Gruppe der `HostBridgeSession`: alle `build_*`/`snapshot*`-Methoden
//! fuer Render- und UI-Vertraege sowie die temporaere `app_state()`-Read-Seam.
//! Reine interne Aufteilung — die oeffentliche Session-Surface bleibt unveraendert.

use fs25_auto_drive_engine::app::projections as engine_projections;
use fs25_auto_drive_engine::app::ui_contract::{
    HostUiSnapshot, PanelState, ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::AppState;
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use glam::Vec2;

use super::context_menu;
use super::snapshots::{build_dialog_snapshot, build_editing_snapshot};
use super::{HostBridgeSession, HostRenderFrameSnapshot};
use crate::dto::{
    HostChromeSnapshot, HostContextMenuSnapshot, HostDialogSnapshot, HostEditingSnapshot,
    HostRouteToolViewportSnapshot, HostSessionSnapshot, HostViewportGeometrySnapshot,
};

impl HostBridgeSession {
    /// Liefert eine read-only Referenz auf den aktuellen `AppState`.
    ///
    /// Diese API ist als temporaere Read-Seam fuer den Ownership-Flip gedacht,
    /// bis alle host-neutralen Snapshots konsumiert werden.
    pub fn app_state(&self) -> &AppState {
        &self.state
    }

    /// Liefert einen referenzierten Snapshot fuer Polling-Hosts.
    ///
    /// Der Snapshot wird nur nach einer erfolgreichen Session-Mutation neu
    /// aufgebaut, damit bei Polling ohne State-Aenderung keine neuen Allokationen
    /// entstehen.
    pub fn snapshot(&mut self) -> &HostSessionSnapshot {
        self.rebuild_snapshot_if_dirty();
        &self.snapshot_cache
    }

    /// Liefert eine besitzende Snapshot-Kopie.
    ///
    /// Diese Methode ist fuer Call-Sites gedacht, die den Snapshot vom Session-
    /// Lebenszyklus entkoppeln muessen.
    pub fn snapshot_owned(&mut self) -> HostSessionSnapshot {
        self.snapshot().clone()
    }

    /// Liefert einen host-neutralen Snapshot aller egui-Dialogzustaende.
    ///
    /// Der Snapshot liest sowohl den host-lokalen `chrome_state` als auch die
    /// fuer Dialog-Popups relevanten Engine-Optionen. Er ist bewusst von
    /// `HostSessionSnapshot` getrennt, damit Flutter und spaetere Hosts die
    /// komplexeren Dialog-Drafts als eigene serialisierbare Surface pollen
    /// koennen, ohne auf `dialog_ui_state_mut()` oder `chrome_state()`
    /// angewiesen zu sein.
    pub fn dialog_snapshot(&self) -> HostDialogSnapshot {
        build_dialog_snapshot(&self.state, &self.chrome_state)
    }

    /// Liefert einen serialisierbaren Snapshot fuer Properties-, Group-Edit- und Resample-Daten.
    ///
    /// Der Snapshot bildet die aktuell ueber `panel_properties_state_mut()` und
    /// `viewport_input_context_mut()` gelesenen Editing-Zustaende host-neutral ab,
    /// damit Flutter und spaetere Hosts dieselben Daten ohne Rust-spezifische
    /// Escape-Hatches pollen koennen.
    pub fn editing_snapshot(&self) -> HostEditingSnapshot {
        build_editing_snapshot(&self.state)
    }

    /// Liefert einen serialisierbaren Snapshot des aktuell relevanten Kontextmenues.
    ///
    /// Die Bridge spiegelt damit die egui-Preconditions fuer Kontextmenue-Aktionen
    /// host-neutral in einer flachen Aktionsliste. `focus_node_id` entspricht dem
    /// vom Host bereits ermittelten fokussierten Node; `None` bedeutet Klick in den
    /// leeren Bereich.
    pub fn context_menu_snapshot(&self, focus_node_id: Option<u64>) -> HostContextMenuSnapshot {
        context_menu::build_context_menu_snapshot(&self.state, focus_node_id)
    }

    /// Baut den aktuellen per-frame Render-Vertrag fuer den angegebenen Viewport.
    pub fn build_render_scene(&self, viewport_size: [f32; 2]) -> RenderScene {
        engine_projections::build_render_scene(&self.state, viewport_size)
    }

    /// Baut den aktuellen Render-Asset-Snapshot.
    pub fn build_render_assets(&self) -> RenderAssetsSnapshot {
        engine_projections::build_render_assets(&self.state)
    }

    /// Baut einen gekoppelten Render-Snapshot aus Szene und Assets.
    ///
    /// Diese Hilfsmethode ist fuer Hosts gedacht, die pro Tick genau einen
    /// read-only Render-Output benoetigen und Szene/Assets nicht separat pollen
    /// wollen.
    pub fn build_render_frame(&self, viewport_size: [f32; 2]) -> HostRenderFrameSnapshot {
        HostRenderFrameSnapshot {
            scene: self.build_render_scene(viewport_size),
            assets: self.build_render_assets(),
        }
    }

    /// Baut einen minimalen, serialisierbaren Viewport-Geometry-Snapshot.
    pub fn build_viewport_geometry_snapshot(
        &self,
        viewport_size: [f32; 2],
    ) -> HostViewportGeometrySnapshot {
        crate::dispatch::build_viewport_geometry_snapshot(&self.state, viewport_size)
    }

    /// Baut den host-neutralen Host-UI-Snapshot fuer sichtbare Panels.
    ///
    /// Host-native Datei- und Pfaddialoge laufen bewusst nicht ueber diesen
    /// Snapshot, sondern separat ueber `take_dialog_requests()`.
    /// Die Panel-Sichtbarkeit (`show_command_palette`, `show_options_dialog`)
    /// stammt aus dem `chrome_state` und wird hier eingefuegt.
    pub fn build_host_ui_snapshot(&self) -> HostUiSnapshot {
        let mut snapshot = engine_projections::build_host_ui_snapshot(&self.state);
        for panel in &mut snapshot.panels {
            match panel {
                PanelState::CommandPalette(state) => {
                    state.visible = self.chrome_state.show_command_palette;
                }
                PanelState::Options(state) => {
                    state.visible = self.chrome_state.show_options_dialog;
                }
                _ => {}
            }
        }
        snapshot
    }

    /// Baut den host-neutralen Chrome-Snapshot fuer Menues, Defaults und Status.
    ///
    /// Die Felder `show_command_palette` und `show_options_dialog` stammen aus
    /// `chrome_state`, das per `drain_engine_requests()` nach jedem Engine-Intent
    /// aktualisiert wird.
    pub fn build_host_chrome_snapshot(&self) -> HostChromeSnapshot {
        let mut snapshot = crate::dispatch::build_host_chrome_snapshot(&self.state);
        snapshot.show_command_palette = self.chrome_state.show_command_palette;
        snapshot.show_options_dialog = self.chrome_state.show_options_dialog;
        snapshot
    }

    /// Baut den host-neutralen Route-Tool-Viewport-Snapshot.
    pub fn build_route_tool_viewport_snapshot(&self) -> HostRouteToolViewportSnapshot {
        crate::dispatch::build_route_tool_viewport_snapshot(&self.state)
    }

    /// Baut den host-neutralen Overlay-Snapshot fuer den aktuellen Viewport.
    ///
    /// Die Methode benoetigt mutablen Zugriff, weil der App-Layer beim Aufbau
    /// bei Bedarf Overlay- und Boundary-Caches im `AppState` aufwaermt.
    pub fn build_viewport_overlay_snapshot(
        &mut self,
        cursor_world: Option<Vec2>,
    ) -> ViewportOverlaySnapshot {
        engine_projections::build_viewport_overlay_snapshot(&mut self.state, cursor_world)
    }
}
