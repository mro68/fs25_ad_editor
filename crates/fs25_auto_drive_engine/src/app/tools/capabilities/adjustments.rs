//! Capabilities fuer nicht-klickbasierte Tool-Anpassungen.

/// Optionale Capability fuer Alt+Scroll-basierte Tool-Rotation.
pub trait RouteToolRotate {
    /// Verarbeitet Alt+Scroll-Rotation.
    fn on_scroll_rotate(&mut self, delta: f32);
}

/// Optionale Capability fuer Segment- und Node-Count-Shortcuts.
pub trait RouteToolSegmentAdjustments {
    /// Erhoeht die Anzahl der Nodes um 1.
    fn increase_node_count(&mut self);

    /// Verringert die Anzahl der Nodes um 1.
    fn decrease_node_count(&mut self);

    /// Erhoeht die Segmentlaenge um den tool-spezifischen Schritt.
    fn increase_segment_length(&mut self);

    /// Verringert die Segmentlaenge um den tool-spezifischen Schritt.
    fn decrease_segment_length(&mut self);
}
