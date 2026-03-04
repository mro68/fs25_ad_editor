//! Unit-Tests fuer den Constraint-Route-Solver.

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
        min_distance: 0.0,
    };
    let result = solve_route(&input);
    assert!(
        result.positions.len() >= 2,
        "Mindestens 2 Punkte erwartet, got {}",
        result.positions.len()
    );
    assert!((result.positions[0] - input.start).length() < 0.1);
    assert!((result.positions.last().unwrap() - input.end).length() < 0.1);
    // Keine Steuerpunkte ohne Nachbar-Richtungen
    assert!(result.approach_steerer.is_none());
    assert!(result.departure_steerer.is_none());
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
        min_distance: 0.0,
    };
    let result = solve_route(&input);
    assert!(
        result.positions.len() >= 3,
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
        min_distance: 0.0,
    };
    let result = solve_route(&input);
    for window in result.positions.windows(2) {
        let dist = window[0].distance(window[1]);
        assert!(dist <= 5.0 + 0.5, "Segment zu lang: {:.2} (max 5.0)", dist);
    }
}

#[test]
fn steerer_wird_bei_scharfem_winkel_eingefuegt() {
    // Nachbar-Richtung senkrecht zur Route-Richtung → 90° Winkel
    let input = ConstraintRouteInput {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(30.0, 0.0),
        control_nodes: vec![],
        max_segment_length_m: 6.0,
        max_direction_change_deg: 30.0,
        start_neighbor_directions: vec![Vec2::new(0.0, -1.0)], // Nachbar nach unten
        end_neighbor_directions: vec![],
        min_distance: 0.0,
    };
    let result = solve_route(&input);
    // Approach-Steuerpunkt sollte vorhanden sein
    assert!(
        result.approach_steerer.is_some(),
        "Approach-Steuerpunkt erwartet bei scharfem Winkel"
    );
    // Steuerpunkt muss in +Y-Richtung liegen (weg vom Nachbar)
    let ap = result.approach_steerer.unwrap();
    assert!(
        ap.y > 0.0,
        "Approach-Steuerpunkt sollte in +Y-Richtung liegen: {:?}",
        ap
    );
    // Mehr Punkte als ohne Steerer
    let input_ohne = ConstraintRouteInput {
        start_neighbor_directions: vec![],
        ..input.clone()
    };
    let result_ohne = solve_route(&input_ohne);
    assert!(
        result.positions.len() >= result_ohne.positions.len(),
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
        min_distance: 0.0,
    };
    let result = solve_route(&input);
    assert!(result.positions.len() >= 2);
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
        min_distance: 0.0,
    };
    let result = solve_route(&input);
    assert!(!result.positions.is_empty());
}

#[test]
fn approach_steerer_kein_steerer_bei_kleinem_winkel() {
    // forward=(1,0), neighbor_dir=(-1,0) → approach_dir=(1,0) → angle=0 → kein Steerer
    let result = compute_approach_steerer(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        &[Vec2::new(-1.0, 0.0)],
        std::f32::consts::FRAC_PI_4, // 45°
        5.0,
        10.0,
    );
    assert!(result.is_none(), "Winkel ist 0° → kein Steerer erwartet");
}

#[test]
fn approach_steerer_bei_90_grad_winkel() {
    // forward=(1,0), neighbor_dir=(0,-1) → approach_dir=(0,1) → angle=90°
    let result = compute_approach_steerer(
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        &[Vec2::new(0.0, -1.0)],
        std::f32::consts::FRAC_PI_4, // 45°
        5.0,
        10.0,
    );
    assert!(result.is_some(), "Winkel ist 90° → Steerer erwartet");
    let p = result.unwrap();
    assert!(p.y > 0.0, "Steerer sollte in +Y-Richtung liegen: {:?}", p);
}

#[test]
fn departure_steerer_bei_scharfem_winkel() {
    // Route kommt von links (forward=(1,0)), Ende hat Nachbar nach rechts
    let result = compute_departure_steerer(
        Vec2::new(10.0, 0.0),
        Vec2::new(1.0, 0.0),         // forward
        &[Vec2::new(1.0, 0.0)],      // Nachbar in gleicher Richtung
        std::f32::consts::FRAC_PI_4, // 45°
        5.0,
        10.0,
    );
    // forward=(1,0), best_dir=(1,0), depart_dir=(1,0), angle = acos(1)=0 → kein Steerer
    assert!(
        result.is_none(),
        "Nachbar in Fahrtrichtung → kein Steerer noetig"
    );
}

#[test]
fn solver_result_enthaelt_steuerpunkte() {
    let input = ConstraintRouteInput {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(20.0, 0.0),
        control_nodes: vec![],
        max_segment_length_m: 5.0,
        max_direction_change_deg: 30.0,
        start_neighbor_directions: vec![Vec2::new(0.0, -1.0)],
        end_neighbor_directions: vec![Vec2::new(0.0, 1.0)],
        min_distance: 0.0,
    };
    let result = solve_route(&input);
    assert!(
        result.approach_steerer.is_some(),
        "Approach-Steuerpunkt erwartet"
    );
    assert!(
        result.departure_steerer.is_some(),
        "Departure-Steuerpunkt erwartet"
    );
    assert!(result.positions.len() >= 2);
}
