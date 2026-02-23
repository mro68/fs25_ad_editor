//! Toolbar für Editor-Werkzeugauswahl.

use crate::app::{AppIntent, AppState, EditorTool};

// ── SVG-Icon-Konstanten (compile-time eingebettet) ──────────────

const ICON_SIZE: egui::Vec2 = egui::Vec2::new(20.0, 20.0);
const ICON_SIZE_DROPDOWN: egui::Vec2 = egui::Vec2::new(18.0, 18.0);

/// Erstellt ein `egui::Image` aus einer `ImageSource` in der gewünschten Größe.
fn svg_icon(source: egui::ImageSource<'_>, size: egui::Vec2) -> egui::Image<'_> {
    egui::Image::new(source).fit_to_exact_size(size)
}

/// Rendert die Toolbar und gibt erzeugte Events zurück.
pub fn render_toolbar(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let active = state.editor.active_tool;

    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Werkzeug:");
            ui.separator();

            // ── Select-Button mit SVG-Icon ──
            let select_icon = svg_icon(
                egui::include_image!("../../assets/icon_select_node.svg"),
                ICON_SIZE,
            );
            let select_btn = egui::Button::image_and_text(select_icon, "Select (1)");
            if ui
                .add(select_btn.selected(active == EditorTool::Select))
                .clicked()
            {
                events.push(AppIntent::SetEditorToolRequested {
                    tool: EditorTool::Select,
                });
            }

            // ── Connect-Button mit SVG-Icon ──
            let connect_icon = svg_icon(
                egui::include_image!("../../assets/icon_connect.svg"),
                ICON_SIZE,
            );
            let connect_btn = egui::Button::image_and_text(connect_icon, "Connect (2)");
            if ui
                .add(connect_btn.selected(active == EditorTool::Connect))
                .clicked()
            {
                events.push(AppIntent::SetEditorToolRequested {
                    tool: EditorTool::Connect,
                });
            }

            // ── AddNode-Button mit SVG-Icon ──
            let add_icon = svg_icon(
                egui::include_image!("../../assets/icon_add_node.svg"),
                ICON_SIZE,
            );
            let add_btn = egui::Button::image_and_text(add_icon, "Add Node (3)");
            if ui
                .add(add_btn.selected(active == EditorTool::AddNode))
                .clicked()
            {
                events.push(AppIntent::SetEditorToolRequested {
                    tool: EditorTool::AddNode,
                });
            }

            ui.separator();

            // ── Linien-Tools als Dropdown-Menü mit SVG-Icons ──
            let active_route_index = if active == EditorTool::Route {
                state.editor.tool_manager.active_index()
            } else {
                None
            };
            let tool_entries = state.editor.tool_manager.tool_entries();
            let selected_label = active_route_index
                .and_then(|idx| tool_entries.iter().find(|(i, _, _)| *i == idx))
                .map(|(_, name, _icon)| name.to_string())
                .unwrap_or_else(|| "Linie wählen…".to_string());

            let popup_id = ui.make_persistent_id("line_tools_dropdown").with("popup");

            egui::ComboBox::from_id_salt("line_tools_dropdown")
                .selected_text(&selected_label)
                .width(185.0)
                .show_ui(ui, |ui| {
                    for &(idx, name, _icon) in &tool_entries {
                        render_line_tool_item(
                            ui,
                            idx,
                            name,
                            active_route_index,
                            popup_id,
                            &mut events,
                        );
                    }
                });

            ui.separator();

            // Delete-Button (nur wenn Selektion vorhanden)
            let has_selection = !state.selection.selected_node_ids.is_empty();
            if ui
                .add_enabled(has_selection, egui::Button::new("🗑 Delete (Del)"))
                .clicked()
            {
                events.push(AppIntent::DeleteSelectedRequested);
            }

            // Connect-Tool Status
            if active == EditorTool::Connect {
                ui.separator();
                if let Some(source_id) = state.editor.connect_source_node {
                    ui.label(format!("Startknoten: {} → Wähle Zielknoten", source_id));
                } else {
                    ui.label("Wähle Startknoten");
                }
            }

            // Route-Tool Status
            if active == EditorTool::Route {
                ui.separator();
                if let Some(tool) = state.editor.tool_manager.active_tool() {
                    ui.label(tool.status_text());
                }
            }

            // Background-Map-Controls (rechts ausgerichtet)
            if state.view.background_map.is_some() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut opacity = state.view.background_opacity;
                    ui.label("Hintergrund:");
                    if ui
                        .add(egui::Slider::new(&mut opacity, 0.0..=1.0).text("Opacity"))
                        .changed()
                    {
                        events.push(AppIntent::SetBackgroundOpacity { opacity });
                    }

                    let visible = state.view.background_visible;
                    let toggle_text = if visible {
                        "👁 Sichtbar"
                    } else {
                        "🚫 Ausgeblendet"
                    };
                    if ui.button(toggle_text).clicked() {
                        events.push(AppIntent::ToggleBackgroundVisibility);
                    }
                });
            }
        });
    });

    events
}

/// Gibt die `ImageSource` für das SVG-Icon eines Route-Tools anhand des Index zurück.
fn route_tool_icon(idx: usize) -> egui::ImageSource<'static> {
    match idx {
        0 => egui::include_image!("../../assets/icon_straight_road.svg"),
        1 => egui::include_image!("../../assets/icon_bezier_quadratic.svg"),
        2 => egui::include_image!("../../assets/icon_bezier_cubic.svg"),
        3 => egui::include_image!("../../assets/icon_spline.svg"),
        _ => egui::include_image!("../../assets/icon_straight_road.svg"),
    }
}

/// Rendert einen Dropdown-Eintrag für ein Linien-Tool mit SVG-Icon und Text.
fn render_line_tool_item(
    ui: &mut egui::Ui,
    idx: usize,
    name: &str,
    active_route_index: Option<usize>,
    popup_id: egui::Id,
    events: &mut Vec<AppIntent>,
) {
    let is_sel = active_route_index == Some(idx);
    let row_height = ui.spacing().interact_size.y.max(22.0);
    let row_size = egui::vec2(ui.available_width(), row_height);
    let (rect, resp) = ui.allocate_exact_size(row_size, egui::Sense::click());

    // Hintergrund
    let bg = if is_sel {
        Some(ui.visuals().selection.bg_fill)
    } else if resp.hovered() {
        Some(ui.visuals().widgets.hovered.bg_fill)
    } else {
        None
    };
    if let Some(bg) = bg {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(2), bg);
    }

    // SVG-Icon links
    let icon_rect = egui::Rect::from_min_size(
        rect.left_center() - egui::vec2(0.0, ICON_SIZE_DROPDOWN.y / 2.0) + egui::vec2(4.0, 0.0),
        ICON_SIZE_DROPDOWN,
    );
    let icon_image = svg_icon(route_tool_icon(idx), ICON_SIZE_DROPDOWN);
    icon_image.paint_at(ui, icon_rect);

    // Text rechts neben dem Icon
    let text_color = if is_sel {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().text_color()
    };
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    ui.painter().text(
        icon_rect.right_center() + egui::vec2(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        name,
        font_id,
        text_color,
    );

    if resp.clicked() {
        events.push(AppIntent::SelectRouteToolRequested { index: idx });
        egui::Popup::close_id(ui.ctx(), popup_id);
    }
}
