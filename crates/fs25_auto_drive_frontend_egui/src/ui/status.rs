//! Status-Bar am unteren Bildschirmrand.

use crate::app::EditorTool;
use crate::shared::{t, I18nKey};
use crate::ui::common::host_active_tool_to_editor;
use fs25_auto_drive_host_bridge::HostChromeSnapshot;

/// Rendert die Status-Bar
pub fn render_status_bar(
    ctx: &egui::Context,
    host_chrome_snapshot: &HostChromeSnapshot,
) {
    let mut top_ui = crate::ui::common::create_top_level_ui(ctx, "status_bar_top_level");
    render_status_bar_inside(&mut top_ui, host_chrome_snapshot);
}

/// Rendert die Status-Bar innerhalb eines bestehenden Top-Level-UIs.
pub(crate) fn render_status_bar_inside(
    ui_root: &mut egui::Ui,
    host_chrome_snapshot: &HostChromeSnapshot,
) {
    let lang = host_chrome_snapshot.options.language;
    let active_tool = host_active_tool_to_editor(host_chrome_snapshot.active_tool);

    egui::Panel::bottom("status_bar").show_inside(ui_root, |ui| {
        ui.horizontal(|ui| {
            if host_chrome_snapshot.has_map {
                ui.label(format!(
                    "{}: {} | {}: {} | {}: {}",
                    t(lang, I18nKey::StatusNodes),
                    host_chrome_snapshot.node_count,
                    t(lang, I18nKey::StatusConnections),
                    host_chrome_snapshot.connection_count,
                    t(lang, I18nKey::StatusMarkers),
                    host_chrome_snapshot.marker_count
                ));

                ui.separator();

                if let Some(ref map_name) = host_chrome_snapshot.map_name {
                    ui.label(format!("{}: {}", t(lang, I18nKey::StatusMap), map_name));
                    ui.separator();
                }
            } else {
                ui.label(t(lang, I18nKey::StatusNoFile));
            }

            ui.separator();

            ui.label(format!(
                "{}: {:.2}x | {}: ({:.1}, {:.1})",
                t(lang, I18nKey::StatusZoom),
                host_chrome_snapshot.camera_zoom,
                t(lang, I18nKey::StatusPosition),
                host_chrome_snapshot.camera_position[0],
                host_chrome_snapshot.camera_position[1]
            ));

            ui.separator();

            // Heightmap-Status
            if let Some(ref hm_path) = host_chrome_snapshot.heightmap_path {
                let filename = std::path::Path::new(hm_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                ui.label(format!(
                    "{}: {}",
                    t(lang, I18nKey::StatusHeightmap),
                    filename
                ));
            } else {
                ui.label(format!(
                    "{}: {}",
                    t(lang, I18nKey::StatusHeightmap),
                    t(lang, I18nKey::StatusHeightmapNone)
                ));
            }

            ui.separator();

            let selected_count = host_chrome_snapshot.selection_count;
            if selected_count > 0 {
                let example_id = host_chrome_snapshot
                    .selection_example_id
                    .unwrap_or_default();
                ui.label(format!(
                    "{}: {} ({} {})",
                    t(lang, I18nKey::StatusSelectedNodes),
                    selected_count,
                    t(lang, I18nKey::StatusExample),
                    example_id
                ));
            } else {
                ui.label(format!("{}: 0", t(lang, I18nKey::StatusSelectedNodes)));
            }

            ui.separator();

            // Aktives Werkzeug
            let tool_name = match active_tool {
                EditorTool::Select => t(lang, I18nKey::ToolNameSelect),
                EditorTool::Connect => t(lang, I18nKey::ToolNameConnect),
                EditorTool::AddNode => t(lang, I18nKey::ToolNameAddNode),
                EditorTool::Route => t(lang, I18nKey::ToolNameRoute),
            };
            ui.label(format!("{}: {}", t(lang, I18nKey::StatusTool), tool_name));

            // Statusnachricht (z.B. Duplikat-Bereinigung)
            if let Some(ref msg) = host_chrome_snapshot.status_message {
                ui.separator();
                ui.label(egui::RichText::new(format!("⚠ {}", msg)).color(egui::Color32::YELLOW));
            }

            // FPS-Anzeige (rechts)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "{}: {:.0}",
                    t(lang, I18nKey::StatusFps),
                    ui.ctx().input(|i| 1.0 / i.stable_dt)
                ));
            });
        });
    });
}

