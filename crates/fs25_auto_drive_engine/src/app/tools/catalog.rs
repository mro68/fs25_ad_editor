//! Kanonischer Tool-Katalog fuer alle Route-Tools.

use crate::app::tool_contract::RouteToolId;
use crate::shared::I18nKey;

use super::{
    bypass, color_path, curve, field_boundary, field_path, parking, route_offset, smooth_curve,
    spline, straight_line, RouteTool,
};

/// Gemeinsame Anzeige-Gruppe eines Route-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolGroup {
    /// Grundlegende Streckenwerkzeuge.
    Basics,
    /// Abschnitts- und Generator-Werkzeuge.
    Section,
    /// Analyse-Werkzeuge.
    Analysis,
}

/// UI-Surface fuer Route-Tool-Eintraege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolSurface {
    /// Schwebendes Floating-Menue.
    FloatingMenu,
    /// Defaults-Panel in der Sidebar.
    DefaultsPanel,
    /// Hauptmenue.
    MainMenu,
    /// Command Palette.
    CommandPalette,
}

/// Katalogschluessel fuer Route-Tool-Icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolIconKey {
    /// Icon fuer Gerade Strecke.
    Straight,
    /// Icon fuer Bézier Grad 2.
    CurveQuad,
    /// Icon fuer Bézier Grad 3.
    CurveCubic,
    /// Icon fuer Spline.
    Spline,
    /// Icon fuer Ausweichstrecke.
    Bypass,
    /// Icon fuer Geglaettete Kurve.
    SmoothCurve,
    /// Icon fuer Parkplatz.
    Parking,
    /// Icon fuer Feldgrenze.
    FieldBoundary,
    /// Icon fuer Feldweg.
    FieldPath,
    /// Icon fuer Streckenversatz.
    RouteOffset,
    /// Icon fuer Farbpfad.
    ColorPath,
}

/// Verfuegbarkeits-Anforderung eines Route-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolRequirement {
    /// Erfordert geladene Farmland-Daten.
    FarmlandLoaded,
    /// Erfordert ein geladenes Hintergrundbild.
    BackgroundLoaded,
    /// Erfordert eine geordnete Ketten-Selektion.
    OrderedChainSelection,
}

/// Persistenz- und Editierbarkeitsvertrag eines Route-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolBackingMode {
    /// Sichtbar und ausfuehrbar, aber ohne GroupRecord.
    Ephemeral,
    /// Persistiert, aber nicht editierbar.
    GroupBackedReadOnly,
    /// Persistiert und ueber Tool-Edit nachbearbeitbar.
    GroupBackedEditable,
}

impl RouteToolBackingMode {
    /// Gibt `true` zurueck wenn das Tool einen GroupRecord-Vertrag hat.
    pub fn is_group_backed(self) -> bool {
        matches!(self, Self::GroupBackedReadOnly | Self::GroupBackedEditable)
    }

    /// Gibt `true` zurueck wenn das Tool ueber den Tool-Edit-Flow bearbeitbar ist.
    pub fn is_editable(self) -> bool {
        matches!(self, Self::GroupBackedEditable)
    }
}

/// Ueber den Katalog aufgeloester Grund fuer einen deaktivierten Tool-Eintrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolDisabledReason {
    /// Farmland-Daten fehlen.
    MissingFarmland,
    /// Hintergrundbild fehlt.
    MissingBackground,
    /// Geordnete Ketten-Selektion fehlt.
    MissingOrderedChain,
}

/// Read-only Kontext fuer die Verfuegbarkeitsaufloesung.
#[derive(Debug, Clone, Copy, Default)]
pub struct RouteToolAvailabilityContext {
    /// Farmland-Daten sind geladen.
    pub has_farmland: bool,
    /// Hintergrundbild ist geladen.
    pub has_background: bool,
    /// Die aktuelle Selektion bildet eine geordnete Kette.
    pub has_ordered_chain: bool,
}

