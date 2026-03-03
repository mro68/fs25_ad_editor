//! Unit-Tests für den Constraint-Route-Solver.

use crate::app::tools::constraint_route::geometry::*;
use glam::Vec2;

#[test]
fn gerade_strecke_ohne_constraints() {
    let input = ConstraintRouteInput {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(30.0, 0.0),
        control_nodes: vec![],
        max_segment_length_m: 6.0,
        max_direction_change_deg: 45.0,
        start_neighbor_directions: vec![],
        end_neighbor_directions: vec![],
    };
    let result = solve_route(&input);
    // Sollte mindestens Start + End enthalten
    assert!(
        result.len() >= 2,
        "Mindestens 2 Punkte erwartet, got {}",
        result.len()
    );
    // Erster Punkt ≈ Start, letzter ≈ End
    assert!((result[0] - input.start).length() < 0.1);
    assert!((result.last().unwrap() - &input.end).length() < 0.1);
}

#[test]
fn strecke_mit_kontrollpunkt() {
    let input = ConstraintRouteInput {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(30.0, 0.0),
        control_nodes: vec![Vec2::new(15.0, 10.0)],
        max_segment_length_m: 6.0,
        max_direction_change_deg: 45.0,
        start_neighbor_directions: vec![],
        end_neighbor_directions: vec![],
    };
    let result = solve_route(&input);
    assert!(
        result.len() >= 3,
        "Mindestens 3 Punkte erwartet mit Kontrollpunkt"
    );
}

#[test]
fn max_segment_laenge_eingehalten() {
    let input = ConstraintRouteInput {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(60.0, 0.0),
        control_nodes: vec![],
        max_segment_length_m: 5.0,
        max_direction_change_deg: 90.0,
        start_neighbor_directions: vec![],
        end_neighbor_directions: vec![],
    };
    let result = solve_route(&input);
    // Prüfe dass kein Segment länger als max_segment_length + Toleranz ist
    for window in result.windows(2) {
        let dist = window[0].distance(window[1]);
        assert!(dist <= 5.0 + 0.5, "Segment zu lang: {:.2} (max 5.0)", dist);
    }
}

#[test]
fn steerer_wird_bei_scharfem_winkel_eingefuegt() {
    // Nachbar-Richtung senkrecht zur Route-Richtung
    let input = ConstraintRouteInput {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(30.0, 0.0),
        control_nodes: vec![],
        max_segment_length_m: 6.0,
        max_direction_change_deg: 30.0,
        start_neighbor_directions: vec![Vec2::new(0.0, 1.0)], // Nachbar nach oben
        end_neighbor_directions: vec![],
    };
    let result = solve_route(&input);
    // Sollte mehr Punkte haben als ohne Steerer (Steerer + Glättung)
    let input_ohne = ConstraintRouteInput {
        start_neighbor_directions: vec![],
        ..input.clone()
    };
    let result_ohne = solve_route(&input_ohne);
    assert!(
        result.len() >= result_ohne.len(),
        "Mit Steerer sollten mindestens so viele Punkte wie ohne entstehen"
    );
}

#[test]
fn kurze_strecke_liefert_mindestens_2_punkte() {
    let input = ConstraintRouteInput {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(1.0, 0.0),
        control_nodes: vec![],
        max_segment_length_m: 6.0,
        max_direction_change_deg: 45.0,
        start_neighbor_directions: vec![],
        end_neighbor_directions: vec![],
    };
    let result = solve_route(&input);
    assert!(result.len() >= 2);
}

#[test]
fn identische_start_end_position() {
    let input = ConstraintRouteInput {
        start: Vec2::new(10.0, 10.0),
        end: Vec2::new(10.0, 10.0),
        control_nodes: vec![],
        max_segment_length_m: 6.0,
        max_direction_change_deg: 45.0,
        start_neighbor_directions: vec![],
        end_neighbor_directions: vec![],
    };
    let result = solve_route(&input);
    // Sollte nicht abstürzen, mindestens 1 Punkt
    assert!(!result.is_empty());
}
