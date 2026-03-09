//! Gemeinsame Icon-Konstanten und -Hilfsfunktionen fuer die UI.

use crate::app::{AppState, ConnectionDirection, ConnectionPriority};
use egui::{Image, ImageSource, Vec2};

/// Standard-Icon-Groesse fuer Tool-Buttons.
pub const ICON_SIZE: f32 = 40.0;

/// Erstellt ein `Image`-Widget aus einer `ImageSource`.
pub fn svg_icon(source: ImageSource<'_>, size: f32) -> Image<'_> {
    Image::new(source).fit_to_exact_size(Vec2::splat(size))
}

/// Liefert die `ImageSource` fuer ein Route-Tool anhand seines Index.
pub fn route_tool_icon(idx: usize) -> ImageSource<'static> {
    match idx {
        0 => egui::include_image!("../../assets/icons/icon_straight_line.svg"),
        1 => egui::include_image!("../../assets/icons/icon_bezier_quadratic.svg"),
        2 => egui::include_image!("../../assets/icons/icon_bezier_cubic.svg"),
        3 => egui::include_image!("../../assets/icons/icon_spline.svg"),
        4 => egui::include_image!("../../assets/icons/icon_bypass.svg"),
        5 => egui::include_image!("../../assets/icons/icon_constraint_route.svg"),
        6 => egui::include_image!("../../assets/icons/icon_parking.svg"),
        7 => egui::include_image!("../../assets/icons/icon_field_boundary.svg"),
        8 => egui::include_image!("../../assets/icons/icon_route_offset.svg"),
        _ => egui::include_image!("../../assets/icons/icon_straight_line.svg"),
    }
}

/// Wandelt eine RGBA-Farbe im Bereich 0..=1 in `egui::Color32` um.
pub(crate) fn color32_from_rgba(color: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

/// Liefert die Standardfarbe fuer Funktions-Icons anhand der Default-Prioritaet.
pub(crate) fn function_icon_color(state: &AppState) -> egui::Color32 {
    match state.editor.default_priority {
        ConnectionPriority::Regular => color32_from_rgba(state.options.connection_color_regular),
        ConnectionPriority::SubPriority => color32_from_rgba(state.options.node_color_subprio),
    }
}

/// Liefert die Akzentfarbe fuer aktive Icons anhand der Default-Richtung.
pub(crate) fn accent_icon_color(state: &AppState) -> egui::Color32 {
    match state.editor.default_direction {
        ConnectionDirection::Regular => color32_from_rgba(state.options.connection_color_regular),
        ConnectionDirection::Dual => color32_from_rgba(state.options.connection_color_dual),
        ConnectionDirection::Reverse => color32_from_rgba(state.options.connection_color_reverse),
    }
}
