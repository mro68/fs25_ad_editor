//! InputState und zugehoerige Typen fuer das Viewport-Input-Handling.

use super::context_menu;
use super::DragSelection;

/// Modus des primaeren (Links-)Drags im Viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PrimaryDragMode {
    #[default]
    None,
    /// Drag eines Route-Tool-Steuerpunkts (Anker/CP)
    RouteToolPointDrag,
}

/// Immutable Snapshot des Kontextmenue-States beim Rechtsklick.
///
/// Dieser Snapshot gefriert den kompletten Zustand ein, der zum Zeitpunkt
/// des Rechtsklicks galt — damit Menueinhalt stabil bleibt, solange das
/// Popup offen ist. Zustandsaenderungen (Escape, Deselection etc.) beeinflussen
/// nicht das bereits offene Menue.
#[derive(Debug, Clone)]
pub(crate) struct ContextMenuSnapshot {
    /// Eingefrorene Menu-Variante
    pub(crate) variant: context_menu::MenuVariant,
    /// Eingefrorene Selection-Menge (geklonter Arc = O(1))
    pub(crate) selection: indexmap::IndexSet<u64>,
    /// Bildschirmposition des Rechtsklicks (fuer Panel-Positionierung)
    pub(crate) screen_pos: Option<egui::Pos2>,
}

/// Verwaltet den Input-Zustand fuer das Viewport (Drag, Selektion, Scroll)
#[derive(Default)]
pub struct InputState {
    pub(crate) primary_drag_mode: PrimaryDragMode,
    /// Gibt an, ob der aktuelle Primaer-Drag ueber die Bridge-Seam laeuft.
    pub(crate) primary_drag_via_bridge: bool,
    pub(crate) drag_selection: Option<DragSelection>,
    /// Snapshot des Menue-Zustands, gueltig solange das Popup offen ist.
    /// Wird beim Rechtsklick gesetzt und erst geleert, wenn egui das Popup schliesst.
    pub(crate) context_menu_snapshot: Option<ContextMenuSnapshot>,
    /// Bildschirmposition des letzten CM-Klicks fuer Edit-Panel-Positionierung.
    pub edit_panel_pos: Option<[f32; 2]>,
    /// Zeigt an, ob gerade eine Gruppen-Rotation per Alt+Mausrad laeuft.
    /// Wird benutzt um Begin/End-Lifecycle Intents korrekt zu emittieren.
    pub(crate) rotation_active: bool,
    /// Unterdrueckt egui-Smoothing-Folgeframes nach einem diskreten Wheel-Notch,
    /// damit ein physischer Raster-Schritt genau einen Zoomschritt ausloest.
    pub(crate) suppress_smoothed_scroll_zoom: bool,
}

impl InputState {
    /// Erstellt einen neuen, leeren Input-Zustand.
    pub fn new() -> Self {
        Self {
            primary_drag_mode: PrimaryDragMode::None,
            primary_drag_via_bridge: false,
            drag_selection: None,
            context_menu_snapshot: None,
            edit_panel_pos: None,
            rotation_active: false,
            suppress_smoothed_scroll_zoom: false,
        }
    }
}
