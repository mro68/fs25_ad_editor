//! Top-Menue (File, Edit, View, etc.).

use crate::app::{AppIntent, AppState, RenderQuality};
use crate::shared::{t, I18nKey};

/// Rendert die Menue-Leiste
pub fn render_menu(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let lang = state.options.language;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(t(lang, I18nKey::MenuFile), |ui| {
                if ui.button(t(lang, I18nKey::MenuOpen)).clicked() {
                    events.push(AppIntent::OpenFileRequested);
                    ui.close();
                }

                ui.separator();

                let has_file = state.road_map.is_some();

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
                let can_undo = state.can_undo();
                let can_redo = state.can_redo();

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
                let has_selection = !state.selection.selected_node_ids.is_empty();
                let has_clipboard = !state.clipboard.nodes.is_empty();

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
                let has_farmland = state
                    .farmland_polygons
                    .as_ref()
                    .is_some_and(|p| !p.is_empty());
                if ui
                    .add_enabled(
                        has_farmland,
                        egui::Button::new(t(lang, I18nKey::MenuDetectField)),
                    )
                    .on_disabled_hover_text(t(lang, I18nKey::MenuExtrasNeedBackground))
                    .clicked()
                {
                    events.push(AppIntent::SelectRouteToolRequested {
                        index: crate::app::group_registry::TOOL_INDEX_FIELD_BOUNDARY,
                    });
                    ui.close();
                }
                if ui
                    .add_enabled(
                        has_farmland,
                        egui::Button::new(t(lang, I18nKey::MenuTraceAllFields)),
                    )
                    .on_disabled_hover_text(t(lang, I18nKey::MenuExtrasNeedBackground))
                    .on_hover_text(t(lang, I18nKey::MenuTraceAllFieldsHelp))
                    .clicked()
                {
                    events.push(AppIntent::OpenTraceAllFieldsDialogRequested);
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
