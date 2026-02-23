//! Top-Menü (File, Edit, View, etc.).

use crate::app::{AppIntent, AppState, RenderQuality};

/// Rendert die Menü-Leiste
pub fn render_menu(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    events.push(AppIntent::OpenFileRequested);
                    ui.close();
                }

                ui.separator();

                let has_file = state.road_map.is_some();

                if ui
                    .add_enabled(has_file, egui::Button::new("Save"))
                    .clicked()
                {
                    events.push(AppIntent::SaveRequested);
                    ui.close();
                }

                if ui
                    .add_enabled(has_file, egui::Button::new("Save As..."))
                    .clicked()
                {
                    events.push(AppIntent::SaveAsRequested);
                    ui.close();
                }

                ui.separator();

                // Heightmap-Option
                let heightmap_label = if state.ui.heightmap_path.is_some() {
                    "Change Heightmap..."
                } else {
                    "Select Heightmap..."
                };

                if ui.button(heightmap_label).clicked() {
                    events.push(AppIntent::HeightmapSelectionRequested);
                    ui.close();
                }

                if state.ui.heightmap_path.is_some() && ui.button("Clear Heightmap").clicked() {
                    events.push(AppIntent::HeightmapCleared);
                    ui.close();
                }

                ui.separator();

                if ui.button("Übersichtskarte generieren...").clicked() {
                    events.push(AppIntent::GenerateOverviewRequested);
                    ui.close();
                }

                ui.separator();

                if ui.button("Exit").clicked() {
                    events.push(AppIntent::ExitRequested);
                    ui.close();
                }
            });

            // Edit menu: Undo / Redo / Optionen
            ui.menu_button("Edit", |ui| {
                let can_undo = state.can_undo();
                let can_redo = state.can_redo();

                if ui
                    .add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)"))
                    .clicked()
                {
                    events.push(AppIntent::UndoRequested);
                    ui.close();
                }

                if ui
                    .add_enabled(can_redo, egui::Button::new("Redo (Ctrl+Y / Shift+Cmd+Z)"))
                    .clicked()
                {
                    events.push(AppIntent::RedoRequested);
                    ui.close();
                }

                ui.separator();

                if ui.button("Optionen...").clicked() {
                    events.push(AppIntent::OpenOptionsDialogRequested);
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Reset Camera").clicked() {
                    events.push(AppIntent::ResetCameraRequested);
                    ui.close();
                }

                if ui.button("Zoom In").clicked() {
                    events.push(AppIntent::ZoomInRequested);
                    ui.close();
                }

                if ui.button("Zoom Out").clicked() {
                    events.push(AppIntent::ZoomOutRequested);
                    ui.close();
                }

                ui.separator();

                // Background-Map-Option
                let background_label = if state.view.background_map.is_some() {
                    "Hintergrund ändern..."
                } else {
                    "Hintergrund laden..."
                };

                if ui.button(background_label).clicked() {
                    events.push(AppIntent::BackgroundMapSelectionRequested);
                    ui.close();
                }

                ui.separator();

                ui.menu_button("Render Quality", |ui| {
                    let quality = state.view.render_quality;

                    if ui
                        .selectable_label(quality == RenderQuality::Low, "Low")
                        .clicked()
                    {
                        events.push(AppIntent::RenderQualityChanged {
                            quality: RenderQuality::Low,
                        });
                        ui.close();
                    }

                    if ui
                        .selectable_label(quality == RenderQuality::Medium, "Medium")
                        .clicked()
                    {
                        events.push(AppIntent::RenderQualityChanged {
                            quality: RenderQuality::Medium,
                        });
                        ui.close();
                    }

                    if ui
                        .selectable_label(quality == RenderQuality::High, "High")
                        .clicked()
                    {
                        events.push(AppIntent::RenderQualityChanged {
                            quality: RenderQuality::High,
                        });
                        ui.close();
                    }
                });
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    log::info!("FS25 AutoDrive Editor v{}", env!("CARGO_PKG_VERSION"));
                    ui.close();
                }
            });
        });
    });

    events
}
