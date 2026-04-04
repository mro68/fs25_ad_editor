use crate::app::group_registry::GroupRegistry;
use crate::app::history::Snapshot;
use crate::app::tool_contract::RouteToolId;
use crate::app::tool_editing::{ActiveToolEditSession, ToolEditStore};
use crate::app::CommandLog;
use crate::core::{Connection, FarmlandGrid, FieldPolygon, MapMarker, MapNode, RoadMap};
use crate::shared::{EditorOptions, RenderMap};
use glam::Vec2;
use indexmap::IndexSet;
use std::cell::RefCell;
use std::sync::Arc;

use super::{EditorTool, EditorToolState, SelectionState, UiState, ViewState};

/// Zwischenablage fuer Nodes, Verbindungen und Marker
#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    /// Kopierte Nodes
    pub nodes: Vec<MapNode>,
    /// Kopierte Verbindungen (nur intern: beide Endpunkte in der Selektion)
    pub connections: Vec<Connection>,
    /// Kopierte Marker (nur fuer selektierte Nodes)
    pub markers: Vec<MapMarker>,
    /// Geometrisches Zentrum der Kopie (fuer relativen Offset beim Paste)
    pub center: Vec2,
}

/// Zustand einer aktiven Gruppen-Bearbeitung.
///
/// Wird in `AppState::group_editing` gespeichert. `None` = Normal-Modus.
#[derive(Debug, Clone)]
pub struct GroupEditState {
    /// Record-ID der bearbeiteten Gruppe
    pub record_id: u64,
    /// Lock-Zustand vor dem Edit (wird bei Apply/Cancel wiederhergestellt)
    pub was_locked: bool,
}

/// Cache-Eintrag fuer `compute_dimmed_ids`.
///
/// Tuple: `(selection_generation, registry_dimmed_generation, gecachtes_Ergebnis)`.
type DimmedIdsCache = Option<(u64, u64, Arc<IndexSet<u64>>)>;

/// Cache-Eintrag fuer den render-seitigen Map-Snapshot.
///
/// Tuple: `(render_instance_id, render_revision, gecachter_RenderMap_Snapshot)`.
type RenderMapCache = Option<(u64, u64, Arc<RenderMap>)>;

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
    /// Zwischenablage
    pub clipboard: Clipboard,
    /// Aktuelle Vorschau-Position beim Einfuegen
    pub paste_preview_pos: Option<Vec2>,
    /// Verlauf ausgefuehrter Commands
    pub command_log: CommandLog,
    /// Undo/Redo-History (Snapshot-basiert)
    pub history: crate::app::history::EditHistory,
    /// Laufzeit-Optionen (Farben, Groessen, Breiten)
    pub options: EditorOptions,
    /// Geteilte Arc-Variante der Optionen fuer RenderScene-Build (O(1) Clone pro Frame)
    options_arc: Arc<EditorOptions>,
    /// Ob der Options-Dialog angezeigt wird
    pub show_options_dialog: bool,
    /// In-Session-Registry aller erstellten Segmente (fuer nachtraegliche Bearbeitung)
    pub group_registry: GroupRegistry,
    /// Signalisiert dem Host (eframe), die Anwendung kontrolliert zu beenden
    pub should_exit: bool,
    /// Beim letzten Overview-Laden extrahierte Farmland-Feldgrenz-Polygone
    ///
    /// Enthält geordnete Umriss-Vertices pro Feld in Weltkoordinaten.
    /// `None` solange noch keine Overview mit Farmland-Daten geladen wurde.
    pub farmland_polygons: Option<Arc<Vec<FieldPolygon>>>,
    /// GRLE-Raster mit Farmland-IDs fuer Pixel-basierte Analysen (z.B. Feldweg-Erkennung).
    /// `None` solange kein Overview mit GRLE-Daten geladen wurde.
    pub farmland_grid: Option<Arc<FarmlandGrid>>,
    /// Gecachtes Hintergrundbild fuer farbbasierte Tool-Analysen.
    /// `None` solange kein Overview geladen wurde.
    pub background_image: Option<Arc<image::DynamicImage>>,
    /// Aktive Gruppen-Bearbeitung (None = Normal-Modus, Some = Edit-Modus aktiv)
    pub group_editing: Option<GroupEditState>,
    /// Separater Store fuer tool-spezifische Edit-Payloads.
    pub tool_edit_store: ToolEditStore,
    /// Aktive Tool-Edit-Session mit Backups fuer Cancel/Wiederherstellung.
    pub active_tool_edit_session: Option<ActiveToolEditSession>,
    /// Lazy Cache fuer `compute_dimmed_ids`.
    ///
    /// Tuple: `(selection_generation, registry_dimmed_generation, gecachtes_Ergebnis)`.
    /// Interior Mutability via `RefCell`, da `render_scene::build()` nur `&AppState` erhaelt.
    /// Wird invalidiert wenn sich selection oder group_registry aendern (Generations-Vergleich).
    pub(crate) dimmed_ids_cache: RefCell<DimmedIdsCache>,
    /// Lazy Cache fuer den render-seitigen Map-Snapshot.
    ///
    /// Wird ueber `(render_instance_id, render_revision)` invalidiert, damit der
    /// Snapshot nur bei render-relevanten RoadMap-Aenderungen neu aufgebaut wird.
    pub(crate) render_map_cache: RefCell<RenderMapCache>,
}

