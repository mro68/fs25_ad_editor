//! Drag-Logik fuer das ColorPath-Tool (CP-08).
//!
//! Stellt das [`RouteToolDrag`](super::super::RouteToolDrag)-Verhalten fuer die
//! Wizard-Phase [`ColorPathPhase::Editing`](super::state::ColorPathPhase::Editing)
//! bereit. In `Idle`/`Sampling` bleibt das Tool Drag-inaktiv.
//!
//! Die eigentliche Zustandsaenderung geschieht ueber
//! [`super::editable::EditableCenterlines::move_junction`], das
//! [`super::editable::EditableCenterlines::bump_revision`] aufruft und dadurch
//! den Stage-F-Cache aus CP-07 automatisch invalidiert.

use glam::Vec2;

use crate::core::RoadMap;

use super::editable::EditableJunctionId;
use super::state::ColorPathTool;

/// Gibt die Weltpositionen aller aktuell draggbaren Junctions zurueck.
///
/// Nur in der Phase [`ColorPathPhase::Editing`] werden Treffer geliefert;
/// in allen anderen Phasen bleibt der Vektor leer. Die Reihenfolge ist
/// deterministisch aufsteigend nach [`EditableJunctionId`], damit Hosts und
/// UI-Snapshots stabile Indizes beobachten (F4).
pub(crate) fn drag_targets(tool: &ColorPathTool) -> Vec<Vec2> {
    if !tool.phase.is_editing() {
        return Vec::new();
    }
    let Some(editable) = tool.editable.as_ref() else {
        return Vec::new();
    };
    let mut entries: Vec<(EditableJunctionId, Vec2)> = editable
        .junctions
        .iter()
        .map(|(id, junction)| (*id, junction.world_pos))
        .collect();
    entries.sort_by_key(|(id, _)| id.0);
    entries.into_iter().map(|(_, pos)| pos).collect()
}

/// Sucht die naechstgelegene Junction innerhalb des Pick-Radius.
///
/// Liefert `None`, wenn keine Junction im Radius liegt oder das Tool nicht
/// in [`ColorPathPhase::Editing`] ist.
pub(crate) fn pick_junction(
    tool: &ColorPathTool,
    pos: Vec2,
    pick_radius: f32,
) -> Option<EditableJunctionId> {
    if !tool.phase.is_editing() {
        return None;
    }
    let editable = tool.editable.as_ref()?;
    let mut best: Option<(EditableJunctionId, f32)> = None;
    for junction in editable.junctions.values() {
        let dist = junction.world_pos.distance(pos);
        if dist <= pick_radius && best.is_none_or(|(_, prev)| dist < prev) {
            best = Some((junction.id, dist));
        }
    }
    best.map(|(id, _)| id)
}

/// Startet einen Junction-Drag, falls `pos` nahe einer Junction liegt.
pub(crate) fn on_drag_start(
    tool: &mut ColorPathTool,
    pos: Vec2,
    _road_map: &RoadMap,
    pick_radius: f32,
) -> bool {
    let Some(id) = pick_junction(tool, pos, pick_radius) else {
        return false;
    };
    tool.dragging_junction = Some(id);
    true
}

/// Aktualisiert die Position der gegriffenen Junction waehrend des Drags.
///
/// Ruft [`super::editable::EditableCenterlines::move_junction`] auf; dieser
/// bumpt die Revision und invalidiert damit den Stage-F-Cache (CP-07).
/// Setzt zusaetzlich das `editable_user_dirty`-Flag (CP-03), damit
/// nachfolgende Matching-/Preview-Core-Slider-Resyncs die User-Position
/// erhalten statt zu ueberschreiben.
pub(crate) fn on_drag_update(tool: &mut ColorPathTool, pos: Vec2) {
    let Some(id) = tool.dragging_junction else {
        return;
    };
    if let Some(editable) = tool.editable.as_mut() {
        editable.move_junction(id, pos);
        tool.editable_user_dirty = true;
    }
}

