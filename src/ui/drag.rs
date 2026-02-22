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

    let stroke = egui::Stroke::new(1.5, ui.visuals().selection.stroke.color);
    let fill = ui.visuals().selection.bg_fill.gamma_multiply(0.15);
    let painter = ui.painter();

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
