//! Optionen-Dialog für Farben, Größen und Breiten.

use crate::app::AppIntent;
use crate::shared::EditorOptions;

/// Zeigt den Options-Dialog und gibt erzeugte Events zurück.
pub fn show_options_dialog(
    ctx: &egui::Context,
    show: bool,
    options: &EditorOptions,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !show {
        return events;
    }

    // Arbeitskopie der Optionen für Live-Bearbeitung
    let mut opts = options.clone();
    let mut changed = false;

    egui::Window::new("Optionen")
        .collapsible(true)
        .resizable(true)
        .default_width(360.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(500.0)
                .show(ui, |ui| {
                    // ── Nodes ───────────────────────────────────────
                    ui.collapsing("Nodes", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Größe (Welt):");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.node_size_world)
                                        .range(0.1..=5.0)
                                        .speed(0.01),
                                )
                                .changed();
                        });
                        changed |= color_edit(ui, "Standardfarbe:", &mut opts.node_color_default);
                        changed |= color_edit(ui, "SubPrio-Farbe:", &mut opts.node_color_subprio);
                        changed |= color_edit(ui, "Selektiert:", &mut opts.node_color_selected);
                        changed |= color_edit(ui, "Warnung:", &mut opts.node_color_warning);
                        ui.horizontal(|ui| {
                            ui.label("Hitbox (% der Größe):");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.hitbox_scale_percent)
                                        .range(50.0..=500.0)
                                        .speed(5.0)
                                        .suffix(" %"),
                                )
                                .changed();
                        });
                    });

                    // ── Selektion ───────────────────────────────────
                    ui.collapsing("Selektion", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Größenfaktor:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.selection_size_factor)
                                        .range(1.0..=5.0)
                                        .speed(0.05),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Pick-Radius (px):");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.selection_pick_radius_px)
                                        .range(4.0..=50.0)
                                        .speed(0.5),
                                )
                                .changed();
                        });
                    });

                    // ── Connections ──────────────────────────────────
                    ui.collapsing("Verbindungen", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Breite Hauptstraße:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.connection_thickness_world)
                                        .range(0.01..=2.0)
                                        .speed(0.01),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Breite Nebenstraße:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(
                                        &mut opts.connection_thickness_subprio_world,
                                    )
                                    .range(0.01..=2.0)
                                    .speed(0.01),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Pfeillänge:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.arrow_length_world)
                                        .range(0.1..=5.0)
                                        .speed(0.05),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Pfeilbreite:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.arrow_width_world)
                                        .range(0.1..=5.0)
                                        .speed(0.05),
                                )
                                .changed();
                        });
                        changed |= color_edit(
                            ui,
                            "Regular (Einbahn):",
                            &mut opts.connection_color_regular,
                        );
                        changed |= color_edit(
                            ui,
                            "Dual (Bidirektional):",
                            &mut opts.connection_color_dual,
                        );
                        changed |= color_edit(
                            ui,
                            "Reverse (Rückwärts):",
                            &mut opts.connection_color_reverse,
                        );
                    });

                    // ── Marker ──────────────────────────────────────
                    ui.collapsing("Marker", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Pin-Größe:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.marker_size_world)
                                        .range(0.5..=10.0)
                                        .speed(0.1),
                                )
                                .changed();
                        });
                        changed |= color_edit(ui, "Pin-Farbe:", &mut opts.marker_color);
                        changed |= color_edit(ui, "Umriss-Farbe:", &mut opts.marker_outline_color);
                    });

                    // ── Kamera ──────────────────────────────────────
                    ui.collapsing("Kamera", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Min Zoom:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.camera_zoom_min)
                                        .range(0.01..=10.0)
                                        .speed(0.01),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Max Zoom:");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.camera_zoom_max)
                                        .range(1.0..=1000.0)
                                        .speed(1.0),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Zoom-Schritt (Menü):");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.camera_zoom_step)
                                        .range(1.01..=3.0)
                                        .speed(0.01),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Zoom-Schritt (Scroll):");
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut opts.camera_scroll_zoom_step)
                                        .range(1.01..=2.0)
                                        .speed(0.01),
                                )
                                .changed();
                        });
                    });

                    // ── Hintergrund ─────────────────────────────────
                    ui.collapsing("Hintergrund", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Sichtbarkeit normal:");
                            changed |= ui
                                .add(
                                    egui::Slider::new(
                                        &mut opts.background_opacity_normal,
                                        0.0..=1.0,
                                    )
                                    .step_by(0.05),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Sichtbarkeit bei Min-Zoom:");
                            changed |= ui
                                .add(
                                    egui::Slider::new(
                                        &mut opts.background_opacity_min_zoom,
                                        0.0..=1.0,
                                    )
                                    .step_by(0.05),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Fade-Out ab Zoom-Stufe:");
                            changed |= ui
                                .add(
                                    egui::Slider::new(
                                        &mut opts.background_fade_start_zoom,
                                        opts.camera_zoom_min..=opts.camera_zoom_max,
                                    )
                                    .step_by(0.5),
                                )
                                .changed();
                        });
                    });

                    // ── Übersichtskarte ─────────────────────────────
                    ui.collapsing("Übersichtskarte (Standard-Layer)", |ui| {
                        changed |= ui
                            .checkbox(&mut opts.overview_layers.hillshade, "Hillshade")
                            .changed();
                        changed |= ui
                            .checkbox(&mut opts.overview_layers.farmlands, "Farmland-Grenzen")
                            .changed();
                        changed |= ui
                            .checkbox(&mut opts.overview_layers.farmland_ids, "Farmland-IDs")
                            .changed();
                        changed |= ui
                            .checkbox(&mut opts.overview_layers.pois, "POI-Marker")
                            .changed();
                        changed |= ui
                            .checkbox(&mut opts.overview_layers.legend, "Legende")
                            .changed();
                    });
                });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Standardwerte").clicked() {
                    events.push(AppIntent::ResetOptionsRequested);
                }
                if ui.button("Schließen").clicked() {
                    events.push(AppIntent::CloseOptionsDialogRequested);
                }
            });
        });

    // Änderungen sofort anwenden (Live-Preview)
    if changed {
        events.push(AppIntent::OptionsChanged { options: opts });
    }

    events
}

/// Hilfsfunktion: Farb-Editor für [f32; 4] mit Alpha.
fn color_edit(ui: &mut egui::Ui, label: &str, color: &mut [f32; 4]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        let mut c = egui::Color32::from_rgba_unmultiplied(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        );
        if ui.color_edit_button_srgba(&mut c).changed() {
            color[0] = c.r() as f32 / 255.0;
            color[1] = c.g() as f32 / 255.0;
            color[2] = c.b() as f32 / 255.0;
            color[3] = c.a() as f32 / 255.0;
            changed = true;
        }
    });
    changed
}
