use crate::app::{ConnectionDirection, ConnectionPriority};

const OPTION_ICON_SIZE: egui::Vec2 = egui::Vec2::new(32.0, 32.0);

fn direction_icon(direction: ConnectionDirection) -> egui::ImageSource<'static> {
    match direction {
        ConnectionDirection::Regular => {
            egui::include_image!("../../../assets/icon_direction_regular.svg")
        }
        ConnectionDirection::Dual => {
            egui::include_image!("../../../assets/icon_direction_dual.svg")
        }
        ConnectionDirection::Reverse => {
            egui::include_image!("../../../assets/icon_direction_reverse.svg")
        }
    }
}

fn priority_icon(priority: ConnectionPriority) -> egui::ImageSource<'static> {
    match priority {
        ConnectionPriority::Regular => {
            egui::include_image!("../../../assets/icon_priority_main.svg")
        }
        ConnectionPriority::SubPriority => {
            egui::include_image!("../../../assets/icon_priority_side.svg")
        }
    }
}

fn selectable_icon(
    ui: &mut egui::Ui,
    icon: egui::ImageSource<'static>,
    tooltip: &'static str,
    selected: bool,
) -> egui::Response {
    let tint = if selected {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_white_alpha(100)
    };
    let image = egui::Image::new(icon)
        .fit_to_exact_size(OPTION_ICON_SIZE)
        .tint(tint);
    let mut button = egui::Button::image(image);
    if selected {
        button = button.fill(ui.visuals().selection.bg_fill);
    }
    ui.add(button).on_hover_text(tooltip)
}

fn render_direction_icon_selector_inner(
    ui: &mut egui::Ui,
    selected: &mut ConnectionDirection,
    layout_vertical: bool,
    id_suffix: &str,
) {
    ui.push_id(format!("dir_icon_selector_{id_suffix}"), |ui| {
        let render = |ui: &mut egui::Ui| {
            for (value, tooltip) in [
                (ConnectionDirection::Regular, "Einbahn vorwaerts"),
                (ConnectionDirection::Dual, "Zweirichtungsverkehr"),
                (ConnectionDirection::Reverse, "Einbahn rueckwaerts"),
            ] {
                if selectable_icon(ui, direction_icon(value), tooltip, *selected == value).clicked()
                {
                    *selected = value;
                }
            }
        };

        if layout_vertical {
            ui.vertical(render);
        } else {
            ui.horizontal(render);
        }
    });
}

fn render_priority_icon_selector_inner(
    ui: &mut egui::Ui,
    selected: &mut ConnectionPriority,
    layout_vertical: bool,
    id_suffix: &str,
) {
    ui.push_id(format!("prio_icon_selector_{id_suffix}"), |ui| {
        let render = |ui: &mut egui::Ui| {
            for (value, tooltip) in [
                (ConnectionPriority::Regular, "Hauptstrasse"),
                (ConnectionPriority::SubPriority, "Nebenstrasse"),
            ] {
                if selectable_icon(ui, priority_icon(value), tooltip, *selected == value).clicked()
                {
                    *selected = value;
                }
            }
        };

        if layout_vertical {
            ui.vertical(render);
        } else {
            ui.horizontal(render);
        }
    });
}

pub fn render_direction_icon_selector(
    ui: &mut egui::Ui,
    selected: &mut ConnectionDirection,
    id_suffix: &str,
) {
    render_direction_icon_selector_inner(ui, selected, false, id_suffix);
}

pub fn render_direction_icon_selector_vertical(
    ui: &mut egui::Ui,
    selected: &mut ConnectionDirection,
    id_suffix: &str,
) {
    render_direction_icon_selector_inner(ui, selected, true, id_suffix);
}

pub fn render_priority_icon_selector(
    ui: &mut egui::Ui,
    selected: &mut ConnectionPriority,
    id_suffix: &str,
) {
    render_priority_icon_selector_inner(ui, selected, false, id_suffix);
}

pub fn render_priority_icon_selector_vertical(
    ui: &mut egui::Ui,
    selected: &mut ConnectionPriority,
    id_suffix: &str,
) {
    render_priority_icon_selector_inner(ui, selected, true, id_suffix);
}
