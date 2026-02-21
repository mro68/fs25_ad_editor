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

            // Route-Tools (dynamisch aus ToolManager)
            let tool_names: Vec<(usize, String)> = state
                .editor
                .tool_manager
                .tool_names()
                .into_iter()
                .map(|(i, name)| (i, name.to_string()))
                .collect();
            let active_route_index = if active == EditorTool::Route {
                state.editor.tool_manager.active_index()
            } else {
                None
            };
            for (index, name) in &tool_names {
                let is_active = active_route_index == Some(*index);
                if ui.selectable_label(is_active, name.as_str()).clicked() {
                    events.push(AppIntent::SelectRouteToolRequested { index: *index });
                }
            }

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
