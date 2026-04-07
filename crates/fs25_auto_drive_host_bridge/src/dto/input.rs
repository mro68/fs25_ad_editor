//! Viewport-Input-Event-DTOs fuer den kanonischen Input-Vertrag der Host-Bridge.

use serde::{Deserialize, Serialize};

/// Stabile Pointer-Button-Klassifikation fuer den Viewport-Input-Vertrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostPointerButton {
    /// Primaere Pointer-Taste.
    Primary,
    /// Mittlere Pointer-Taste.
    Middle,
    /// Sekundaere Pointer-Taste.
    Secondary,
}

/// Stabile Tap-Klassifikation fuer den Viewport-Input-Vertrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTapKind {
    /// Einfacher Tap bzw. einzelner Klick.
    Single,
    /// Doppelter Tap bzw. Doppelklick.
    Double,
}

/// Host-neutrale Modifiers fuer Viewport-Input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInputModifiers {
    /// Shift-Modifizierer.
    pub shift: bool,
    /// Alt-/Option-Modifizierer.
    pub alt: bool,
    /// Plattformneutraler Command-Modifizierer (`Ctrl` bzw. `Cmd`).
    pub command: bool,
}

/// Batch von host-neutralen Viewport-Input-Events fuer die kanonische Session-Surface.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HostViewportInputBatch {
    /// In Reihenfolge empfangene Viewport-Input-Events.
    pub events: Vec<HostViewportInputEvent>,
}

/// Kleines host-neutrales Viewport-Input-Event fuer die Bridge-Surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostViewportInputEvent {
    /// Aktualisiert die bekannte Viewport-Groesse der Session.
    Resize {
        /// Neue Viewport-Groesse in Pixeln [width, height].
        size_px: [f32; 2],
    },
    /// Einzelner Tap bzw. Klick an Bildschirmposition.
    Tap {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Art des Taps.
        tap_kind: HostTapKind,
        /// Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: [f32; 2],
        /// Aktive Modifiers zum Zeitpunkt des Taps.
        modifiers: HostInputModifiers,
    },
    /// Start eines Drags an Bildschirmposition.
    DragStart {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: [f32; 2],
        /// Aktive Modifiers zum Zeitpunkt des Starts.
        modifiers: HostInputModifiers,
    },
    /// Delta-Update eines laufenden Drags.
    DragUpdate {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Aktuelle Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: [f32; 2],
        /// Delta in Bildschirm-Pixeln seit dem letzten Update.
        delta_px: [f32; 2],
    },
    /// Ende eines laufenden Drags.
    DragEnd {
        /// Verwendeter Pointer-Button.
        button: HostPointerButton,
        /// Optionale finale Bildschirmposition relativ zum Viewport.
        screen_pos: Option<[f32; 2]>,
    },
    /// Scroll-Ereignis an optionaler Bildschirmposition.
    Scroll {
        /// Optionale Bildschirmposition in Pixeln relativ zum Viewport.
        screen_pos: Option<[f32; 2]>,
        /// Geglaettete Scroll-Differenz fuer Zoom-Interpretation.
        smooth_delta_y: f32,
        /// Rohes Scroll-Delta fuer spaetere Tick-basierte Erweiterungen.
        raw_delta_y: f32,
        /// Aktive Modifiers zum Zeitpunkt des Scrollens.
        modifiers: HostInputModifiers,
    },
}
