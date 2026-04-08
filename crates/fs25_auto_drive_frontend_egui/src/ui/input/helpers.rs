//! Hilfsfunktionen fuer Viewport-Input-Konvertierungen.

use crate::app::Camera2D;
use fs25_auto_drive_host_bridge::{HostInputModifiers, HostPointerButton, HostTapKind};

/// Konvertiert egui-Modifier-Zustand in Bridge-Modifiers.
pub(crate) fn host_modifiers(modifiers: egui::Modifiers) -> HostInputModifiers {
    HostInputModifiers {
        shift: modifiers.shift,
        alt: modifiers.alt,
        command: modifiers.command || modifiers.ctrl,
    }
}

/// Rechnet eine absolute Bildschirmposition in eine Viewport-lokale Position um.
pub(crate) fn to_viewport_screen_pos(
    pointer_pos: egui::Pos2,
    response: &egui::Response,
) -> [f32; 2] {
    let local = pointer_pos - response.rect.min;
    [local.x, local.y]
}

/// Mappt einen egui-Pointer-Button auf einen optionalen Bridge-Button.
pub(crate) fn host_pointer_button(button: egui::PointerButton) -> Option<HostPointerButton> {
    match button {
        egui::PointerButton::Primary => Some(HostPointerButton::Primary),
        egui::PointerButton::Middle => Some(HostPointerButton::Middle),
        egui::PointerButton::Secondary => Some(HostPointerButton::Secondary),
        _ => None,
    }
}

/// Mappt einen egui-Doppelklick-Bool auf einen Bridge-Tap-Kind.
pub(crate) fn host_tap_kind(is_double: bool) -> HostTapKind {
    if is_double {
        HostTapKind::Double
    } else {
        HostTapKind::Single
    }
}

/// Rechnet eine Bildschirmposition in Weltkoordinaten um.
pub(crate) fn screen_pos_to_world(
    pointer_pos: egui::Pos2,
    response: &egui::Response,
    viewport_size: [f32; 2],
    camera: &Camera2D,
) -> glam::Vec2 {
    let local = pointer_pos - response.rect.min;
    camera.screen_to_world(
        glam::Vec2::new(local.x, local.y),
        glam::Vec2::new(viewport_size[0], viewport_size[1]),
    )
}
