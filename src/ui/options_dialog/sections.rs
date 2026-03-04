use crate::shared::{EditorOptions, SelectionStyle, ValueAdjustInputMode};

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

pub(super) fn render_nodes(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
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
    changed
}

pub(super) fn render_tools(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.collapsing("Tools", |ui| {
        ui.horizontal(|ui| {
            ui.label("Wertänderung:");
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
            changed |= ui
                .add(
                    egui::DragValue::new(&mut opts.snap_scale_percent)
                        .range(50.0..=2000.0)
                        .speed(10.0)
                        .suffix(" %"),
                )
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("Mausrad-Schritt Distanz:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut opts.mouse_wheel_distance_step_m)
                        .range(0.01..=5.0)
                        .speed(0.01)
                        .suffix(" m"),
                )
                .changed();
        });
    });
    changed
}

pub(super) fn render_selection(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.collapsing("Selektion", |ui| {
        ui.horizontal(|ui| {
            ui.label("Größenfaktor (%):");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut opts.selection_size_factor)
                        .range(100.0..=200.0)
                        .speed(1.0),
                )
                .changed();
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
    });
    changed
}

pub(super) fn render_connections(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
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
                    egui::DragValue::new(&mut opts.connection_thickness_subprio_world)
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
        changed |= color_edit(ui, "Regular (Einbahn):", &mut opts.connection_color_regular);
        changed |= color_edit(ui, "Dual (Bidirektional):", &mut opts.connection_color_dual);
        changed |= color_edit(
            ui,
            "Reverse (Rückwärts):",
            &mut opts.connection_color_reverse,
        );
    });
    changed
}

pub(super) fn render_markers(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
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
    changed
}

pub(super) fn render_camera(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
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
    changed
}

pub(super) fn render_background(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.collapsing("Hintergrund", |ui| {
        ui.horizontal(|ui| {
            ui.label("Standard-Deckung:");
            changed |= ui
                .add(
                    egui::Slider::new(&mut opts.bg_opacity, 0.0..=1.0)
                        .step_by(0.05)
                        .fixed_decimals(2),
                )
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("Deckung bei Min-Zoom:");
            changed |= ui
                .add(
                    egui::Slider::new(&mut opts.bg_opacity_at_min_zoom, 0.0..=1.0)
                        .step_by(0.05)
                        .fixed_decimals(2),
                )
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("Fade-out ab Zoom:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut opts.bg_fade_start_zoom)
                        .range(0.1..=50.0)
                        .speed(0.1),
                )
                .changed();
        });
    });
    changed
}

pub(super) fn render_overview_layers(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
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
    changed
}

pub(super) fn render_node_behavior(ui: &mut egui::Ui, opts: &mut EditorOptions) -> bool {
    let mut changed = false;
    ui.collapsing("Node-Verhalten", |ui| {
        if ui
            .checkbox(&mut opts.reconnect_on_delete, "Nach Löschen verbinden")
            .on_hover_text(
                "Wenn aktiviert: Wird ein Node mit jeweils genau einem Vorgänger und Nachfolger \
                 gelöscht, werden Vorgänger und Nachfolger direkt miteinander verbunden.",
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
    });
    changed
}