impl AppState {
    /// Erstellt einen neuen, leeren App-State
    pub fn new() -> Self {
        let options = EditorOptions::default();
        let options_arc = Arc::new(options.clone());

        Self {
            road_map: None,
            view: ViewState::new(),
            ui: UiState::new(),
            selection: SelectionState::new(),
            editor: EditorToolState::new(),
            clipboard: Clipboard::default(),
            paste_preview_pos: None,
            command_log: CommandLog::new(),
            history: crate::app::history::EditHistory::new_with_capacity(200),
            options,
            options_arc,
            show_options_dialog: false,
            group_registry: GroupRegistry::new(),
            should_exit: false,
            farmland_polygons: None,
            farmland_grid: None,
            background_image: None,
            group_editing: None,
            tool_edit_store: ToolEditStore::new(),
            active_tool_edit_session: None,
            dimmed_ids_cache: RefCell::new(None),
            render_map_cache: RefCell::new(None),
        }
    }

    /// Gibt eine Referenz auf die aktuelle RoadMap zurück, falls eine Karte geladen ist.
    ///
    /// Bevorzugtes Pattern gegen `state.road_map.as_ref().unwrap()` in Use-Cases.
    pub fn road_map_ref(&self) -> Option<&RoadMap> {
        self.road_map.as_deref()
    }

    /// Gibt geladene Farmland-Polygone als Arc-Clone zurueck.
    pub fn farmland_polygons_arc(&self) -> Option<Arc<Vec<FieldPolygon>>> {
        self.farmland_polygons.clone()
    }

    /// Gibt das geladene Farmland-Raster als Arc-Clone zurueck.
    pub fn farmland_grid_arc(&self) -> Option<Arc<FarmlandGrid>> {
        self.farmland_grid.clone()
    }

    /// Gibt das aktuell kanonische Hintergrundbild als Arc-Clone zurueck.
    ///
    /// Primäre Quelle ist die geladene `BackgroundMap`. Das Feld
    /// `background_image` dient als kompatibler Fallback fuer Legacy-Pfade.
    pub fn background_image_arc(&self) -> Option<Arc<image::DynamicImage>> {
        self.view
            .background_map
            .as_deref()
            .map(|background| background.image_arc())
            .or_else(|| self.background_image.clone())
    }

    /// Gibt `true` zurueck, wenn Farmland-Polygone geladen sind.
    pub fn has_farmland_polygons(&self) -> bool {
        self.farmland_polygons.is_some()
    }

    /// Gibt `true` zurueck, wenn ein Hintergrundbild geladen ist.
    pub fn has_background_image(&self) -> bool {
        self.background_image_arc().is_some()
    }

    /// Gibt die Anzahl der Nodes zurueck (fuer UI-Anzeige)
    pub fn node_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.node_count())
    }

    /// Gibt die Anzahl der Connections zurueck (fuer UI-Anzeige)
    pub fn connection_count(&self) -> usize {
        self.road_map.as_ref().map_or(0, |rm| rm.connection_count())
    }

    /// Liefert die ID des aktiven Route-Tools, wenn der Editor im Route-Modus ist.
    pub fn active_route_tool_id(&self) -> Option<RouteToolId> {
        if self.editor.active_tool == EditorTool::Route {
            self.editor.tool_manager.active_id()
        } else {
            None
        }
    }

    /// Undo/Redo helpers
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Gibt zurueck, ob ein Redo-Schritt verfuegbar ist.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Erstellt einen Undo-Snapshot des aktuellen Zustands.
    /// Reduziert Boilerplate in mutierenden Use-Cases.
    pub fn record_undo_snapshot(&mut self) {
        let snap = Snapshot::from_state(self);
        self.history.record_snapshot(snap);
    }

    /// Liefert die Arc-Variante der Optionen (fuer RenderScene-Build, zero-copy pro Frame).
    pub fn options_arc(&self) -> Arc<EditorOptions> {
        self.options_arc.clone()
    }

    /// Setzt neue Optionen und aktualisiert den geteilten Arc.
    pub fn set_options(&mut self, options: EditorOptions) {
        self.options = options;
        self.options_arc = Arc::new(self.options.clone());
    }

    /// Aktualisiert den geteilten Arc nach in-place Mutationen der Optionen.
    pub fn refresh_options_arc(&mut self) {
        self.options_arc = Arc::new(self.options.clone());
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
