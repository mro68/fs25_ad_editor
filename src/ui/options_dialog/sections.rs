//! Einzelne Einstellungs-Abschnitte fuer den Options-Dialog.
//!
//! Jede `render_*`-Funktion rendert einen thematischen Block und gibt `true`
//! zurueck wenn sich ein Wert geaendert hat.

use crate::shared::{EditorOptions, SelectionStyle, ValueAdjustInputMode};
use crate::ui::common::apply_wheel_step;

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

/// Rendert die Node-Darstellungseinstellungen (Groesse, Farben, Hitbox).
pub(super) fn render_nodes(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Groesse (Welt):");
        let r = ui.add(
            egui::DragValue::new(&mut opts.node_size_world)
                .range(0.1..=5.0)
                .speed(0.01),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.node_size_world, 0.1, 0.1..=5.0);
    });
    changed |= color_edit(ui, "Standardfarbe:", &mut opts.node_color_default);
    changed |= color_edit(ui, "SubPrio-Farbe:", &mut opts.node_color_subprio);
    changed |= color_edit(ui, "Selektiert:", &mut opts.node_color_selected);
    changed |= color_edit(ui, "Warnung:", &mut opts.node_color_warning);
    ui.horizontal(|ui| {
        ui.label("Hitbox (% der Groesse):");
        let r = ui.add(
            egui::DragValue::new(&mut opts.hitbox_scale_percent)
                .range(50.0..=500.0)
                .speed(5.0)
                .suffix(" %"),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.hitbox_scale_percent, 10.0, 50.0..=500.0);
    });
    changed
}

/// Rendert die Werkzeug-Einstellungen (Eingabemodus, Snap-Radius, Mausrad-Schritt).
pub(super) fn render_tools(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Wertaenderung:");
        let current_label = match opts.value_adjust_input_mode {
            ValueAdjustInputMode::DragHorizontal => "LMT li/re",
            ValueAdjustInputMode::MouseWheel => "Mausrad hoch/runter",
        };
        egui::ComboBox::from_id_salt("value_adjust_input_mode")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        &mut opts.value_adjust_input_mode,
                        ValueAdjustInputMode::DragHorizontal,
                        "LMT li/re",
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .selectable_value(
                        &mut opts.value_adjust_input_mode,
                        ValueAdjustInputMode::MouseWheel,
                        "Mausrad hoch/runter",
                    )
                    .changed()
                {
                    changed = true;
                }
            });
    });
    ui.horizontal(|ui| {
        ui.label("Snap-Radius:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.snap_scale_percent)
                .range(50.0..=2000.0)
                .speed(10.0)
                .suffix(" %"),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.snap_scale_percent, 10.0, 50.0..=2000.0);
    });
    ui.horizontal(|ui| {
        ui.label("Mausrad-Schritt Distanz:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.mouse_wheel_distance_step_m)
                .range(0.01..=5.0)
                .speed(0.01)
                .suffix(" m"),
        );
        changed |= r.changed()
            | apply_wheel_step(
                ui,
                &r,
                &mut opts.mouse_wheel_distance_step_m,
                0.1,
                0.01..=5.0,
            );
    });
    changed
}

/// Rendert die Selektions-Einstellungen (Groessenfaktor, Markierungsstil).
pub(super) fn render_selection(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Groessenfaktor (%):");
        let r = ui.add(
            egui::DragValue::new(&mut opts.selection_size_factor)
                .range(100.0..=200.0)
                .speed(1.0),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.selection_size_factor, 5.0, 100.0..=200.0);
    });
    ui.horizontal(|ui| {
        ui.label("Markierungsstil:");
        let current_label = match opts.selection_style {
            SelectionStyle::Ring => "Ring",
            SelectionStyle::Gradient => "Farbverlauf",
        };
        egui::ComboBox::from_id_salt("selection_style")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for (style, label) in [
                    (SelectionStyle::Ring, "Ring"),
                    (SelectionStyle::Gradient, "Farbverlauf"),
                ] {
                    if ui
                        .selectable_value(&mut opts.selection_style, style, label)
                        .changed()
                    {
                        changed = true;
                    }
                }
            });
    });
    ui.separator();
    ui.label("Doppelklick-Segment:");
    ui.horizontal(|ui| {
        changed |= ui
            .checkbox(&mut opts.segment_stop_at_junction, "Bei Kreuzung stoppen")
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label("Max. Winkel (°):");
        let r = ui.add(
            egui::DragValue::new(&mut opts.segment_max_angle_deg)
                .range(0.0..=180.0)
                .speed(1.0),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.segment_max_angle_deg, 5.0, 0.0..=180.0);
        if opts.segment_max_angle_deg == 0.0 {
            ui.weak("(deaktiviert)");
        }
    });
    changed
}

