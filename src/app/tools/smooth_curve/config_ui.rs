//! UI-Konfigurationspanel fuer das Geglättete-Kurve-Tool.
//!
//! Enthaelt:
//! - Max-Winkel-Slider (Solver-Parameter)
//! - Segment-Laenge (Max. Abstand zwischen Nodes)
//! - Minimaldistanz (filtert zu nahe beieinanderliegende Nodes)
//! - Steuerpunkte (automatisch berechnete Approach/Departure-Punkte)
//! - Kontrollpunkt-Liste mit Entfernen-Button
//! - Generierte Wegpunkt-Anzahl (Info)

use super::super::common::{wheel_dir, SegmentConfig};
use super::state::SmoothCurveTool;

impl SmoothCurveTool {
    /// Rendert nur den Max-Abstand-Slider (ohne Node-Anzahl).
    ///
    /// Fuer das Geglättete-Kurve-Tool ist die Node-Anzahl vom Solver bestimmt
    /// (kruemmungsadaptive Verteilung), daher wird nur der Abstand-Slider angezeigt.
    /// Gibt `(changed, recreate_needed)` zurueck.
    fn render_segment_distance_only(
        seg: &mut SegmentConfig,
        ui: &mut egui::Ui,
        adjusting: bool,
        ready: bool,
        length: f32,
        label: &str,
        distance_wheel_step_m: f32,
    ) -> (bool, bool) {
        let mut changed = false;
        let mut recreate = false;

        if adjusting || ready {
            ui.label(format!("{}: {:.1} m", label, length));
            ui.add_space(4.0);
        }

        ui.label("Max. Abstand:");
        let max_seg = if adjusting || ready {
            length.max(1.0)
        } else {
            20.0
        };
        let response =
            ui.add(egui::Slider::new(&mut seg.max_segment_length, 1.0..=max_seg).suffix(" m"));
        let mut distance_changed = response.changed();
        let wheel_dir = wheel_dir(ui, &response);
        if distance_wheel_step_m > 0.0 && wheel_dir != 0.0 {
            seg.max_segment_length =
                (seg.max_segment_length + wheel_dir * distance_wheel_step_m).clamp(1.0, max_seg);
            distance_changed = true;
        }
        if distance_changed {
            changed = true;
            if adjusting {
                recreate = true;
            }
        }

        (changed, recreate)
    }
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurueck wenn sich eine Einstellung geaendert hat.
    pub(super) fn render_config_view(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> bool {
        let mut changed = false;

        // Max-Winkel-Slider
        ui.label("Max. Richtungsaenderung:");
        let angle_response = ui.add(
            egui::Slider::new(&mut self.max_angle_deg, 5.0..=135.0)
                .suffix("°")
                .fixed_decimals(0),
        );
        let mut angle_changed = angle_response.changed();
        let angle_wheel_dir = wheel_dir(ui, &angle_response);
        if angle_wheel_dir != 0.0 {
            self.max_angle_deg = (self.max_angle_deg + angle_wheel_dir).clamp(5.0, 135.0);
            angle_changed = true;
        }
        if angle_changed {
            changed = true;
            self.update_preview();
            if !self.lifecycle.last_created_ids.is_empty() {
                self.lifecycle.recreate_needed = true;
            }
        }

        ui.add_space(6.0);

        // Segment-Konfiguration — nur Max-Abstand-Slider (keine Node-Anzahl,
        // da der Solver die Verteilung kruemmungsadaptiv bestimmt)
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();

        let length = if adjusting {
            let start = self.last_start_anchor.unwrap().position();
            let end = self.lifecycle.last_end_anchor.unwrap().position();
            start.distance(end)
        } else {
            self.total_distance()
        };

        let ready = self.start.is_some() && self.end.is_some();
        let (seg_changed, recreate) = Self::render_segment_distance_only(
            &mut self.seg,
            ui,
            adjusting,
            ready,
            length,
            "Routenlaenge",
            distance_wheel_step_m,
        );
        if seg_changed {
            changed = true;
            self.update_preview();
        }
        if recreate {
            self.lifecycle.recreate_needed = true;
        }

        // Minimaldistanz-Slider
        ui.add_space(4.0);
        ui.label("Minimaldistanz:");
        let min_dist_response = ui.add(
            egui::Slider::new(&mut self.min_distance, 0.5..=20.0)
                .suffix(" m")
                .fixed_decimals(1),
        );
        let mut min_dist_changed = min_dist_response.changed();
        let min_dist_wheel_dir = wheel_dir(ui, &min_dist_response);
        if distance_wheel_step_m > 0.0 && min_dist_wheel_dir != 0.0 {
            self.min_distance =
                (self.min_distance + min_dist_wheel_dir * distance_wheel_step_m).clamp(0.5, 20.0);
            min_dist_changed = true;
        }
        if min_dist_changed {
            changed = true;
            self.update_preview();
            if !self.lifecycle.last_created_ids.is_empty() {
                self.lifecycle.recreate_needed = true;
            }
        }

        // ── Steuerpunkte (Auto-Approach/Departure) ────────────────────
        let has_steerers = self.approach_steerer.is_some() || self.departure_steerer.is_some();
        if has_steerers {
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Steuerpunkte").strong());
            ui.label(
                egui::RichText::new("Verschiebbar per Drag im Viewport")
                    .weak()
                    .small(),
            );

            if let Some(ap) = self.approach_steerer {
                ui.horizontal(|ui| {
                    let label = if self.approach_manual {
                        format!("  ⊳ Approach: ({:.1}, {:.1}) ✎", ap.x, ap.y)
                    } else {
                        format!("  ⊳ Approach: ({:.1}, {:.1})", ap.x, ap.y)
                    };
                    ui.label(label);
                    if self.approach_manual
                        && ui
                            .small_button("↺")
                            .on_hover_text("Zuruecksetzen auf Auto")
                            .clicked()
                    {
                        self.approach_manual = false;
                        self.update_preview();
                        changed = true;
                        if !self.lifecycle.last_created_ids.is_empty() {
                            self.lifecycle.recreate_needed = true;
                        }
                    }
                });
            }

            if let Some(dp) = self.departure_steerer {
                ui.horizontal(|ui| {
                    let label = if self.departure_manual {
                        format!("  ⊲ Departure: ({:.1}, {:.1}) ✎", dp.x, dp.y)
                    } else {
                        format!("  ⊲ Departure: ({:.1}, {:.1})", dp.x, dp.y)
                    };
                    ui.label(label);
                    if self.departure_manual
                        && ui
                            .small_button("↺")
                            .on_hover_text("Zuruecksetzen auf Auto")
                            .clicked()
                    {
                        self.departure_manual = false;
                        self.update_preview();
                        changed = true;
                        if !self.lifecycle.last_created_ids.is_empty() {
                            self.lifecycle.recreate_needed = true;
                        }
                    }
                });
            }
        }

        // ── Kontrollpunkt-Liste ───────────────────────────────────────
        if !self.control_nodes.is_empty() {
            ui.add_space(6.0);
            ui.label(format!("Kontrollpunkte ({})", self.control_nodes.len()));

            let mut remove_idx = None;
            for (i, cp) in self.control_nodes.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("  #{}: ({:.1}, {:.1})", i + 1, cp.x, cp.y));
                    if ui.small_button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }
            if let Some(idx) = remove_idx {
                self.control_nodes.remove(idx);
                self.sync_derived();
                self.update_preview();
                changed = true;
                if !self.lifecycle.last_created_ids.is_empty() {
                    self.lifecycle.recreate_needed = true;
                }
            }
        }

        // ── Vorschau-Statistik ────────────────────────────────────────
        if !self.preview_positions.is_empty() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!(
                    "Vorschau: {} Wegpunkte",
                    self.preview_positions.len()
                ))
                .weak()
                .small(),
            );
        }

        changed
    }
}
