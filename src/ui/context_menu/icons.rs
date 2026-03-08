//! Icon- und Farbhilfs-Funktionen für das Kontextmenü.

use crate::app::{ConnectionDirection, ConnectionPriority};
use crate::shared::EditorOptions;

use super::commands::CommandId;

/// Icon-Größe für SVG-Icons im Kontextmenü.
pub(super) const CM_ICON_SIZE: egui::Vec2 = egui::Vec2::new(16.0, 16.0);
pub(super) const CM_CHOICE_ICON_SIZE: egui::Vec2 = egui::Vec2::new(32.0, 32.0);

pub(super) fn is_direction_or_priority(id: CommandId) -> bool {
    matches!(
        id,
        CommandId::DirectionRegular
            | CommandId::DirectionDual
            | CommandId::DirectionReverse
            | CommandId::PriorityRegular
            | CommandId::PrioritySub
    )
}

pub(super) fn direction_or_priority_tooltip(id: CommandId) -> &'static str {
    match id {
        CommandId::DirectionRegular => "Einbahn vorwaerts",
        CommandId::DirectionDual => "Zweirichtungsverkehr",
        CommandId::DirectionReverse => "Einbahn rueckwaerts",
        CommandId::PriorityRegular => "Hauptstrasse",
        CommandId::PrioritySub => "Nebenstrasse",
        _ => "",
    }
}

fn color32_from_rgba(color: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn function_icon_color(options: &EditorOptions, priority: ConnectionPriority) -> egui::Color32 {
    match priority {
        ConnectionPriority::Regular => color32_from_rgba(options.connection_color_regular),
        ConnectionPriority::SubPriority => color32_from_rgba(options.node_color_subprio),
    }
}

fn direction_icon_color(options: &EditorOptions, direction: ConnectionDirection) -> egui::Color32 {
    match direction {
        ConnectionDirection::Regular => color32_from_rgba(options.connection_color_regular),
        ConnectionDirection::Dual => color32_from_rgba(options.connection_color_dual),
        ConnectionDirection::Reverse => color32_from_rgba(options.connection_color_reverse),
    }
}

/// Gibt das SVG-Icon für einen Command zurück (dieselben wie in der Toolbar).
pub(super) fn command_icon(
    id: CommandId,
    options: &EditorOptions,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
) -> Option<egui::Image<'static>> {
    let source: egui::ImageSource<'static> = match id {
        CommandId::SetToolSelect => {
            egui::include_image!("../../../assets/icon_select_node.svg")
        }
        CommandId::SetToolConnect => {
            egui::include_image!("../../../assets/icon_connect.svg")
        }
        CommandId::SetToolAddNode => {
            egui::include_image!("../../../assets/icon_add_node.svg")
        }
        CommandId::SetToolRouteStraight | CommandId::RouteStraight => {
            egui::include_image!("../../../assets/new/minus.svg")
        }
        CommandId::SetToolRouteConstraint | CommandId::RouteConstraint => {
            egui::include_image!("../../../assets/icon_constraint_route.svg")
        }
        CommandId::SetToolRouteQuadratic | CommandId::RouteQuadratic => {
            egui::include_image!("../../../assets/icon_bezier_quadratic.svg")
        }
        CommandId::SetToolRouteCubic | CommandId::RouteCubic => {
            egui::include_image!("../../../assets/icon_bezier_cubic.svg")
        }
        CommandId::SetToolFieldBoundary => {
            egui::include_image!("../../../assets/icon_field_boundary.svg")
        }
        CommandId::DirectionRegular => {
            egui::include_image!("../../../assets/icon_direction_regular.svg")
        }
        CommandId::DirectionDual => {
            egui::include_image!("../../../assets/icon_direction_dual.svg")
        }
        CommandId::DirectionReverse => {
            egui::include_image!("../../../assets/icon_direction_reverse.svg")
        }
        CommandId::PriorityRegular => {
            egui::include_image!("../../../assets/icon_priority_main.svg")
        }
        CommandId::PrioritySub => {
            egui::include_image!("../../../assets/icon_priority_side.svg")
        }
        _ => return None,
    };

    let tint = match id {
        CommandId::DirectionRegular => direction_icon_color(options, ConnectionDirection::Regular),
        CommandId::DirectionDual => direction_icon_color(options, ConnectionDirection::Dual),
        CommandId::DirectionReverse => direction_icon_color(options, ConnectionDirection::Reverse),
        CommandId::PriorityRegular => function_icon_color(options, ConnectionPriority::Regular),
        CommandId::PrioritySub => function_icon_color(options, ConnectionPriority::SubPriority),
        _ => {
            let _accent = direction_icon_color(options, default_direction);
            function_icon_color(options, default_priority)
        }
    };

    Some(
        egui::Image::new(source)
            .fit_to_exact_size(CM_ICON_SIZE)
            .tint(tint),
    )
}
