//! Geteilter Icon-Button mit Dropdown-Popup fuer Werkzeuggruppen.
//!
//! Klick linke Haelfte: aktives Item aktivieren.
//! Klick rechte Haelfte (Pfeil-Bereich): Auswahl-Popup sofort oeffnen.

use crate::ui::icons::{svg_icon, ICON_SIZE};

/// Zustand eines geteilten Icon-Buttons (Dropdown-Gruppe).
#[derive(Debug, Clone, Default)]
pub struct LongPressState {
    /// Ob das Auswahl-Popup aktuell offen ist.
    pub popup_open: bool,
    /// Position des Popups im Screen-Space.
    pub popup_pos: Option<egui::Pos2>,
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
pub struct LongPressGroup<'a, T: Clone + PartialEq> {
    /// Eindeutige ID fuer egui.
    pub id: &'static str,
    /// Anzeigename der Gruppe.
    pub label: &'static str,
    /// Alle auswählbaren Items.
    pub items: &'a [LongPressItem<T>],
}

/// Rendert einen geteilten Icon-Button fuer eine Gruppe auswaehlbarer Items.
///
/// - `display_value`: Welches Icon gezeigt wird und im Popup selektiert ist (letztes benutztes Item).
/// - `is_button_active`: Ob der Button in Akzentfarbe aufleuchtet (nur wenn dieses Tool gerade aktiv ist).
/// - Klick links: aktiviert das angezeigte Item.
/// - Klick rechts (Pfeil, nur ab 2 Items): oeffnet das Auswahl-Popup sofort.
pub fn render_long_press_button<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    group: &LongPressGroup<'_, T>,
    display_value: &T,
    is_button_active: bool,
    lp_state: &mut LongPressState,
) -> Option<T> {
    let display_item = group
        .items
        .iter()
        .find(|item| &item.value == display_value)
        .or_else(|| group.items.first())?;

    let button_tint = if is_button_active {
        active_icon_color
    } else {
        icon_color
    };

    let has_multiple = group.items.len() > 1;

    let icon = svg_icon(display_item.icon.clone(), ICON_SIZE).tint(button_tint);
    let response = ui
        .add(egui::Button::image(icon).selected(is_button_active))
        .on_hover_text(display_item.tooltip);

    if has_multiple {
        paint_dropdown_arrow(ui, &response);
    }

    if response.clicked() {
        if has_multiple {
            // Split-Klick: rechte 40 % des Buttons oeffnen das Auswahl-Popup sofort.
            let click_pos = ui.ctx().input(|i| i.pointer.interact_pos());
            if let Some(pos) = click_pos {
                let split_x = response.rect.left() + response.rect.width() * 0.6;
                if pos.x >= split_x {
                    lp_state.popup_open = true;
                    lp_state.popup_pos = Some(response.rect.right_top());
                    return None;
                }
            }
        }
        // Linke Haelfte (oder Single-Item): aktuelles Item aktivieren.
        return Some(display_item.value.clone());
    }

    if lp_state.popup_open {
        if lp_state.popup_pos.is_none() {
            lp_state.popup_pos = Some(response.rect.right_top());
        }
        return render_popup(
            ui.ctx(),
            group,
            display_value,
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
    group: &LongPressGroup<'_, T>,
    selected_value: &T,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    lp_state: &mut LongPressState,
) -> Option<T> {
    let popup_pos = lp_state.popup_pos.unwrap_or(egui::pos2(0.0, 0.0));

    let area_response = egui::Area::new(egui::Id::new(("lp_popup", group.id)))
        .order(egui::Order::Foreground)
        .fixed_pos(popup_pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(group.label);
                        ui.separator();

                        let mut selected = None;
                        for item in group.items {
                            let is_active = &item.value == selected_value;
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

/// Zeichnet einen Dropdown-Pfeil in die untere rechte Ecke des Buttons.
pub fn paint_dropdown_arrow(ui: &egui::Ui, response: &egui::Response) {
    let rect = response.rect;
    let size = 8.0;
    let inset = 3.0;

    let p1 = egui::pos2(rect.right() - size - inset, rect.bottom() - size - inset);
    let p2 = egui::pos2(rect.right() - inset, rect.bottom() - size - inset);
    let p3 = egui::pos2(rect.right() - (size * 0.5) - inset, rect.bottom() - inset);

    ui.painter().add(egui::Shape::convex_polygon(
        vec![p1, p2, p3],
        egui::Color32::WHITE,
        egui::Stroke::NONE,
    ));
}