/// Kanonischer Descriptor eines Route-Tools.
#[derive(Clone, Copy)]
pub struct RouteToolDescriptor {
    /// Stabile Tool-ID.
    pub id: RouteToolId,
    /// Kanonischer Anzeigename des Tools.
    pub name: &'static str,
    /// Legacy-Icon fuer textbasierte Toollisten und Tests.
    pub legacy_icon: &'static str,
    /// Kurzbeschreibung des Tools.
    pub description: &'static str,
    /// Katalogschluessel fuer die Icon-Aufloesung in UI-Surfaces.
    pub icon_key: RouteToolIconKey,
    /// Anzeige-Gruppe ueber alle Surfaces.
    pub group: RouteToolGroup,
    /// Surfaces, auf denen das Tool sichtbar ist.
    pub visible_on: &'static [RouteToolSurface],
    /// Aktivierungs-Voraussetzungen.
    pub requirements: &'static [RouteToolRequirement],
    /// Persistenz- und Editierbarkeitsvertrag.
    pub backing_mode: RouteToolBackingMode,
    /// Factory fuer die Tool-Instanz im ToolManager.
    pub factory: fn() -> Box<dyn RouteTool>,
}

impl std::fmt::Debug for RouteToolDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteToolDescriptor")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("group", &self.group)
            .field("backing_mode", &self.backing_mode)
            .finish()
    }
}

impl RouteToolDescriptor {
    /// Liefert den kanonischen Slot dieses Tools im Katalog.
    pub fn slot(self) -> usize {
        route_tool_slot(self.id).expect("invariant: descriptor id exists in catalog")
    }

    /// Prueft die Sichtbarkeit auf einer bestimmten Surface.
    pub fn visible_on(self, surface: RouteToolSurface) -> bool {
        self.visible_on.contains(&surface)
    }

    /// Prueft ob der Descriptor ueber den Tool-Edit-Flow bearbeitbar ist.
    pub fn is_editable(self) -> bool {
        self.backing_mode.is_editable()
    }
}

/// Aufgeloester Surface-Eintrag auf Basis des Katalogs und des Availability-Kontexts.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedRouteToolEntry {
    /// Descriptor des Route-Tools.
    pub descriptor: &'static RouteToolDescriptor,
    /// Kanonischer Slot im Katalog.
    pub slot: usize,
    /// Ist der Eintrag aktuell aktivierbar?
    pub enabled: bool,
    /// Optionaler Disabled-Grund.
    pub disabled_reason: Option<RouteToolDisabledReason>,
}

const ALL_ROUTE_TOOL_SURFACES: [RouteToolSurface; 4] = [
    RouteToolSurface::FloatingMenu,
    RouteToolSurface::DefaultsPanel,
    RouteToolSurface::MainMenu,
    RouteToolSurface::CommandPalette,
];

const REQUIREMENTS_NONE: [RouteToolRequirement; 0] = [];
const REQUIREMENT_FARMLAND: [RouteToolRequirement; 1] = [RouteToolRequirement::FarmlandLoaded];
const REQUIREMENT_BACKGROUND: [RouteToolRequirement; 1] = [RouteToolRequirement::BackgroundLoaded];
const REQUIREMENT_ORDERED_CHAIN: [RouteToolRequirement; 1] =
    [RouteToolRequirement::OrderedChainSelection];

fn make_straight() -> Box<dyn RouteTool> {
    Box::new(straight_line::StraightLineTool::new())
}

fn make_curve_quad() -> Box<dyn RouteTool> {
    Box::new(curve::CurveTool::new())
}

fn make_curve_cubic() -> Box<dyn RouteTool> {
    Box::new(curve::CurveTool::new_cubic())
}

fn make_spline() -> Box<dyn RouteTool> {
    Box::new(spline::SplineTool::new())
}

fn make_bypass() -> Box<dyn RouteTool> {
    Box::new(bypass::BypassTool::new())
}

fn make_smooth_curve() -> Box<dyn RouteTool> {
    Box::new(smooth_curve::SmoothCurveTool::new())
}

fn make_parking() -> Box<dyn RouteTool> {
    Box::new(parking::ParkingTool::new())
}

fn make_field_boundary() -> Box<dyn RouteTool> {
    Box::new(field_boundary::FieldBoundaryTool::new())
}

fn make_field_path() -> Box<dyn RouteTool> {
    Box::new(field_path::FieldPathTool::new())
}

fn make_route_offset() -> Box<dyn RouteTool> {
    Box::new(route_offset::RouteOffsetTool::new())
}

fn make_color_path() -> Box<dyn RouteTool> {
    Box::new(color_path::ColorPathTool::new())
}

