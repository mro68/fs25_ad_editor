//! Gemeinsame Icon-Konstanten und -Hilfsfunktionen fuer die UI.

use egui::{Image, ImageSource, Vec2};

/// Standard-Icon-Groesse fuer Tool-Buttons.
pub const ICON_SIZE: f32 = 20.0;

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
