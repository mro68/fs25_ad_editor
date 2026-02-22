//! Vertex-Generierung für Connection-Linien und Pfeilspitzen.

use super::super::types::ConnectionVertex;
use crate::shared::EditorOptions;
use crate::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Bestimmt die Farbe einer Verbindung anhand von Richtung und Priorität.
pub(super) fn connection_color(
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    options: &EditorOptions,
) -> [f32; 4] {
    let base = match direction {
        ConnectionDirection::Regular => options.connection_color_regular,
        ConnectionDirection::Dual => options.connection_color_dual,
        ConnectionDirection::Reverse => options.connection_color_reverse,
    };

    match priority {
        ConnectionPriority::Regular => base,
        ConnectionPriority::SubPriority => [
            (base[0] + 1.0) * 0.5,
            (base[1] + 1.0) * 0.5,
            (base[2] + 1.0) * 0.5,
            base[3],
        ],
    }
}

/// Erzeugt ein Quad (2 Dreiecke) für ein Liniensegment mit gegebener Breite.
pub(super) fn push_line_quad(
    vertices: &mut Vec<ConnectionVertex>,
    start: Vec2,
    end: Vec2,
    thickness: f32,
    color: [f32; 4],
) {
    let dir = (end - start).normalize();
    let perp = Vec2::new(-dir.y, dir.x) * (thickness * 0.5);

    let v0 = start + perp;
    let v1 = start - perp;
    let v2 = end + perp;
    let v3 = end - perp;

    vertices.push(ConnectionVertex::new([v0.x, v0.y], color));
    vertices.push(ConnectionVertex::new([v1.x, v1.y], color));
    vertices.push(ConnectionVertex::new([v2.x, v2.y], color));

    vertices.push(ConnectionVertex::new([v2.x, v2.y], color));
    vertices.push(ConnectionVertex::new([v1.x, v1.y], color));
    vertices.push(ConnectionVertex::new([v3.x, v3.y], color));
}

/// Erzeugt ein Dreieck als Richtungspfeil an der gegebenen Position.
pub(super) fn push_arrow(
    vertices: &mut Vec<ConnectionVertex>,
    center: Vec2,
    direction: Vec2,
    length: f32,
    width: f32,
    color: [f32; 4],
) {
    let dir = direction.normalize();
    let perp = Vec2::new(-dir.y, dir.x);

    let tip = center + dir * (length * 0.5);
    let base = center - dir * (length * 0.5);
    let left = base + perp * (width * 0.5);
    let right = base - perp * (width * 0.5);

    vertices.push(ConnectionVertex::new([tip.x, tip.y], color));
    vertices.push(ConnectionVertex::new([left.x, left.y], color));
    vertices.push(ConnectionVertex::new([right.x, right.y], color));
}