/// Kanonischer Katalog aller Route-Tools.
pub const ROUTE_TOOL_CATALOG: [RouteToolDescriptor; 11] = [
    RouteToolDescriptor {
        id: RouteToolId::Straight,
        name: "Gerade Strecke",
        legacy_icon: "━",
        description: "Zeichnet eine gerade Linie zwischen zwei Punkten mit Zwischen-Nodes",
        icon_key: RouteToolIconKey::Straight,
        group: RouteToolGroup::Basics,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENTS_NONE,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_straight,
    },
    RouteToolDescriptor {
        id: RouteToolId::CurveQuad,
        name: "Bézier Grad 2",
        legacy_icon: "⌒",
        description: "Zeichnet eine quadratische Bézier-Kurve mit einem Steuerpunkt",
        icon_key: RouteToolIconKey::CurveQuad,
        group: RouteToolGroup::Basics,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENTS_NONE,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_curve_quad,
    },
    RouteToolDescriptor {
        id: RouteToolId::CurveCubic,
        name: "Bézier Grad 3",
        legacy_icon: "〜",
        description: "Zeichnet eine kubische Bézier-Kurve mit zwei Steuerpunkten",
        icon_key: RouteToolIconKey::CurveCubic,
        group: RouteToolGroup::Basics,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENTS_NONE,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_curve_cubic,
    },
    RouteToolDescriptor {
        id: RouteToolId::Spline,
        name: "Spline",
        legacy_icon: "〰",
        description: "Zeichnet einen Catmull-Rom-Spline durch alle geklickten Punkte",
        icon_key: RouteToolIconKey::Spline,
        group: RouteToolGroup::Basics,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENTS_NONE,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_spline,
    },
    RouteToolDescriptor {
        id: RouteToolId::Bypass,
        name: "Ausweichstrecke",
        legacy_icon: "⤴",
        description: "Generiert eine parallele Ausweichstrecke zur selektierten Kette",
        icon_key: RouteToolIconKey::Bypass,
        group: RouteToolGroup::Section,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENT_ORDERED_CHAIN,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_bypass,
    },
    RouteToolDescriptor {
        id: RouteToolId::SmoothCurve,
        name: "Geglättete Kurve",
        legacy_icon: "⊿",
        description:
            "Erzeugt eine winkelgeglaettete Route mit automatischen Tangenten-Uebergaengen",
        icon_key: RouteToolIconKey::SmoothCurve,
        group: RouteToolGroup::Basics,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENTS_NONE,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_smooth_curve,
    },
    RouteToolDescriptor {
        id: RouteToolId::Parking,
        name: "Parkplatz",
        legacy_icon: "\u{1f17f}",
        description: "Erzeugt ein Parkplatz-Layout mit Wendekreis",
        icon_key: RouteToolIconKey::Parking,
        group: RouteToolGroup::Section,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENTS_NONE,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_parking,
    },
    RouteToolDescriptor {
        id: RouteToolId::FieldBoundary,
        name: "Feld erkennen",
        legacy_icon: "\u{1f33e}",
        description: "Erzeugt eine Route entlang der erkannten Feldgrenze",
        icon_key: RouteToolIconKey::FieldBoundary,
        group: RouteToolGroup::Analysis,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENT_FARMLAND,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_field_boundary,
    },
    RouteToolDescriptor {
        id: RouteToolId::FieldPath,
        name: "Feldweg",
        legacy_icon: "\u{1f6e4}",
        description: "Berechnet Mittellinien zwischen Farmland-Grenzen",
        icon_key: RouteToolIconKey::FieldPath,
        group: RouteToolGroup::Analysis,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENT_FARMLAND,
        backing_mode: RouteToolBackingMode::Ephemeral,
        factory: make_field_path,
    },
    RouteToolDescriptor {
        id: RouteToolId::RouteOffset,
        name: "Strecke versetzen",
        legacy_icon: "⇶",
        description: "Verschiebt eine selektierte Kette parallel nach links und/oder rechts",
        icon_key: RouteToolIconKey::RouteOffset,
        group: RouteToolGroup::Section,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENT_ORDERED_CHAIN,
        backing_mode: RouteToolBackingMode::GroupBackedEditable,
        factory: make_route_offset,
    },
    RouteToolDescriptor {
        id: RouteToolId::ColorPath,
        name: "Farb-Pfad",
        legacy_icon: "🎨",
        description: "Erkennt Wege anhand der Farbe im Hintergrundbild",
        icon_key: RouteToolIconKey::ColorPath,
        group: RouteToolGroup::Analysis,
        visible_on: &ALL_ROUTE_TOOL_SURFACES,
        requirements: &REQUIREMENT_BACKGROUND,
        backing_mode: RouteToolBackingMode::Ephemeral,
        factory: make_color_path,
    },
];

