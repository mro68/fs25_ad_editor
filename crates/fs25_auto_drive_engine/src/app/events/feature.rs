//! Interne Feature-Klassifikation fuer Intent- und Command-Dispatch.

/// Gemeinsame Feature-Slices fuer `AppIntent` und `AppCommand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppEventFeature {
    /// Datei- und XML-I/O inklusive Heightmap-Speicherpfadlogik.
    FileIo,
    /// Kamera-, Viewport- und Background-/Overview-Operationen.
    View,
    /// Selektion, Move/Rotate-Lifecycles und Gruppen-Picks.
    Selection,
    /// Node-/Connection-Editing, Marker, Copy/Paste und Editing-Extras.
    Editing,
    /// Route-Tool-Interaktionen und Tool-spezifische Shortcuts.
    RouteTool,
    /// Gruppen- und Segment-Operationen inklusive Edit-Flow.
    Group,
    /// Dialoge, Options- und Overlay-State.
    Dialog,
    /// Undo/Redo-History.
    History,
}
