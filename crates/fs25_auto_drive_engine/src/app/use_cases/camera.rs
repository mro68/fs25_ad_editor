//! Use-Case-Funktionen fuer Kamera-Steuerung.

use crate::app::AppState;
use crate::core::RoadMap;

/// Padding-Faktor fuer Zoom-to-Selection (80 % Viewport-Fuellgrad).
const SELECTION_ZOOM_PADDING: f32 = 0.8;

/// Setzt die Kamera auf Default zurueck.
pub fn reset_camera(state: &mut AppState) {
    state.view.camera = Default::default();
}

/// Zoomt die Kamera stufenweise hinein.
pub fn zoom_in(state: &mut AppState) {
    state.view.camera.zoom_by_clamped(
        state.options.camera_zoom_step,
        state.options.camera_zoom_min,
        state.options.camera_zoom_max,
    );
}

/// Zoomt die Kamera stufenweise heraus.
pub fn zoom_out(state: &mut AppState) {
    state.view.camera.zoom_by_clamped(
        1.0 / state.options.camera_zoom_step,
        state.options.camera_zoom_min,
        state.options.camera_zoom_max,
    );
}

/// Verschiebt die Kamera basierend auf einem Delta.
pub fn pan(state: &mut AppState, delta: glam::Vec2) {
    state.view.camera.pan(delta);
}

/// Zoomt auf einen optionalen Fokuspunkt (Mausposition) hin.
///
/// Falls `focus_world` angegeben ist, bleibt der Welt-Punkt unter
/// der Maus nach dem Zoom stabil an derselben Bildschirmposition.
pub fn zoom_towards(state: &mut AppState, factor: f32, focus_world: Option<glam::Vec2>) {
    if let Some(focus) = focus_world {
        let old_zoom = state.view.camera.zoom;
        state.view.camera.zoom_by_clamped(
            factor,
            state.options.camera_zoom_min,
            state.options.camera_zoom_max,
        );
        let new_zoom = state.view.camera.zoom;
        // Kamera-Position korrigieren, damit focus_world an gleicher Stelle bleibt
        let scale = old_zoom / new_zoom;
        state.view.camera.position = focus + (state.view.camera.position - focus) * scale;
    } else {
        state.view.camera.zoom_by_clamped(
            factor,
            state.options.camera_zoom_min,
            state.options.camera_zoom_max,
        );
    }
}