/// Liefert den gesamten Route-Tool-Katalog.
pub fn route_tool_catalog() -> &'static [RouteToolDescriptor] {
    &ROUTE_TOOL_CATALOG
}

/// Liefert den Descriptor fuer eine stabile Tool-ID.
pub fn route_tool_descriptor(tool_id: RouteToolId) -> &'static RouteToolDescriptor {
    ROUTE_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.id == tool_id)
        .expect("invariant: every RouteToolId exists in ROUTE_TOOL_CATALOG")
}

/// Liefert den Descriptor fuer einen kanonischen Slot.
pub fn route_tool_descriptor_by_slot(slot: usize) -> Option<&'static RouteToolDescriptor> {
    ROUTE_TOOL_CATALOG.get(slot)
}

/// Liefert den kanonischen Slot fuer eine Tool-ID.
pub fn route_tool_slot(tool_id: RouteToolId) -> Option<usize> {
    ROUTE_TOOL_CATALOG
        .iter()
        .position(|descriptor| descriptor.id == tool_id)
}

/// Liefert die route-tool-spezifische Gruppenbeschriftung.
pub fn route_tool_group_label_key(group: RouteToolGroup) -> I18nKey {
    match group {
        RouteToolGroup::Basics => I18nKey::SidebarBasics,
        RouteToolGroup::Section => I18nKey::SidebarEdit,
        RouteToolGroup::Analysis => I18nKey::SidebarAnalysis,
    }
}

/// Liefert das kurze Label fuer Surfaces wie Floating-Menue, Hauptmenue und Palette.
pub fn route_tool_label_key(tool_id: RouteToolId) -> I18nKey {
    match tool_id {
        RouteToolId::Straight => I18nKey::FloatingBasicStraight,
        RouteToolId::CurveQuad => I18nKey::FloatingBasicQuadratic,
        RouteToolId::CurveCubic => I18nKey::FloatingBasicCubic,
        RouteToolId::Spline => I18nKey::FloatingBasicSpline,
        RouteToolId::Bypass => I18nKey::FloatingEditBypass,
        RouteToolId::SmoothCurve => I18nKey::FloatingBasicSmoothCurve,
        RouteToolId::Parking => I18nKey::FloatingEditParking,
        RouteToolId::FieldBoundary => I18nKey::FloatingAnalysisFieldBoundary,
        RouteToolId::FieldPath => I18nKey::FloatingAnalysisFieldPath,
        RouteToolId::RouteOffset => I18nKey::FloatingEditRouteOffset,
        RouteToolId::ColorPath => I18nKey::FloatingAnalysisColorPath,
    }
}

/// Liefert den Defaults-Panel-Tooltip eines Route-Tools.
pub fn route_tool_defaults_tooltip_key(tool_id: RouteToolId) -> I18nKey {
    match tool_id {
        RouteToolId::Straight => I18nKey::LpStraight,
        RouteToolId::CurveQuad => I18nKey::LpCurveQuad,
        RouteToolId::CurveCubic => I18nKey::LpCurveCubic,
        RouteToolId::Spline => I18nKey::LpSpline,
        RouteToolId::Bypass => I18nKey::LpBypass,
        RouteToolId::SmoothCurve => I18nKey::LpSmoothCurve,
        RouteToolId::Parking => I18nKey::LpParking,
        RouteToolId::FieldBoundary => I18nKey::LpFieldBoundary,
        RouteToolId::FieldPath => I18nKey::LpFieldPath,
        RouteToolId::RouteOffset => I18nKey::LpRouteOffset,
        RouteToolId::ColorPath => I18nKey::LpColorPath,
    }
}

/// Liefert den i18n-Schluessel fuer einen Disabled-Grund.
pub fn route_tool_disabled_reason_key(reason: RouteToolDisabledReason) -> I18nKey {
    match reason {
        RouteToolDisabledReason::MissingFarmland => I18nKey::RouteToolNeedFarmland,
        RouteToolDisabledReason::MissingBackground => I18nKey::RouteToolNeedBackground,
        RouteToolDisabledReason::MissingOrderedChain => I18nKey::RouteToolNeedOrderedChain,
    }
}

