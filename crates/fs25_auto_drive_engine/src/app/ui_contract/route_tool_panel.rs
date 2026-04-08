//! Egui-freier Panel-Vertrag fuer das Floating-Route-Tool-Panel.

mod analysis_family;
mod common;
mod curve_family;
mod generator_family;
mod limits;
pub use analysis_family::{
    ColorPathPanelAction, ColorPathPanelPhase, ColorPathPanelState, ColorPathPreviewStats,
    ExistingConnectionModeChoice, FieldBoundaryPanelAction, FieldBoundaryPanelState,
    FieldPathModeChoice, FieldPathPanelAction, FieldPathPanelPhase, FieldPathPanelState,
    FieldPathPreviewStatus, FieldPathSelectionSummary, RouteOffsetPanelAction,
    RouteOffsetPanelState,
};
pub use common::{
    RouteToolPanelEffect, RouteToolPanelFollowUp, RouteToolPanelState, SegmentConfigPanelAction,
    SegmentConfigPanelState, SegmentLengthKind, SegmentPanelMode, TangentNoneReason,
    TangentSelectionState,
};
pub use curve_family::{
    CurveDegreeChoice, CurvePanelAction, CurvePanelState, CurveTangentsPanelState,
    SplinePanelAction, SplinePanelState, TangentHelpHint,
};
pub use generator_family::{
    BypassPanelAction, BypassPanelState, ParkingPanelAction, ParkingPanelState,
    ParkingRampSideChoice, SmoothCurvePanelAction, SmoothCurvePanelState, SmoothCurveSteererState,
    StraightPanelAction, StraightPanelState,
};
pub use limits::{
    BYPASS_BASE_SPACING_LIMITS, BYPASS_OFFSET_LIMITS, PARKING_BAY_LENGTH_LIMITS,
    PARKING_ENTRY_EXIT_T_LIMITS, PARKING_MAX_NODE_DISTANCE_LIMITS, PARKING_NUM_ROWS_LIMITS,
    PARKING_RAMP_LENGTH_LIMITS, PARKING_ROTATION_STEP_LIMITS, PARKING_ROW_SPACING_LIMITS,
    ROUTE_OFFSET_BASE_SPACING_LIMITS, ROUTE_OFFSET_DISTANCE_LIMITS, SMOOTH_CURVE_MAX_ANGLE_LIMITS,
    SMOOTH_CURVE_MIN_DISTANCE_LIMITS,
};
use serde::{Deserialize, Serialize};

/// Tool-spezifischer Read-Zustand fuer das Panel.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteToolConfigState {
    /// Panelzustand fuer das Gerade-Strecke-Tool.
    Straight(StraightPanelState),
    /// Panelzustand fuer das Bézier-Kurven-Tool.
    Curve(CurvePanelState),
    /// Panelzustand fuer das Catmull-Rom-Spline-Tool.
    Spline(SplinePanelState),
    /// Panelzustand fuer das Geglaettete-Kurve-Tool.
    SmoothCurve(SmoothCurvePanelState),
    /// Panelzustand fuer das Ausweichstrecken-Tool.
    Bypass(BypassPanelState),
    /// Panelzustand fuer das Parkplatz-Tool.
    Parking(ParkingPanelState),
    /// Panelzustand fuer das Feldgrenz-Tool.
    FieldBoundary(FieldBoundaryPanelState),
    /// Panelzustand fuer das Feldweg-Tool.
    FieldPath(FieldPathPanelState),
    /// Panelzustand fuer das Strecken-Versatz-Tool.
    RouteOffset(RouteOffsetPanelState),
    /// Panelzustand fuer das Farb-Pfad-Tool.
    ColorPath(ColorPathPanelState),
}

/// Semantische Panel-Aktion fuer das aktive Route-Tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RouteToolPanelAction {
    /// Panel-Aktion fuer das Gerade-Strecke-Tool.
    Straight(StraightPanelAction),
    /// Panel-Aktion fuer das Bézier-Kurven-Tool.
    Curve(CurvePanelAction),
    /// Panel-Aktion fuer das Catmull-Rom-Spline-Tool.
    Spline(SplinePanelAction),
    /// Panel-Aktion fuer das Geglaettete-Kurve-Tool.
    SmoothCurve(SmoothCurvePanelAction),
    /// Panel-Aktion fuer das Ausweichstrecken-Tool.
    Bypass(BypassPanelAction),
    /// Panel-Aktion fuer das Parkplatz-Tool.
    Parking(ParkingPanelAction),
    /// Panel-Aktion fuer das Feldgrenz-Tool.
    FieldBoundary(FieldBoundaryPanelAction),
    /// Panel-Aktion fuer das Feldweg-Tool.
    FieldPath(FieldPathPanelAction),
    /// Panel-Aktion fuer das Strecken-Versatz-Tool.
    RouteOffset(RouteOffsetPanelAction),
    /// Panel-Aktion fuer das Farb-Pfad-Tool.
    ColorPath(ColorPathPanelAction),
}