/// Rendert die Verbindungs-Darstellungseinstellungen (Breite, Pfeilgroessen, Farben).
pub(super) fn render_connections(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Breite Hauptstrasse:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.connection_thickness_world)
                .range(0.01..=2.0)
                .speed(0.01),
        );
        changed |= r.changed()
            | apply_wheel_step(
                ui,
                &r,
                &mut opts.connection_thickness_world,
                0.1,
                0.01..=2.0,
            );
    });
    ui.horizontal(|ui| {
        ui.label("Breite Nebenstrasse:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.connection_thickness_subprio_world)
                .range(0.01..=2.0)
                .speed(0.01),
        );
        changed |= r.changed()
            | apply_wheel_step(
                ui,
                &r,
                &mut opts.connection_thickness_subprio_world,
                0.1,
                0.01..=2.0,
            );
    });
    ui.horizontal(|ui| {
        ui.label("Pfeillaenge:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.arrow_length_world)
                .range(0.1..=5.0)
                .speed(0.05),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.arrow_length_world, 0.5, 0.1..=5.0);
    });
    ui.horizontal(|ui| {
        ui.label("Pfeilbreite:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.arrow_width_world)
                .range(0.1..=5.0)
                .speed(0.05),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.arrow_width_world, 0.5, 0.1..=5.0);
    });
    changed |= color_edit(ui, "Einbahn vorwaerts:", &mut opts.connection_color_regular);
    changed |= color_edit(ui, "Zweirichtungsverkehr:", &mut opts.connection_color_dual);
    changed |= color_edit(
        ui,
        "Einbahn rueckwaerts:",
        &mut opts.connection_color_reverse,
    );
    changed
}

/// Rendert die Marker-Darstellungseinstellungen (Pin-Groesse, Farben, Umrissstaerke).
pub(super) fn render_markers(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(egui::include_image!("../../../assets/icons/map-pin.svg"))
                .fit_to_exact_size(egui::Vec2::new(14.0, 14.0)),
        );
        ui.label("Pin-Groesse:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.marker_size_world)
                .range(0.5..=10.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.marker_size_world, 1.0, 0.5..=10.0);
    });
    changed |= color_edit(ui, "Pin-Farbe:", &mut opts.marker_color);
    ui.horizontal(|ui| {
        ui.label("Umrissstaerke:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.marker_outline_width)
                .range(0.01..=0.3)
                .speed(0.005)
                .fixed_decimals(3),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.marker_outline_width, 0.01, 0.01..=0.3);
    });
    changed
}

/// Rendert die Kamera-Einstellungen (Zoom-Grenzen, Scroll-Schritt, Kompensation).
pub(super) fn render_camera(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Min Zoom:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_min)
                .range(0.01..=10.0)
                .speed(0.01),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.camera_zoom_min, 0.1, 0.01..=10.0);
    });
    ui.horizontal(|ui| {
        ui.label("Max Zoom:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_max)
                .range(1.0..=1000.0)
                .speed(1.0),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.camera_zoom_max, 5.0, 1.0..=1000.0);
    });
    ui.horizontal(|ui| {
        ui.label("Zoom-Schritt (Menue):");
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_step)
                .range(1.01..=3.0)
                .speed(0.01),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.camera_zoom_step, 0.05, 1.01..=3.0);
    });
    ui.horizontal(|ui| {
        ui.label("Zoom-Schritt (Scroll):");
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_scroll_zoom_step)
                .range(1.01..=2.0)
                .speed(0.01),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.camera_scroll_zoom_step, 0.05, 1.01..=2.0);
    });
    ui.horizontal(|ui| {
        ui.label("Zoom-Kompensation Max:");
        let r = ui
            .add(
                egui::Slider::new(&mut opts.zoom_compensation_max, 1.0..=8.0)
                    .step_by(0.1)
                    .fixed_decimals(1),
            )
            .on_hover_text(
                "Wie stark Nodes und Verbindungen beim Herauszoomen vergroessert werden (1.0 = deaktiviert, 4.0 = Standard)"
            );
        changed |= r.changed() | apply_wheel_step(ui, &r, &mut opts.zoom_compensation_max, 0.1, 1.0..=8.0);
    });
    changed
}

