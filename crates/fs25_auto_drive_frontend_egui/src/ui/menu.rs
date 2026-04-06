//! Top-Menue (File, Edit, View, etc.).

use crate::app::tool_contract::RouteToolId;
use crate::app::tools::{route_tool_group_label_key, route_tool_label_key, RouteToolGroup};
use crate::app::{AppIntent, AppState, RenderQuality};
use crate::shared::{t, I18nKey};
use crate::ui::common::{
    host_route_tool_disabled_reason_key, host_route_tool_entries_for, host_route_tool_to_engine,
};
use fs25_auto_drive_host_bridge::{HostChromeSnapshot, HostRouteToolGroup, HostRouteToolSurface};

fn push_route_tool_selection(events: &mut Vec<AppIntent>, tool_id: RouteToolId) {
    events.push(AppIntent::SelectRouteToolRequested { tool_id });
}

fn render_route_tool_group_menu(
    ui: &mut egui::Ui,
    state: &AppState,
    host_chrome_snapshot: &HostChromeSnapshot,
    events: &mut Vec<AppIntent>,
    group: HostRouteToolGroup,
) {
    let lang = state.options.language;
    let active_route_id = state.active_route_tool_id();

    let group_label_key = match group {
        HostRouteToolGroup::Basics => route_tool_group_label_key(RouteToolGroup::Basics),
        HostRouteToolGroup::Section => route_tool_group_label_key(RouteToolGroup::Section),
        HostRouteToolGroup::Analysis => route_tool_group_label_key(RouteToolGroup::Analysis),
    };

    ui.menu_button(t(lang, group_label_key), |ui| {
        for entry in
            host_route_tool_entries_for(host_chrome_snapshot, HostRouteToolSurface::MainMenu, group)
        {
            let engine_tool_id = host_route_tool_to_engine(entry.tool);
            let response = ui.add_enabled(
                entry.enabled,
                egui::Button::new(t(lang, route_tool_label_key(engine_tool_id)))
                    .selected(active_route_id == Some(engine_tool_id)),
            );

            let response = if entry.enabled {
                response.on_hover_text(t(lang, route_tool_label_key(engine_tool_id)))
            } else {
                response.on_disabled_hover_text(t(
                    lang,
                    host_route_tool_disabled_reason_key(
                        entry
                            .disabled_reason
                            .expect("disabled route tool menu entry requires reason"),
                    ),
                ))
            };

            if response.clicked() {
                push_route_tool_selection(events, engine_tool_id);
                ui.close();
            }
        }
    });
}

/// Rendert die Menue-Leiste
pub fn render_menu(
    ctx: &egui::Context,
    state: &AppState,
    host_chrome_snapshot: &HostChromeSnapshot,
) -> Vec<AppIntent> {
    let mut top_ui = crate::ui::common::create_top_level_ui(ctx, "menu_bar_top_level");
    render_menu_inside(&mut top_ui, state, host_chrome_snapshot)
}

