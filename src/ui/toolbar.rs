//! Toolbar für Editor-Werkzeugauswahl.

use crate::app::{AppIntent, AppState, EditorTool};

/// Rendert die Toolbar und gibt erzeugte Events zurück.
pub fn render_toolbar(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let active = state.editor.active_tool;

    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Werkzeug:");
            ui.separator();

            if ui
                .selectable_label(active == EditorTool::Select, "⊹ Select (1)")
                .clicked()
            {
                events.push(AppIntent::SetEditorToolRequested {
                    tool: EditorTool::Select,
                });
            }

            if ui
                .selectable_label(active == EditorTool::Connect, "⟷ Connect (2)")
                .clicked()
            {
                events.push(AppIntent::SetEditorToolRequested {
                    tool: EditorTool::Connect,
                });
            }

            if ui
                .selectable_label(active == EditorTool::AddNode, "＋ Add Node (3)")
                .clicked()
            {
                events.push(AppIntent::SetEditorToolRequested {
                    tool: EditorTool::AddNode,
                });
            }

            ui.separator();

            // Linien-Tools als Dropdown-Menü
            let active_route_index = if active == EditorTool::Route {
                state.editor.tool_manager.active_index()
            } else {
                None
            };
            let tool_entries = state.editor.tool_manager.tool_entries();
            let selected_label = active_route_index
                .and_then(|idx| tool_entries.iter().find(|(i, _, _)| *i == idx))
                .map(|(_, name, icon)| format!("{name}  {icon}"))
                .unwrap_or_else(|| "Linie wählen…".to_string());

            // Popup-ID vorberechnen, damit der Eintrag das Dropdown schließen kann
            let popup_id = ui.make_persistent_id("line_tools_dropdown").with("popup");

            egui::ComboBox::from_id_salt("line_tools_dropdown")
                .selected_text(&selected_label)
                .width(185.0)
                .show_ui(ui, |ui| {
                    for &(idx, name, icon) in &tool_entries {
                        render_line_tool_item(
                            ui,
                            idx,
                            name,
                            icon,
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
                    // Opacity-Slider
                    let mut opacity = state.view.background_opacity;
                    ui.label("Hintergrund:");
                    if ui
                        .add(egui::Slider::new(&mut opacity, 0.0..=1.0).text("Opacity"))
                        .changed()
                    {
                        events.push(AppIntent::SetBackgroundOpacity { opacity });
                    }

                    // Toggle-Button
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

/// Rendert einen Dropdown-Eintrag für ein Linien-Tool mit Text links und Icon rechts.
fn render_line_tool_item(
    ui: &mut egui::Ui,
    idx: usize,
    name: &str,
    icon: &str,
    active_route_index: Option<usize>,
    popup_id: egui::Id,
    events: &mut Vec<AppIntent>,
) {
    let is_sel = active_route_index == Some(idx);
    let row_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
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

    let text_color = if is_sel {
        ui.visuals().strong_text_color()
    } else {
        ui.visuals().text_color()
    };
    let font_id = egui::TextStyle::Button.resolve(ui.style());

    // Text links
    ui.painter().text(
        rect.left_center() + egui::vec2(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        name,
        font_id.clone(),
        text_color,
    );
    // Icon rechts
    ui.painter().text(
        rect.right_center() - egui::vec2(6.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        icon,
        font_id,
        text_color,
    );

    if resp.clicked() {
        events.push(AppIntent::SelectRouteToolRequested { index: idx });
        egui::Popup::close_id(ui.ctx(), popup_id);
    }
}
