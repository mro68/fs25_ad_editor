//! Sidebar-Konfiguration fuer das ColorPathTool.

use super::state::{ColorPathPhase, ColorPathTool};

/// Rendert die ColorPathTool-Konfiguration im Properties-Panel.
///
/// Gibt `true` zurueck wenn sich Einstellungen geaendert haben (Neuzeichnung noetig).
pub(super) fn render_config_view(
    tool: &mut ColorPathTool,
    ui: &mut egui::Ui,
    _distance_wheel_step_m: f32,
) -> bool {
    let mut changed = false;

    ui.label("Farb-Pfad-Erkennung");
    ui.separator();

    // ── Status-Text ──────────────────────────────────────────────────────────
    let status = match tool.phase {
        ColorPathPhase::Idle => "Alt+Lasso fuer Farbsample",
        ColorPathPhase::Sampling => "Berechnen fuer Mittellinie",
        ColorPathPhase::Preview => "Enter zum Einfuegen, Reset zum Zuruecksetzen",
    };
    ui.colored_label(egui::Color32::LIGHT_BLUE, status);
    ui.separator();

    // ── Phase-Aktionen ────────────────────────────────────────────────────────
    match tool.phase {
        ColorPathPhase::Idle => {
            if ui.button("Starten \u{2192}").clicked() {
                tool.phase = ColorPathPhase::Sampling;
                changed = true;
            }
        }
        ColorPathPhase::Sampling => {
            // Farbinfos anzeigen
            let sample_count = tool.sampled_colors.len();
            if let Some(avg) = tool.avg_color {
                let color = egui::Color32::from_rgb(avg[0], avg[1], avg[2]);
                ui.horizontal(|ui| {
                    ui.label(format!("Samples: {sample_count}  Ø-Farbe:"));
                    // Farbvorschau-Quadrat (Mittelwert)
                    let (rect, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color);
                });
                let palette_size = tool.color_palette.len();
                ui.label(format!("Palette: {palette_size} Farben"));
                // Palette-Vorschau: kleine Quadrate fuer jeden Eintrag (max. 20)
                ui.horizontal_wrapped(|ui| {
                    for &c in tool.color_palette.iter().take(20) {
                        let pc = egui::Color32::from_rgb(c[0], c[1], c[2]);
                        let (rect, _) =
                            ui.allocate_exact_size(egui::Vec2::splat(10.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 1.0, pc);
                    }
                });
            } else {
                ui.label(format!("Samples: {sample_count}"));
                ui.colored_label(egui::Color32::GRAY, "Alt+Drag zum Sampeln von Farben");
            }

            ui.separator();

            if ui
                .add_enabled(
                    !tool.sampled_colors.is_empty(),
                    egui::Button::new("Berechnen \u{2192}"),
                )
                .on_disabled_hover_text("Zuerst Farben sampeln (Alt+Lasso)")
                .clicked()
            {
                tool.compute_pipeline();
                changed = true;
            }
        }
        ColorPathPhase::Preview => {
            let path_count = tool.skeleton_paths.len();
            let node_count = tool.resampled_nodes.len();
            ui.label(format!("Pfade: {path_count}  Nodes: {node_count}"));

            if path_count > 1 {
                ui.separator();
                ui.label("Pfad auswaehlen:");
                for i in 0..path_count {
                    let pt_count = tool.skeleton_paths[i].len();
                    let label = format!("Pfad {} ({} Punkte)", i + 1, pt_count);
                    let selected = tool.selected_path_index == Some(i);
                    if ui.selectable_label(selected, label).clicked() {
                        tool.select_path(i);
                        changed = true;
                    }
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !tool.resampled_nodes.is_empty(),
                        egui::Button::new("\u{2713} Uebernehmen"),
                    )
                    .on_disabled_hover_text("Keine Nodes zum Einfuegen")
                    .clicked()
                {
                    use crate::app::tools::RouteTool;
                    // ReadyToExecute wird ueber die normale Enter-Bestaetigung ausgeloest;
                    // hier nur visuelles Feedback — Execute laeuft ueber den Controller-Flow
                    let _ = tool.is_ready(); // Trigger fuer spaeteren Dispatch
                    changed = true;
                }

                if ui.button("\u{2190} Zurueck").clicked() {
                    tool.phase = ColorPathPhase::Sampling;
                    tool.skeleton_paths.clear();
                    tool.selected_path_index = None;
                    tool.centerline.clear();
                    tool.resampled_nodes.clear();
                    tool.mask.clear();
                    changed = true;
                }
            });
        }
    }

    ui.separator();

    // ── Reset-Button ─────────────────────────────────────────────────────────
    if ui.button("Reset").clicked() {
        use crate::app::tools::RouteTool;
        tool.reset();
        changed = true;
    }

    ui.separator();

    // ── Konfigurations-Slider ─────────────────────────────────────────────────
    ui.label("Einstellungen:");

    ui.horizontal(|ui| {
        ui.label("Farbtoleranz:");
        if ui
            .add(egui::Slider::new(&mut tool.config.color_tolerance, 5.0..=80.0).suffix(""))
            .changed()
        {
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Knotenabstand:");
        if ui
            .add(egui::Slider::new(&mut tool.config.node_spacing, 1.0..=50.0).suffix(" m"))
            .changed()
        {
            // Resampling in Preview-Phase sofort neu berechnen
            if tool.phase == ColorPathPhase::Preview {
                tool.apply_selected_path();
            }
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Vereinfachung:");
        if ui
            .add(egui::Slider::new(&mut tool.config.simplify_tolerance, 0.0..=20.0).suffix(" m"))
            .changed()
        {
            // Vereinfachung + Resampling in Preview-Phase sofort neu berechnen
            if tool.phase == ColorPathPhase::Preview {
                tool.apply_selected_path();
            }
            changed = true;
        }
    });

    ui.separator();

    if ui
        .checkbox(&mut tool.config.noise_filter, "Rauschfilter")
        .changed()
    {
        changed = true;
    }

    if ui
        .checkbox(
            &mut tool.config.connect_to_existing,
            "An Bestand anschliessen",
        )
        .changed()
    {
        changed = true;
    }

    changed
}
