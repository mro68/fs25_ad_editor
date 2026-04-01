use crate::app::ui_contract::RouteToolPanelAdapter;
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority};
use crate::ui::properties::selectors::{
    render_direction_icon_selector, render_priority_icon_selector,
};

/// Route-Tool-Panel: Tool-Config + Ausfuehren/Abbrechen.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_route_tool_panel(
    ctx: &egui::Context,
    mut route_tool: RouteToolPanelAdapter<'_>,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    distance_wheel_step_m: f32,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
    let panel_data = route_tool.data();

    let mut window = egui::Window::new("📐 Route-Tool")
        .collapsible(false)
        .resizable(false)
        .default_width(360.0)
        .min_width(320.0)
        .max_width(420.0)
        .auto_sized();

    if let Some(pos) = panel_pos {
        window = window.default_pos(pos);
    }

    window.show(ctx, |ui| {
        ui.set_min_width(320.0);
        ui.set_max_width(420.0);

        if let Some(status_text) = panel_data.status_text.as_deref() {
            ui.label(status_text);
        }

        ui.add_space(6.0);
        let mut selected_dir = default_direction;
        render_direction_icon_selector(ui, &mut selected_dir, "route_tool_floating");
        if selected_dir != default_direction {
            events.push(AppIntent::SetDefaultDirectionRequested {
                direction: selected_dir,
            });
        }

        ui.add_space(4.0);
        let mut selected_prio = default_priority;
        render_priority_icon_selector(ui, &mut selected_prio, "route_tool_floating");
        if selected_prio != default_priority {
            events.push(AppIntent::SetDefaultPriorityRequested {
                priority: selected_prio,
            });
        }

        ui.add_space(6.0);

        let config_result = route_tool.render_config(ui, distance_wheel_step_m);
        if config_result.changed && config_result.needs_recreate {
            events.push(AppIntent::RouteToolConfigChanged);
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(panel_data.has_pending_input, egui::Button::new("✓ Ausfuehren"))
                .clicked()
            {
                events.push(AppIntent::RouteToolExecuteRequested);
            }
            if ui.button("✕ Abbrechen").clicked() {
                events.push(AppIntent::RouteToolCancelled);
            }
        });
    });
}
