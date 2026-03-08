//! Toolbar für Editor-Werkzeugauswahl.

use crate::app::segment_registry::TOOL_INDEX_FIELD_BOUNDARY;
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};

// ── SVG-Icon-Konstanten (compile-time eingebettet) ──────────────

const ICON_SIZE: egui::Vec2 = egui::Vec2::new(20.0, 20.0);

/// Erstellt ein `egui::Image` aus einer `ImageSource` in der gewünschten Größe.
fn svg_icon(source: egui::ImageSource<'_>, size: egui::Vec2) -> egui::Image<'_> {
    egui::Image::new(source).fit_to_exact_size(size)
}

fn color32_from_rgba(color: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn function_icon_color(state: &AppState) -> egui::Color32 {
    match state.editor.default_priority {
        ConnectionPriority::Regular => color32_from_rgba(state.options.connection_color_regular),
        ConnectionPriority::SubPriority => color32_from_rgba(state.options.node_color_subprio),
    }
}

fn accent_icon_color(state: &AppState) -> egui::Color32 {
    match state.editor.default_direction {
        ConnectionDirection::Regular => color32_from_rgba(state.options.connection_color_regular),
        ConnectionDirection::Dual => color32_from_rgba(state.options.connection_color_dual),
        ConnectionDirection::Reverse => color32_from_rgba(state.options.connection_color_reverse),
    }
}

/// Rendert die schwebende Werkzeugleiste als `egui::Window` und gibt erzeugte Intents zurück.
///
/// Das Fenster ist zusammenklappbar und nicht skalierbar. Es enthält drei Gruppen,
/// die per `horizontal_wrapped` Layout und `ui.separator()` getrennt werden:
///
/// 1. **Haupt-Tools:** Select, Connect, AddNode — jeweils als Icon-Button
/// 2. **Route-Tools:** alle ToolManager-Einträge außer `FieldBoundaryTool`, als Icon-Buttons
///    (Icons via `route_tool_icon()`), inklusive Status-Anzeige im aktiven Zustand
/// 3. **Aktionen:** Delete-Button (aktiviert wenn Selektion vorhanden)
///
/// Wenn ein Hintergrund-Bild geladen ist, wird eine zusätzliche Hintergrund-Steuergruppe
/// (Sichtbarkeit, Skalierung, 1:1-Reset) in einem zweiten `horizontal_wrapped`-Block gerendert.
pub fn render_toolbar(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let active = state.editor.active_tool;
    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);

    egui::Window::new("🔧 Werkzeuge")
        .id(egui::Id::new("floating_toolbar"))
        .collapsible(true)
        .resizable(false)
        .default_pos(egui::pos2(300.0, 40.0))
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                // ── Haupt-Tools ──
                let select_icon = svg_icon(
                    egui::include_image!("../../assets/icon_select_node.svg"),
                    ICON_SIZE,
                )
                .tint(if active == EditorTool::Select {
                    active_icon_color
                } else {
                    icon_color
                });
                if ui
                    .add(egui::Button::image(select_icon).selected(active == EditorTool::Select))
                    .on_hover_text("Select (1)")
                    .clicked()
                {
                    events.push(AppIntent::SetEditorToolRequested {
                        tool: EditorTool::Select,
                    });
                }

                let connect_icon = svg_icon(
                    egui::include_image!("../../assets/icon_connect.svg"),
                    ICON_SIZE,
                )
                .tint(if active == EditorTool::Connect {
                    active_icon_color
                } else {
                    icon_color
                });
                if ui
                    .add(egui::Button::image(connect_icon).selected(active == EditorTool::Connect))
                    .on_hover_text("Connect (2)")
                    .clicked()
                {
                    events.push(AppIntent::SetEditorToolRequested {
                        tool: EditorTool::Connect,
                    });
                }

                let add_icon = svg_icon(
                    egui::include_image!("../../assets/icon_add_node.svg"),
                    ICON_SIZE,
                )
                .tint(if active == EditorTool::AddNode {
                    active_icon_color
                } else {
                    icon_color
                });
                if ui
                    .add(egui::Button::image(add_icon).selected(active == EditorTool::AddNode))
                    .on_hover_text("Add Node (3)")
                    .clicked()
                {
                    events.push(AppIntent::SetEditorToolRequested {
                        tool: EditorTool::AddNode,
                    });
                }

                ui.separator();

                // ── Route-Tools als Icon-Buttons ──
                let active_route_index = if active == EditorTool::Route {
                    state.editor.tool_manager.active_index()
                } else {
                    None
                };
                for &(idx, name, _icon_name) in &state.editor.tool_manager.tool_entries() {
                    if idx == TOOL_INDEX_FIELD_BOUNDARY {
                        continue;
                    }

                    let is_active = active_route_index == Some(idx);
                    let icon_img = egui::Image::new(route_tool_icon(idx))
                        .fit_to_exact_size(ICON_SIZE)
                        .tint(if is_active {
                            active_icon_color
                        } else {
                            icon_color
                        });

                    if ui
                        .add(egui::Button::image(icon_img).selected(is_active))
                        .on_hover_text(name)
                        .clicked()
                    {
                        events.push(AppIntent::SetEditorToolRequested {
                            tool: EditorTool::Route,
                        });
                        events.push(AppIntent::SelectRouteToolRequested { index: idx });
                    }
                }

                ui.separator();

                // ── Sonstige Aktionen ──
                let has_selection = !state.selection.selected_node_ids.is_empty();
                let delete_icon = svg_icon(
                    egui::include_image!("../../assets/icon_delete.svg"),
                    ICON_SIZE,
                )
                .tint(icon_color);
                if ui
                    .add_enabled(has_selection, egui::Button::image(delete_icon))
                    .on_hover_text("Delete (Del)")
                    .clicked()
                {
                    events.push(AppIntent::DeleteSelectedRequested);
                }

                // Status-Text bleibt in der Toolbar
                if active == EditorTool::Connect {
                    ui.separator();
                    if let Some(source_id) = state.editor.connect_source_node {
                        ui.label(format!("Startknoten: {} -> Waehle Zielknoten", source_id));
                    } else {
                        ui.label("Waehle Startknoten");
                    }
                }

                if active == EditorTool::Route {
                    ui.separator();
                    if let Some(tool) = state.editor.tool_manager.active_tool() {
                        ui.label(tool.status_text());
                    }
                }
            });

            // Background-Controls separat gruppieren
            if state.view.background_map.is_some() {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label("Hintergrund:");

                    let visible = state.view.background_visible;
                    let toggle_icon = if visible {
                        egui::include_image!("../../assets/icon_visible.svg")
                    } else {
                        egui::include_image!("../../assets/icon_hidden.svg")
                    };
                    let toggle_img = svg_icon(toggle_icon, ICON_SIZE).tint(if visible {
                        active_icon_color
                    } else {
                        icon_color
                    });
                    if ui
                        .add(egui::Button::image(toggle_img))
                        .on_hover_text(if visible {
                            "Hintergrund ausblenden"
                        } else {
                            "Hintergrund einblenden"
                        })
                        .clicked()
                    {
                        events.push(AppIntent::ToggleBackgroundVisibility);
                    }

                    let scale = state.view.background_scale;
                    if ui
                        .button("-")
                        .on_hover_text("Ausdehnung halbieren")
                        .clicked()
                    {
                        events.push(AppIntent::ScaleBackground { factor: 0.5 });
                    }
                    ui.label(format!("x{scale:.2}"));
                    if ui
                        .button("+")
                        .on_hover_text("Ausdehnung verdoppeln")
                        .clicked()
                    {
                        events.push(AppIntent::ScaleBackground { factor: 2.0 });
                    }
                    if (scale - 1.0).abs() > f32::EPSILON
                        && ui.button("1:1").on_hover_text("Originalgroesse").clicked()
                    {
                        // Setze Scale zurück auf 1.0 durch Faktor = 1/aktuell
                        events.push(AppIntent::ScaleBackground {
                            factor: 1.0 / scale,
                        });
                    }
                });
            }
        });

    events
}

/// Gibt die `ImageSource` für das SVG-Icon eines Route-Tools anhand des Index zurück.
fn route_tool_icon(idx: usize) -> egui::ImageSource<'static> {
    match idx {
        0 => egui::include_image!("../../assets/new/minus.svg"),
        1 => egui::include_image!("../../assets/icon_bezier_quadratic.svg"),
        2 => egui::include_image!("../../assets/icon_bezier_cubic.svg"),
        3 => egui::include_image!("../../assets/icon_spline.svg"),
        4 => egui::include_image!("../../assets/icon_bypass.svg"),
        5 => egui::include_image!("../../assets/icon_constraint_route.svg"),
        6 => egui::include_image!("../../assets/icon_parking.svg"),
        7 => egui::include_image!("../../assets/icon_field_boundary.svg"),
        8 => egui::include_image!("../../assets/icon_route_offset.svg"),
        _ => egui::include_image!("../../assets/new/minus.svg"),
    }
}