/// Rendert die LOD/Mindestgroessen-Einstellungen (Pixel-Untergrenzen + Node-Decimation).
pub(super) fn render_lod(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.label("Mindestgroessen (Pixel, 0 = deaktiviert):");
    ui.horizontal(|ui| {
        ui.label("Nodes:");
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_node_size_px, 0.0..=20.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(
                "Mindestgroesse fuer Nodes in Pixeln beim Herauszoomen (0 = deaktiviert)",
            );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_node_size_px, 1.0, 0.0..=20.0);
    });
    ui.horizontal(|ui| {
        ui.label("Verbindungen:");
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_connection_width_px, 0.0..=10.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(
                "Mindestbreite fuer Verbindungslinien in Pixeln beim Herauszoomen (0 = deaktiviert)",
            );
        changed |= r.changed() | apply_wheel_step(ui, &r, &mut opts.min_connection_width_px, 0.5, 0.0..=10.0);
    });
    ui.horizontal(|ui| {
        ui.label("Pfeile:");
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_arrow_size_px, 0.0..=20.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(
                "Mindestgroesse fuer Richtungspfeile in Pixeln beim Herauszoomen (0 = deaktiviert)",
            );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_arrow_size_px, 1.0, 0.0..=20.0);
    });
    ui.horizontal(|ui| {
        ui.label("Marker:");
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_marker_size_px, 0.0..=30.0)
                    .step_by(1.0)
                    .fixed_decimals(0),
            )
            .on_hover_text(
                "Mindestgroesse fuer Marker-Pins in Pixeln beim Herauszoomen (0 = deaktiviert)",
            );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_marker_size_px, 1.0, 0.0..=30.0);
    });
    ui.separator();
    ui.label("Node-Ausdünnung:");
    ui.horizontal(|ui| {
        ui.label("Mindestabstand (px):");
        let r = ui
            .add(
                egui::Slider::new(&mut opts.node_decimation_spacing_px, 0.0..=50.0)
                    .step_by(1.0)
                    .fixed_decimals(0),
            )
            .on_hover_text(
                "Mindestabstand zwischen Nodes in Pixeln beim Herauszoomen. 0 = alle Nodes zeigen.",
            );
        changed |= r.changed()
            | apply_wheel_step(
                ui,
                &r,
                &mut opts.node_decimation_spacing_px,
                1.0,
                0.0..=50.0,
            );
    });
    changed
}

/// Rendert die Hintergrundkarten-Einstellungen (Deckung, Fade-out).
pub(super) fn render_background(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Standard-Deckung:");
        let r = ui.add(
            egui::Slider::new(&mut opts.bg_opacity, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |= r.changed() | apply_wheel_step(ui, &r, &mut opts.bg_opacity, 0.05, 0.0..=1.0);
    });
    ui.horizontal(|ui| {
        ui.label("Deckung bei Min-Zoom:");
        let r = ui.add(
            egui::Slider::new(&mut opts.bg_opacity_at_min_zoom, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.bg_opacity_at_min_zoom, 0.05, 0.0..=1.0);
    });
    ui.horizontal(|ui| {
        ui.label("Fade-out ab Zoom:");
        let r = ui.add(
            egui::DragValue::new(&mut opts.bg_fade_start_zoom)
                .range(0.1..=50.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.bg_fade_start_zoom, 0.5, 0.1..=50.0);
    });
    changed
}

/// Rendert die Uebersichtskarten-Layer-Einstellungen (Hillshade, Farmlands, POIs).
pub(super) fn render_overview_layers(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
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
    changed
}

/// Rendert die Node-Verhalten-Einstellungen (Reconnect beim Loeschen, Verbindung teilen).
pub(super) fn render_node_behavior(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    if ui
        .checkbox(&mut opts.reconnect_on_delete, "Nach Loeschen verbinden")
        .on_hover_text(
            "Wenn aktiviert: Wird ein Node mit jeweils genau einem Vorgaenger und Nachfolger \
             geloescht, werden Vorgaenger und Nachfolger direkt miteinander verbunden.",
        )
        .changed()
    {
        changed = true;
    }
    if ui
        .checkbox(
            &mut opts.split_connection_on_place,
            "Verbindung beim Platzieren teilen",
        )
        .on_hover_text(
            "Wenn aktiviert: Wird ein neuer Node nahe einer bestehenden Verbindung \
             platziert, wird diese Verbindung durch den neuen Node aufgeteilt.",
        )
        .changed()
    {
        changed = true;
    }
    changed
}

/// Rendert die Copy/Paste-Einstellungen (Vorschau-Deckung).
pub(super) fn render_copy_paste(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Vorschau-Deckung:").on_hover_text(
            "Transparenz der Paste-Vorschau im Viewport (0 = unsichtbar, 1 = volle Deckkraft).",
        );
        let r = ui.add(
            egui::Slider::new(&mut opts.copy_preview_opacity, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.copy_preview_opacity, 0.05, 0.0..=1.0);
    });
    changed
}
