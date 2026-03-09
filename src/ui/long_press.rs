//! Wiederverwendbares Long-Press-Dropdown-Widget fuer Icon-Buttons.

use crate::ui::icons::{ICON_SIZE, svg_icon};

/// Long-Press-Status einer Button-Gruppe.
#[derive(Debug, Clone)]
pub struct LongPressState {
    /// Startzeitpunkt des gedrueckten Buttons (egui-Zeit in Sekunden).
    pub press_start: Option<f64>,
    /// Ob das Auswahl-Popup aktuell offen ist.
    pub popup_open: bool,
    /// Position des Popups im Screen-Space.
    pub popup_pos: Option<egui::Pos2>,
}

impl Default for LongPressState {
    fn default() -> Self {
        Self {
            press_start: None,
            popup_open: false,
            popup_pos: None,
        }
    }
}

/// Ein auswählbares Item innerhalb einer Long-Press-Gruppe.
#[derive(Clone)]
pub struct LongPressItem<T: Clone> {
    /// Icon fuer das Item.
    pub icon: egui::ImageSource<'static>,
    /// Tooltip fuer Hover-Anzeige.
    pub tooltip: &'static str,
    /// Rueckgabewert bei Auswahl.
    pub value: T,
}

/// Definiert eine Long-Press-Gruppe mit mehreren auswählbaren Items.
#[derive(Clone)]
pub struct LongPressGroup<T: Clone + PartialEq> {
    /// Eindeutige ID fuer egui.
    pub id: &'static str,
    /// Anzeigename der Gruppe.
    pub label: &'static str,
    /// Alle auswählbaren Items.
    pub items: Vec<LongPressItem<T>>,
}

/// Rendert einen Long-Press-Button mit optionalem Auswahl-Popup.
///
/// Kurzer Klick aktiviert das aktuell angezeigte Item.
/// Long-Press (>= 1s) oeffnet ein Popup mit allen Items.
pub fn render_long_press_button<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    group: &LongPressGroup<T>,
    active_value: &T,
    lp_state: &mut LongPressState,
) -> Option<T> {
    let active_item = group
        .items
        .iter()
        .find(|item| &item.value == active_value)
        .or_else(|| group.items.first())?;

    let icon = svg_icon(active_item.icon.clone(), ICON_SIZE).tint(active_icon_color);

    let response = ui
        .add(egui::Button::image(icon).selected(true))
        .on_hover_text(group.label);

    paint_dropdown_arrow(ui, &response);

    let now = ui.ctx().input(|i| i.time);

    if response.is_pointer_button_down_on() {
        if lp_state.press_start.is_none() {
            lp_state.press_start = Some(now);
        }

        if lp_state.press_start.is_some() && !lp_state.popup_open {
            ui.ctx().request_repaint();
        }

        if let Some(start) = lp_state.press_start {
            if now - start >= 1.0 && !lp_state.popup_open {
                lp_state.popup_open = true;
                lp_state.popup_pos = Some(response.rect.right_top());
                lp_state.press_start = None;
            }
        }
    } else if let Some(start) = lp_state.press_start.take() {
        let elapsed = now - start;
        if elapsed < 1.0 && !lp_state.popup_open && response.clicked() {
            return Some(active_item.value.clone());
        }
    }

    if lp_state.popup_open {
        if lp_state.popup_pos.is_none() {
            lp_state.popup_pos = Some(response.rect.right_top());
        }

        return render_popup(
            ui.ctx(),
            group,
            active_value,
            icon_color,
            active_icon_color,
            lp_state,
        );
    }

    None
}

/// Rendert das Long-Press-Popup neben dem Button.
pub fn render_popup<T: Clone + PartialEq>(
    ctx: &egui::Context,
    group: &LongPressGroup<T>,
    active_value: &T,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    lp_state: &mut LongPressState,
) -> Option<T> {
    let popup_pos = lp_state.popup_pos.unwrap_or(egui::pos2(0.0, 0.0));

    let area_response = egui::Area::new(egui::Id::new(("lp_popup", group.id)))
        .order(egui::Order::Foreground)
        .fixed_pos(popup_pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(group.label);
                    ui.separator();

                    let mut selected = None;
                    for item in &group.items {
                        let is_active = &item.value == active_value;
                        let tint = if is_active {
                            active_icon_color
                        } else {
                            icon_color
                        };
                        let icon = svg_icon(item.icon.clone(), ICON_SIZE).tint(tint);

                        if ui
                            .add(egui::Button::image(icon).selected(is_active))
                            .on_hover_text(item.tooltip)
                            .clicked()
                        {
                            selected = Some(item.value.clone());
                        }
                    }

                    selected
                })
                .inner
            })
            .inner
        });

    let selected = area_response.inner;
    if selected.is_some() {
        lp_state.popup_open = false;
        lp_state.popup_pos = None;
        return selected;
    }

    let popup_rect = area_response.response.rect;
    let clicked_outside = ctx.input(|i| {
        if !i.pointer.any_click() {
            return false;
        }
        let pointer_pos = i.pointer.interact_pos().or(i.pointer.hover_pos());
        pointer_pos
            .map(|pos| !popup_rect.contains(pos))
            .unwrap_or(false)
    });

    if clicked_outside {
        lp_state.popup_open = false;
        lp_state.popup_pos = None;
    }

    None
}

/// Zeichnet einen kleinen Dropdown-Pfeil in die untere rechte Ecke des Buttons.
pub fn paint_dropdown_arrow(ui: &egui::Ui, response: &egui::Response) {
    let rect = response.rect;
    let size = 5.0;
    let inset = 2.0;

    let p1 = egui::pos2(rect.right() - size - inset, rect.bottom() - size - inset);
    let p2 = egui::pos2(rect.right() - inset, rect.bottom() - size - inset);
    let p3 = egui::pos2(rect.right() - (size * 0.5) - inset, rect.bottom() - inset);

    ui.painter().add(egui::Shape::convex_polygon(
        vec![p1, p2, p3],
        egui::Color32::WHITE,
        egui::Stroke::NONE,
    ));
}