/// Ermittelt den optionalen Disabled-Grund aus dem Availability-Kontext.
pub fn route_tool_disabled_reason(
    descriptor: &RouteToolDescriptor,
    context: RouteToolAvailabilityContext,
) -> Option<RouteToolDisabledReason> {
    for requirement in descriptor.requirements {
        let missing = match requirement {
            RouteToolRequirement::FarmlandLoaded if !context.has_farmland => {
                Some(RouteToolDisabledReason::MissingFarmland)
            }
            RouteToolRequirement::BackgroundLoaded if !context.has_background => {
                Some(RouteToolDisabledReason::MissingBackground)
            }
            RouteToolRequirement::OrderedChainSelection if !context.has_ordered_chain => {
                Some(RouteToolDisabledReason::MissingOrderedChain)
            }
            _ => None,
        };
        if missing.is_some() {
            return missing;
        }
    }
    None
}

/// Loest die Eintraege einer Gruppe fuer eine bestimmte Surface auf.
pub fn resolve_route_tool_entries(
    surface: RouteToolSurface,
    group: RouteToolGroup,
    context: RouteToolAvailabilityContext,
) -> Vec<ResolvedRouteToolEntry> {
    ROUTE_TOOL_CATALOG
        .iter()
        .filter(|descriptor| descriptor.group == group && descriptor.visible_on(surface))
        .map(|descriptor| {
            let disabled_reason = route_tool_disabled_reason(descriptor, context);
            ResolvedRouteToolEntry {
                descriptor,
                slot: descriptor.slot(),
                enabled: disabled_reason.is_none(),
                disabled_reason,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids_for(surface: RouteToolSurface, group: RouteToolGroup) -> Vec<RouteToolId> {
        resolve_route_tool_entries(surface, group, RouteToolAvailabilityContext::default())
            .into_iter()
            .map(|entry| entry.descriptor.id)
            .collect()
    }

    #[test]
    fn route_tool_catalog_ist_vollstaendig_und_eindeutig() {
        let ids: Vec<RouteToolId> = route_tool_catalog().iter().map(|entry| entry.id).collect();
        assert_eq!(ids, RouteToolId::ALL);

        let mut sorted = ids.clone();
        sorted.sort_by_key(|tool_id| route_tool_slot(*tool_id).unwrap_or(usize::MAX));
        sorted.dedup();
        assert_eq!(sorted.len(), RouteToolId::ALL.len());
    }

    #[test]
    fn alle_surfaces_nutzen_die_gleiche_gruppenmatrix() {
        let expected_basics = vec![
            RouteToolId::Straight,
            RouteToolId::CurveQuad,
            RouteToolId::CurveCubic,
            RouteToolId::Spline,
            RouteToolId::SmoothCurve,
        ];
        let expected_section = vec![
            RouteToolId::Bypass,
            RouteToolId::Parking,
            RouteToolId::RouteOffset,
        ];
        let expected_analysis = vec![
            RouteToolId::FieldBoundary,
            RouteToolId::FieldPath,
            RouteToolId::ColorPath,
        ];

        for surface in [
            RouteToolSurface::FloatingMenu,
            RouteToolSurface::DefaultsPanel,
            RouteToolSurface::MainMenu,
            RouteToolSurface::CommandPalette,
        ] {
            assert_eq!(ids_for(surface, RouteToolGroup::Basics), expected_basics);
            assert_eq!(ids_for(surface, RouteToolGroup::Section), expected_section);
            assert_eq!(
                ids_for(surface, RouteToolGroup::Analysis),
                expected_analysis
            );
        }
    }

    #[test]
    fn persistenz_und_editierbarkeit_sind_explizit_fuer_analysis_ausnahmen() {
        let field_boundary = route_tool_descriptor(RouteToolId::FieldBoundary);
        assert!(field_boundary.backing_mode.is_group_backed());
        assert!(field_boundary.backing_mode.is_editable());

        for tool_id in [RouteToolId::FieldPath, RouteToolId::ColorPath] {
            let descriptor = route_tool_descriptor(tool_id);
            assert_eq!(descriptor.group, RouteToolGroup::Analysis);
            assert_eq!(descriptor.backing_mode, RouteToolBackingMode::Ephemeral);
            assert!(!descriptor.backing_mode.is_group_backed());
            assert!(!descriptor.backing_mode.is_editable());
        }
    }
}