/// Beendet den Junction-Drag und bumpt die Revision final.
///
/// Der zusaetzliche Revisions-Bump stellt sicher, dass auch bei einem Drag
/// ohne Positionsaenderung (z. B. Drag-Klick ohne Bewegung) ein nachfolgender
/// Undo-/Redo-Hook einen eindeutigen Zustand sieht.
pub(crate) fn on_drag_end(tool: &mut ColorPathTool, _road_map: &RoadMap) {
    if tool.dragging_junction.take().is_some() {
        tool.bump_editable_revision();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tools::color_path::editable::EditableCenterlines;
    use crate::app::tools::color_path::skeleton::{
        SkeletonGraphNode, SkeletonGraphNodeKind, SkeletonGraphSegment, SkeletonNetwork,
    };
    use crate::app::tools::color_path::state::ColorPathPhase;

    fn sample_network() -> SkeletonNetwork {
        SkeletonNetwork {
            nodes: vec![
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::OpenEnd,
                    pixel_position: Vec2::ZERO,
                    world_position: Vec2::new(0.0, 0.0),
                },
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::Junction,
                    pixel_position: Vec2::ZERO,
                    world_position: Vec2::new(10.0, 0.0),
                },
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::OpenEnd,
                    pixel_position: Vec2::ZERO,
                    world_position: Vec2::new(10.0, 10.0),
                },
            ],
            segments: vec![
                SkeletonGraphSegment {
                    start_node: 0,
                    end_node: 1,
                    polyline: vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)],
                },
                SkeletonGraphSegment {
                    start_node: 1,
                    end_node: 2,
                    polyline: vec![Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)],
                },
            ],
        }
    }

    fn tool_in_junction_edit() -> ColorPathTool {
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Editing;
        tool.editable = Some(EditableCenterlines::from_skeleton_network(&sample_network()));
        tool
    }

    #[test]
    fn drag_targets_empty_outside_junction_edit() {
        let mut tool = tool_in_junction_edit();
        tool.phase = ColorPathPhase::Sampling;
        assert!(drag_targets(&tool).is_empty());
    }

    #[test]
    fn pick_junction_returns_closest_within_radius() {
        let tool = tool_in_junction_edit();

        // Nahe Junction #1 (10.0, 0.0)
        let pick = pick_junction(&tool, Vec2::new(10.5, 0.2), 1.0);
        assert_eq!(pick, Some(EditableJunctionId(1)));

        // Weit weg von allen Junctions → None
        assert_eq!(pick_junction(&tool, Vec2::new(100.0, 100.0), 1.0), None);
    }

    #[test]
    fn pick_junction_returns_none_outside_junction_edit() {
        let mut tool = tool_in_junction_edit();
        tool.phase = ColorPathPhase::Sampling;
        assert_eq!(pick_junction(&tool, Vec2::new(10.0, 0.0), 1.0), None);
    }

    #[test]
    fn drag_update_moves_junction_and_bumps_revision() {
        let mut tool = tool_in_junction_edit();
        let road_map = RoadMap::default();
        let initial_revision = tool
            .editable
            .as_ref()
            .expect("Editable muss existieren")
            .revision;

        assert!(on_drag_start(
            &mut tool,
            Vec2::new(10.0, 0.0),
            &road_map,
            1.0
        ));
        assert_eq!(tool.dragging_junction, Some(EditableJunctionId(1)));

        on_drag_update(&mut tool, Vec2::new(12.0, 3.0));

        let editable = tool.editable.as_ref().expect("Editable muss existieren");
        let junction = &editable.junctions[&EditableJunctionId(1)];
        assert_eq!(junction.world_pos, Vec2::new(12.0, 3.0));
        assert!(
            editable.revision > initial_revision,
            "Drag muss die Revision bumpen (war {}, ist {})",
            initial_revision,
            editable.revision
        );
    }

    #[test]
    fn drag_end_clears_handle_and_bumps_revision() {
        let mut tool = tool_in_junction_edit();
        let road_map = RoadMap::default();
        assert!(on_drag_start(
            &mut tool,
            Vec2::new(10.0, 0.0),
            &road_map,
            1.0
        ));
        on_drag_update(&mut tool, Vec2::new(11.0, 0.0));
        let pre_end_revision = tool
            .editable
            .as_ref()
            .expect("Editable muss existieren")
            .revision;

        on_drag_end(&mut tool, &road_map);

        assert_eq!(tool.dragging_junction, None);
        let editable = tool.editable.as_ref().expect("Editable muss existieren");
        assert!(
            editable.revision > pre_end_revision,
            "on_drag_end muss die Revision final bumpen"
        );
    }

    #[test]
    fn drag_start_fails_outside_pick_radius() {
        let mut tool = tool_in_junction_edit();
        let road_map = RoadMap::default();
        assert!(!on_drag_start(
            &mut tool,
            Vec2::new(50.0, 50.0),
            &road_map,
            1.0
        ));
        assert_eq!(tool.dragging_junction, None);
    }

    #[test]
    fn reset_while_dragging_clears_handle() {
        use crate::app::ui_contract::ColorPathPanelAction;
        let mut tool = tool_in_junction_edit();
        let road_map = RoadMap::default();
        assert!(on_drag_start(
            &mut tool,
            Vec2::new(10.0, 0.0),
            &road_map,
            1.0
        ));
        assert!(
            tool.dragging_junction.is_some(),
            "Drag-Handle muss nach on_drag_start gesetzt sein"
        );

        let _ = tool.apply_panel_action(ColorPathPanelAction::Reset);

        assert!(
            tool.dragging_junction.is_none(),
            "Reset waehrend aktivem Drag muss dragging_junction clearen (T3)"
        );
        assert_eq!(tool.phase, ColorPathPhase::Idle);
        assert!(tool.editable.is_none());
    }

    #[test]
    fn drag_targets_are_sorted_by_junction_id() {
        let tool = tool_in_junction_edit();
        let targets = drag_targets(&tool);
        let editable = tool.editable.as_ref().expect("Editable muss existieren");
        let mut expected: Vec<(EditableJunctionId, Vec2)> = editable
            .junctions
            .iter()
            .map(|(id, junction)| (*id, junction.world_pos))
            .collect();
        expected.sort_by_key(|(id, _)| id.0);
        let expected_positions: Vec<Vec2> = expected.into_iter().map(|(_, pos)| pos).collect();
        assert_eq!(
            targets, expected_positions,
            "drag_targets muss deterministisch nach EditableJunctionId sortiert liefern (F4)"
        );
    }

    /// CP-04: Drag in `Editing` invalidiert den Stage-F-Cache nur lazy.
    ///
    /// Ein Junction-Drag bumpt die Editable-Revision (Bestandteil des
    /// Stage-F-Cache-Keys), schreibt den Cache aber nicht aktiv zurueck. Der
    /// hinterlegte Schluessel bleibt unveraendert; ein nachfolgender
    /// `ensure_prepared_segments`-Aufruf erkennt die Diskrepanz und triggert
    /// erst dann den Rebuild.
    #[test]
    fn drag_in_editor_invalidates_stage_f_cache_lazy() {
        use crate::app::tools::color_path::state::PreparedSegmentsCacheKey;

        let mut tool = tool_in_junction_edit();
        let road_map = RoadMap::default();
        let pre_revision = tool
            .editable
            .as_ref()
            .expect("Editable muss existieren")
            .revision;
        let stale_key = PreparedSegmentsCacheKey {
            preview_core_revision: 0,
            editable_revision: pre_revision,
            simplify_tolerance_bits: 0,
            node_spacing_bits: 0,
            junction_radius_bits: 0,
        };
        tool.cache.prepared_segments_key = Some(stale_key);

        assert!(on_drag_start(
            &mut tool,
            Vec2::new(10.0, 0.0),
            &road_map,
            1.0
        ));
        on_drag_update(&mut tool, Vec2::new(15.0, 5.0));
        on_drag_end(&mut tool, &road_map);

        let post_revision = tool
            .editable
            .as_ref()
            .expect("Editable muss existieren")
            .revision;
        assert!(
            post_revision > pre_revision,
            "Drag muss die Editable-Revision bumpen (Cache-Key-Bestandteil)"
        );
        assert_eq!(
            tool.cache.prepared_segments_key,
            Some(stale_key),
            "Drag darf den Stage-F-Cache nicht eager zurueckschreiben — \
             Invalidierung erfolgt lazy beim naechsten ensure_prepared_segments()"
        );
    }

    /// CP-04: Ausserhalb von `Editing` ist Drag vollstaendig blockiert.
    ///
    /// Sowohl `Idle` als auch `Sampling` muessen `on_drag_start` ablehnen,
    /// `drag_targets`/`pick_junction` leer/`None` liefern und ein
    /// `on_drag_update` ohne aktiven Handle darf weder die Junction-Position
    /// noch die Editable-Revision veraendern.
    #[test]
    fn drag_blocked_outside_editing() {
        let road_map = RoadMap::default();
        for phase in [ColorPathPhase::Idle, ColorPathPhase::Sampling] {
            let mut tool = tool_in_junction_edit();
            tool.phase = phase;
            let pre_revision = tool
                .editable
                .as_ref()
                .expect("Editable muss existieren")
                .revision;

            assert!(
                drag_targets(&tool).is_empty(),
                "drag_targets muss in Phase {:?} leer sein",
                phase
            );
            assert_eq!(
                pick_junction(&tool, Vec2::new(10.0, 0.0), 1.0),
                None,
                "pick_junction muss in Phase {:?} None liefern",
                phase
            );
            assert!(
                !on_drag_start(&mut tool, Vec2::new(10.0, 0.0), &road_map, 1.0),
                "on_drag_start muss in Phase {:?} ablehnen",
                phase
            );
            assert_eq!(
                tool.dragging_junction, None,
                "dragging_junction muss in Phase {:?} unbesetzt bleiben",
                phase
            );

            // on_drag_update ohne aktiven Handle ist no-op
            on_drag_update(&mut tool, Vec2::new(20.0, 20.0));
            let editable = tool.editable.as_ref().expect("Editable muss existieren");
            assert_eq!(
                editable.revision, pre_revision,
                "Editable-Revision darf in Phase {:?} nicht steigen",
                phase
            );
            assert_eq!(
                editable.junctions[&EditableJunctionId(1)].world_pos,
                Vec2::new(10.0, 0.0),
                "Junction-Position darf in Phase {:?} unveraendert bleiben",
                phase
            );
        }
    }
}
