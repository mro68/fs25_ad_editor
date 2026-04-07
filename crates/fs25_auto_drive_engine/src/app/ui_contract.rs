//! App-weite egui-freie Read-Vertraege fuer Route-Tool-Daten.

mod host_ui;
mod route_tool_panel;
mod viewport_overlay;

use crate::app::tool_contract::TangentSource;
use glam::Vec2;

pub use host_ui::{
    dialog_result_to_intent, panel_action_to_intent, CommandPalettePanelState, DialogRequest,
    DialogRequestKind, DialogResult, HostUiSnapshot, OptionsPanelAction, OptionsPanelState,
    PanelAction, PanelState,
};
pub use route_tool_panel::{
    BypassPanelAction, BypassPanelState, ColorPathPanelAction, ColorPathPanelPhase,
    ColorPathPanelState, ColorPathPreviewStats, CurveDegreeChoice, CurvePanelAction,
    CurvePanelState, CurveTangentsPanelState, ExistingConnectionModeChoice,
    FieldBoundaryPanelAction, FieldBoundaryPanelState, FieldPathModeChoice, FieldPathPanelAction,
    FieldPathPanelPhase, FieldPathPanelState, FieldPathPreviewStatus, FieldPathSelectionSummary,
    ParkingPanelAction, ParkingPanelState, ParkingRampSideChoice, RouteOffsetPanelAction,
    RouteOffsetPanelState, RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect,
    RouteToolPanelFollowUp, RouteToolPanelState, SegmentConfigPanelAction, SegmentConfigPanelState,
    SegmentLengthKind, SegmentPanelMode, SmoothCurvePanelAction, SmoothCurvePanelState,
    SmoothCurveSteererState, SplinePanelAction, SplinePanelState, StraightPanelAction,
    StraightPanelState, TangentHelpHint, TangentNoneReason, TangentSelectionState,
};
pub use route_tool_panel::{
    BYPASS_BASE_SPACING_LIMITS, BYPASS_OFFSET_LIMITS, PARKING_BAY_LENGTH_LIMITS,
    PARKING_ENTRY_EXIT_T_LIMITS, PARKING_MAX_NODE_DISTANCE_LIMITS, PARKING_NUM_ROWS_LIMITS,
    PARKING_RAMP_LENGTH_LIMITS, PARKING_ROTATION_STEP_LIMITS, PARKING_ROW_SPACING_LIMITS,
    ROUTE_OFFSET_BASE_SPACING_LIMITS, ROUTE_OFFSET_DISTANCE_LIMITS, SMOOTH_CURVE_MAX_ANGLE_LIMITS,
    SMOOTH_CURVE_MIN_DISTANCE_LIMITS,
};
pub use viewport_overlay::{
    ClipboardOverlaySnapshot, ClipboardPreviewNode, GroupBoundaryOverlaySnapshot,
    GroupLockOverlaySnapshot, PolylineOverlaySnapshot, ViewportOverlaySnapshot,
};

/// Eine waehlbare Tangenten-Option mit bereits aufbereitetem UI-Label.
#[derive(Debug, Clone, PartialEq)]
pub struct TangentOptionData {
    /// Semantische Quelle der Tangente.
    pub source: TangentSource,
    /// Fertig formatierter UI-Text fuer Menues und Listen.
    pub label: String,
}

/// Reine Menue-Daten fuer die Tangenten-Auswahl eines Route-Tools.
///
/// Enthalten sind nur read-only DTOs und primitive Werte, damit die UI keine
/// Tool-Interna oder `egui`-nahe Zustandsobjekte kennen muss.
#[derive(Debug, Clone, PartialEq)]
pub struct TangentMenuData {
    /// Aufbereitete Optionen fuer die Start-Tangente.
    pub start_options: Vec<TangentOptionData>,
    /// Aufbereitete Optionen fuer die End-Tangente.
    pub end_options: Vec<TangentOptionData>,
    /// Aktuell gewaehlte Start-Tangente.
    pub current_start: TangentSource,
    /// Aktuell gewaehlte End-Tangente.
    pub current_end: TangentSource,
}

/// Read-DTO fuer Route-Tool-spezifische Viewport-Eingaben.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RouteToolViewportData {
    /// Drag-Ziele des aktiven Tools fuer Hit-Tests im Viewport.
    pub drag_targets: Vec<Vec2>,
    /// Gibt an, ob das Tool bereits angefangene Eingaben besitzt.
    pub has_pending_input: bool,
    /// Gibt an, ob Pfeiltasten aktuell als Segment-Shortcuts geroutet werden.
    pub segment_shortcuts_active: bool,
    /// Optional vorbereitete Tangenten-Daten fuer das Kontextmenue.
    pub tangent_menu_data: Option<TangentMenuData>,
    /// Gibt an, ob Alt+Drag als Tool-Lasso statt als Selektion geroutet werden muss.
    pub needs_lasso_input: bool,
}