/// Rendert die Menue-Leiste innerhalb eines bestehenden Top-Level-UIs.
pub(crate) fn render_menu_inside(
    ui_root: &mut egui::Ui,
    state: &AppState,
    host_chrome_snapshot: &HostChromeSnapshot,
) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let lang = state.options.language;

    egui::Panel::top("menu_bar").show_inside(ui_root, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(t(lang, I18nKey::MenuFile), |ui| {
                if ui.button(t(lang, I18nKey::MenuOpen)).clicked() {
                    events.push(AppIntent::OpenFileRequested);
                    ui.close();
                }

                ui.separator();

                let has_file = host_chrome_snapshot.has_map;

                if ui
                    .add_enabled(has_file, egui::Button::new(t(lang, I18nKey::MenuSave)))
                    .clicked()
                {
                    events.push(AppIntent::SaveRequested);
                    ui.close();
                }

                if ui
                    .add_enabled(has_file, egui::Button::new(t(lang, I18nKey::MenuSaveAs)))
                    .clicked()
                {
                    events.push(AppIntent::SaveAsRequested);
                    ui.close();
                }

                ui.separator();

                // Heightmap-Option
                let heightmap_label = if state.ui.heightmap_path.is_some() {
                    t(lang, I18nKey::MenuChangeHeightmap)
                } else {
                    t(lang, I18nKey::MenuSelectHeightmap)
                };

                if ui.button(heightmap_label).clicked() {
                    events.push(AppIntent::HeightmapSelectionRequested);
                    ui.close();
                }

                if state.ui.heightmap_path.is_some()
                    && ui.button(t(lang, I18nKey::MenuClearHeightmap)).clicked()
                {
                    events.push(AppIntent::HeightmapCleared);
                    ui.close();
                }

                ui.separator();

                if ui.button(t(lang, I18nKey::MenuGenerateOverview)).clicked() {
                    events.push(AppIntent::GenerateOverviewRequested);
                    ui.close();
                }

                ui.separator();

                if ui.button(t(lang, I18nKey::MenuExit)).clicked() {
                    events.push(AppIntent::ExitRequested);
                    ui.close();
                }
            });

            // Edit menu: Undo / Redo / Optionen
            ui.menu_button(t(lang, I18nKey::MenuEdit), |ui| {
                let can_undo = host_chrome_snapshot.can_undo;
                let can_redo = host_chrome_snapshot.can_redo;

                if ui
                    .add_enabled(can_undo, egui::Button::new(t(lang, I18nKey::MenuUndo)))
                    .clicked()
                {
                    events.push(AppIntent::UndoRequested);
                    ui.close();
                }

                if ui
                    .add_enabled(can_redo, egui::Button::new(t(lang, I18nKey::MenuRedo)))
                    .clicked()
                {
                    events.push(AppIntent::RedoRequested);
                    ui.close();
                }

                ui.separator();

                // Copy / Paste
                let has_selection = host_chrome_snapshot.has_selection;
                let has_clipboard = host_chrome_snapshot.has_clipboard;

                if ui
                    .add_enabled(has_selection, egui::Button::new(t(lang, I18nKey::MenuCopy)))
                    .clicked()
                {
                    events.push(AppIntent::CopySelectionRequested);
                    ui.close();
                }

                if ui
                    .add_enabled(
                        has_clipboard,
                        egui::Button::new(t(lang, I18nKey::MenuPaste)),
                    )
                    .clicked()
                {
                    events.push(AppIntent::PasteStartRequested);
                    ui.close();
                }

                ui.separator();

                if ui.button(t(lang, I18nKey::MenuOptions)).clicked() {
                    events.push(AppIntent::OpenOptionsDialogRequested);
                    ui.close();
                }
            });

            ui.menu_button(t(lang, I18nKey::MenuRouteTools), |ui| {
                render_route_tool_group_menu(
                    ui,
                    state,
                    host_chrome_snapshot,
                    &mut events,
                    HostRouteToolGroup::Basics,
                );
                render_route_tool_group_menu(
                    ui,
                    state,
                    host_chrome_snapshot,
                    &mut events,
                    HostRouteToolGroup::Section,
                );
                render_route_tool_group_menu(
                    ui,
                    state,
                    host_chrome_snapshot,
                    &mut events,
                    HostRouteToolGroup::Analysis,
                );
            });

            ui.menu_button(t(lang, I18nKey::MenuView), |ui| {
                if ui.button(t(lang, I18nKey::MenuResetCamera)).clicked() {
                    events.push(AppIntent::ResetCameraRequested);
                    ui.close();
                }

                if ui.button(t(lang, I18nKey::MenuZoomIn)).clicked() {
                    events.push(AppIntent::ZoomInRequested);
                    ui.close();
                }

                if ui.button(t(lang, I18nKey::MenuZoomOut)).clicked() {
                    events.push(AppIntent::ZoomOutRequested);
                    ui.close();
                }

                ui.separator();

                // Background-Map-Option
                let background_label = if state.view.background_map.is_some() {
                    t(lang, I18nKey::MenuChangeBackground)
                } else {
                    t(lang, I18nKey::MenuLoadBackground)
                };

                if ui.button(background_label).clicked() {
                    events.push(AppIntent::BackgroundMapSelectionRequested);
                    ui.close();
                }

                ui.separator();

                ui.menu_button(t(lang, I18nKey::MenuRenderQuality), |ui| {
                    let quality = state.view.render_quality;

                    if ui
                        .selectable_label(
                            quality == RenderQuality::Low,
                            t(lang, I18nKey::MenuQualityLow),
                        )
                        .clicked()
                    {
                        events.push(AppIntent::RenderQualityChanged {
                            quality: RenderQuality::Low,
                        });
                        ui.close();
                    }

                    if ui
                        .selectable_label(
                            quality == RenderQuality::Medium,
                            t(lang, I18nKey::MenuQualityMedium),
                        )
                        .clicked()
                    {
                        events.push(AppIntent::RenderQualityChanged {
                            quality: RenderQuality::Medium,
                        });
                        ui.close();
                    }

                    if ui
                        .selectable_label(
                            quality == RenderQuality::High,
                            t(lang, I18nKey::MenuQualityHigh),
                        )
                        .clicked()
                    {
                        events.push(AppIntent::RenderQualityChanged {
                            quality: RenderQuality::High,
                        });
                        ui.close();
                    }
                });
            });

            ui.menu_button(t(lang, I18nKey::MenuExtras), |ui| {
                let has_farmland = state.farmland_polygons_arc().is_some_and(|p| !p.is_empty());
                if ui
                    .add_enabled(
                        has_farmland,
                        egui::Button::new(t(lang, I18nKey::MenuTraceAllFields)),
                    )
                    .on_disabled_hover_text(t(lang, I18nKey::RouteToolNeedFarmland))
                    .on_hover_text(t(lang, I18nKey::MenuTraceAllFieldsHelp))
                    .clicked()
                {
                    events.push(AppIntent::OpenTraceAllFieldsDialogRequested);
                    ui.close();
                }

                ui.separator();

                let has_file = host_chrome_snapshot.has_map;
                if ui
                    .add_enabled(
                        has_file,
                        egui::Button::new(t(lang, I18nKey::MenuCurseplayImport)),
                    )
                    .clicked()
                {
                    events.push(AppIntent::CurseplayImportRequested);
                    ui.close();
                }

                let has_selection = host_chrome_snapshot.has_selection;
                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(t(lang, I18nKey::MenuCurseplayExport)),
                    )
                    .clicked()
                {
                    events.push(AppIntent::CurseplayExportRequested);
                    ui.close();
                }
            });

            ui.menu_button(t(lang, I18nKey::MenuHelp), |ui| {
                if ui.button(t(lang, I18nKey::MenuAbout)).clicked() {
                    log::info!("FS25 AutoDrive Editor v{}", env!("CARGO_PKG_VERSION"));
                    ui.close();
                }
            });
        });
    });

    events
}