/// Zentriert die Kamera auf die gesamte Karte.
///
/// Wenn eine Background-Map geladen ist, werden deren `WorldBounds` als
/// Referenzrahmen verwendet (zeigt die gesamte Hintergrundkarte).
/// Fallback: Bounding Box aller Nodes in der RoadMap.
/// Keine Operation wenn weder Background-Map noch Nodes vorhanden sind.
pub fn center_on_road_map(state: &mut AppState, road_map: &RoadMap) {
    use crate::core::Camera2D;

    // Prioritaet: BackgroundMap-Bounds zeigen die gesamte Karte
    if let Some(bg_map) = state.view.background_map.as_deref() {
        let bounds = bg_map.world_bounds();
        let center_x = (bounds.min_x + bounds.max_x) / 2.0;
        let center_y = (bounds.min_z + bounds.max_z) / 2.0;
        state
            .view
            .camera
            .look_at(glam::Vec2::new(center_x, center_y));

        let width = bounds.max_x - bounds.min_x;
        let height = bounds.max_z - bounds.min_z;
        let max_extent = width.max(height);
        state.view.camera.zoom = Camera2D::BASE_WORLD_EXTENT / (max_extent / 2.0);
        state
            .view
            .camera
            .clamp_zoom(state.options.camera_zoom_min, state.options.camera_zoom_max);

        log::info!(
            "Zoom-to-Map (BackgroundMap): ({:.1}, {:.1}) bis ({:.1}, {:.1}), center: ({:.1}, {:.1}), zoom: {:.2}",
            bounds.min_x,
            bounds.min_z,
            bounds.max_x,
            bounds.max_z,
            center_x,
            center_y,
            state.view.camera.zoom
        );
        return;
    }

    // Fallback: Node-Bounds
    if road_map.nodes().is_empty() {
        return;
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for node in road_map.nodes().values() {
        min_x = min_x.min(node.position.x);
        max_x = max_x.max(node.position.x);
        min_y = min_y.min(node.position.y);
        max_y = max_y.max(node.position.y);
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    state
        .view
        .camera
        .look_at(glam::Vec2::new(center_x, center_y));

    let width = max_x - min_x;
    let height = max_y - min_y;
    let max_extent = width.max(height);
    state.view.camera.zoom = Camera2D::BASE_WORLD_EXTENT / (max_extent / 2.0);
    state
        .view
        .camera
        .clamp_zoom(state.options.camera_zoom_min, state.options.camera_zoom_max);

    log::info!(
        "Zoom-to-Map (Nodes): ({:.1}, {:.1}) bis ({:.1}, {:.1}), center: ({:.1}, {:.1}), zoom: {:.2}",
        min_x,
        min_y,
        max_x,
        max_y,
        center_x,
        center_y,
        state.view.camera.zoom
    );
}

/// Zoomt die Kamera auf die Bounding Box der aktuellen Selektion.
///
/// Berechnet den Mittelpunkt und passenden Zoom-Level fuer alle selektierten Nodes.
/// Keine Operation wenn die Selektion leer ist oder keine selektierten Nodes in der
/// RoadMap vorhanden sind.
pub fn zoom_to_selection_bounds(state: &mut AppState, road_map: &RoadMap) {
    let selected = state.selection.selected_node_ids.clone();
    if selected.is_empty() {
        return;
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut count = 0u32;

    for &id in selected.iter() {
        if let Some(node) = road_map.node(id) {
            min_x = min_x.min(node.position.x);
            max_x = max_x.max(node.position.x);
            min_y = min_y.min(node.position.y);
            max_y = max_y.max(node.position.y);
            count += 1;
        }
    }
    if count == 0 {
        return;
    }

    let center = glam::Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    state.view.camera.look_at(center);

    // Mindestgroesse 1.0 verhindert Division-durch-null bei Einzelnode-Selektion
    let extent = (max_x - min_x).max(max_y - min_y).max(1.0);
    use crate::core::Camera2D;
    state.view.camera.zoom = Camera2D::BASE_WORLD_EXTENT / (extent / 2.0) * SELECTION_ZOOM_PADDING;
    state
        .view
        .camera
        .clamp_zoom(state.options.camera_zoom_min, state.options.camera_zoom_max);
}

/// Zoomt die Kamera passend: auf die Selektion wenn vorhanden, sonst auf die gesamte RoadMap.
pub fn zoom_to_fit(state: &mut AppState, road_map: &RoadMap) {
    if !state.selection.selected_node_ids.is_empty() {
        zoom_to_selection_bounds(state, road_map);
    } else {
        center_on_road_map(state, road_map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_camera_sets_default_position_and_zoom() {
        let mut state = AppState::new();
        state.view.camera.look_at(glam::Vec2::new(100.0, 200.0));
        state.view.camera.zoom_by(5.0);

        reset_camera(&mut state);

        assert_eq!(state.view.camera.position, glam::Vec2::ZERO);
        assert_eq!(state.view.camera.zoom, 1.0);
    }

    #[test]
    fn zoom_in_increases_zoom() {
        let mut state = AppState::new();
        let before = state.view.camera.zoom;

        zoom_in(&mut state);

        assert!(state.view.camera.zoom > before);
    }

    #[test]
    fn zoom_out_decreases_zoom() {
        let mut state = AppState::new();
        let before = state.view.camera.zoom;

        zoom_out(&mut state);

        assert!(state.view.camera.zoom < before);
    }

    #[test]
    fn zoom_in_then_out_returns_to_original() {
        let mut state = AppState::new();
        let original = state.view.camera.zoom;

        zoom_in(&mut state);
        zoom_out(&mut state);

        assert!((state.view.camera.zoom - original).abs() < 1e-5);
    }

    #[test]
    fn pan_moves_camera_position() {
        let mut state = AppState::new();

        pan(&mut state, glam::Vec2::new(10.0, -5.0));

        assert_eq!(state.view.camera.position, glam::Vec2::new(10.0, -5.0));
    }

    #[test]
    fn zoom_by_factor_applies_custom_factor() {
        let mut state = AppState::new();

        zoom_towards(&mut state, 2.0, None);

        assert!((state.view.camera.zoom - 2.0).abs() < 1e-5);
    }

    #[test]
    fn zoom_towards_point_keeps_focus_stable() {
        let mut state = AppState::new();
        let focus = glam::Vec2::new(100.0, 50.0);

        zoom_towards(&mut state, 2.0, Some(focus));

        // Nach Zoom: Kamera muss sich zum Fokuspunkt hin bewegt haben
        assert!(state.view.camera.position.x > 0.0);
        assert!(state.view.camera.position.y > 0.0);
    }
}
