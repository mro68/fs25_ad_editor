//! Drag-Selektion (Rect/Lasso) und Overlay-Painting.

/// Modus der Drag-Selektion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragSelectionMode {
    /// Rechteck-Selektion
    Rect,
    /// Freihand-Lasso-Selektion
    Lasso,
}

/// Zustand einer aktiven Drag-Selektion
#[derive(Debug, Clone)]
pub(crate) struct DragSelection {
    /// Aktiver Selektions-Modus (Rect oder Lasso)
    pub mode: DragSelectionMode,
    /// Additive Selektion (Shift gedrückt) – erweitert statt zu ersetzen
    pub additive: bool,
    /// Startposition der Drag-Aktion in Screen-Koordinaten
    pub start_screen: egui::Pos2,
    /// Gesammelte Punkte der Drag-Aktion in Screen-Koordinaten
    pub points_screen: Vec<egui::Pos2>,
}

impl DragSelection {
    /// Fügt einen Lasso-Punkt hinzu, wenn der Mindestabstand erreicht ist.
    pub fn push_lasso_point(&mut self, pointer_pos: egui::Pos2) {
        let min_distance_sq = 3.0 * 3.0;
        let should_push = self
            .points_screen
            .last()
            .is_none_or(|last| last.distance_sq(pointer_pos) >= min_distance_sq);

        if should_push {
            self.points_screen.push(pointer_pos);
        }
    }
}

/// Zeichnet das Drag-Selektion-Overlay (Rect oder Lasso).
pub(super) fn draw_drag_selection_overlay(
    selection: Option<&DragSelection>,
    ui: &egui::Ui,
    response: &egui::Response,
) {
    let Some(selection) = selection else {
        return;
    };

    let mut stroke_color = ui.visuals().selection.stroke.color;
    if stroke_color.a() == 0 {
        // Fallback, falls Theme die Selection-Stroke transparent setzt.
        stroke_color = egui::Color32::from_rgb(80, 200, 255);
    }
    let stroke = egui::Stroke::new(1.5, stroke_color);

    let mut fill_color = ui.visuals().selection.bg_fill;
    if fill_color.a() == 0 {
        // Fallback, falls Theme die Selection-Fill transparent setzt.
        fill_color = egui::Color32::from_rgba_unmultiplied(80, 200, 255, 40);
    } else {
        fill_color = fill_color.gamma_multiply(0.15);
    }
    let fill = fill_color;
    let painter = ui
        .ctx()
        .layer_painter(egui::LayerId::new(egui::Order::Foreground, response.id))
        .with_clip_rect(response.rect);

    match selection.mode {
        DragSelectionMode::Rect => {
            let current = selection
                .points_screen
                .last()
                .copied()
                .unwrap_or(selection.start_screen);
            let rect =
                egui::Rect::from_two_pos(selection.start_screen, current).intersect(response.rect);
            painter.rect_filled(rect, 0.0, fill);
            painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
        }
        DragSelectionMode::Lasso => {
            if selection.points_screen.len() < 2 {
                return;
            }

            let mut polygon = selection.points_screen.clone();
            if polygon.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(polygon.clone(), fill, stroke));
                polygon.push(polygon[0]);
            }

            painter.add(egui::Shape::line(polygon, stroke));
        }
    }
}
