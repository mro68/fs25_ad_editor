//! Einzelne Einstellungs-Abschnitte fuer den Options-Dialog.
//!
//! Jede `render_*`-Funktion rendert einen thematischen Block und gibt `true`
//! zurueck wenn sich ein Wert geaendert hat.

use crate::shared::{t, EditorOptions, I18nKey, Language, SelectionStyle, ValueAdjustInputMode};
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
pub(super) fn render_nodes(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptNodeSizeWorld));
        let r = ui.add(
            egui::DragValue::new(&mut opts.node_size_world)
                .range(0.1..=5.0)
                .speed(0.01),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.node_size_world, 0.1, 0.1..=5.0);
        r.on_hover_text(t(lang, I18nKey::OptNodeSizeWorldHelp));
    });
    changed |= color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorDefault),
        &mut opts.node_color_default,
    );
    changed |= color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorSubprio),
        &mut opts.node_color_subprio,
    );
    changed |= color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorSelected),
        &mut opts.node_color_selected,
    );
    changed |= color_edit(
        ui,
        t(lang, I18nKey::OptNodeColorWarning),
        &mut opts.node_color_warning,
    );
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptHitboxScale));
        let r = ui.add(
            egui::DragValue::new(&mut opts.hitbox_scale_percent)
                .range(50.0..=500.0)
                .speed(5.0)
                .suffix(" %"),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.hitbox_scale_percent, 10.0, 50.0..=500.0);
        r.on_hover_text(t(lang, I18nKey::OptHitboxScaleHelp));
    });
    changed
}

/// Rendert die Werkzeug-Einstellungen (Eingabemodus, Snap-Radius, Mausrad-Schritt).
pub(super) fn render_tools(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptValueAdjustMode));
        let current_label = match opts.value_adjust_input_mode {
            ValueAdjustInputMode::DragHorizontal => t(lang, I18nKey::OptValueAdjustDrag),
            ValueAdjustInputMode::MouseWheel => t(lang, I18nKey::OptValueAdjustWheel),
        };
        egui::ComboBox::from_id_salt("value_adjust_input_mode")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_value(
                        &mut opts.value_adjust_input_mode,
                        ValueAdjustInputMode::DragHorizontal,
                        t(lang, I18nKey::OptValueAdjustDrag),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .selectable_value(
                        &mut opts.value_adjust_input_mode,
                        ValueAdjustInputMode::MouseWheel,
                        t(lang, I18nKey::OptValueAdjustWheel),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
    })
    .response
    .on_hover_text(t(lang, I18nKey::OptValueAdjustModeHelp));
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSnapRadius));
        let r = ui.add(
            egui::DragValue::new(&mut opts.snap_scale_percent)
                .range(50.0..=2000.0)
                .speed(10.0)
                .suffix(" %"),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.snap_scale_percent, 10.0, 50.0..=2000.0);
        r.on_hover_text(t(lang, I18nKey::OptSnapRadiusHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptMouseWheelDistStep));
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
        r.on_hover_text(t(lang, I18nKey::OptMouseWheelDistStepHelp));
    });
    changed
}

/// Rendert die Selektions-Einstellungen (Groessenfaktor, Markierungsstil).
pub(super) fn render_selection(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    lang: Language,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSelectionSizeFactor));
        let r = ui.add(
            egui::DragValue::new(&mut opts.selection_size_factor)
                .range(100.0..=200.0)
                .speed(1.0),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.selection_size_factor, 5.0, 100.0..=200.0);
        r.on_hover_text(t(lang, I18nKey::OptSelectionSizeFactorHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSelectionStyle));
        let current_label = match opts.selection_style {
            SelectionStyle::Ring => t(lang, I18nKey::OptSelectionStyleRing),
            SelectionStyle::Gradient => t(lang, I18nKey::OptSelectionStyleGradient),
        };
        egui::ComboBox::from_id_salt("selection_style")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for (style, label) in [
                    (
                        SelectionStyle::Ring,
                        t(lang, I18nKey::OptSelectionStyleRing),
                    ),
                    (
                        SelectionStyle::Gradient,
                        t(lang, I18nKey::OptSelectionStyleGradient),
                    ),
                ] {
                    if ui
                        .selectable_value(&mut opts.selection_style, style, label)
                        .changed()
                    {
                        changed = true;
                    }
                }
            });
    })
    .response
    .on_hover_text(t(lang, I18nKey::OptSelectionStyleHelp));
    ui.separator();
    ui.label(t(lang, I18nKey::OptDoubleClickSegment));
    ui.horizontal(|ui| {
        changed |= ui
            .checkbox(
                &mut opts.segment_stop_at_junction,
                t(lang, I18nKey::OptSegmentStopAtJunction),
            )
            .on_hover_text(t(lang, I18nKey::OptSegmentStopAtJunctionHelp))
            .changed();
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptSegmentMaxAngle));
        let r = ui.add(
            egui::DragValue::new(&mut opts.segment_max_angle_deg)
                .range(0.0..=180.0)
                .speed(1.0),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.segment_max_angle_deg, 5.0, 0.0..=180.0);
        r.on_hover_text(t(lang, I18nKey::OptSegmentMaxAngleHelp));
        if opts.segment_max_angle_deg == 0.0 {
            ui.weak(t(lang, I18nKey::OptSegmentDisabled));
        }
    });
    changed
}

/// Rendert die Verbindungs-Darstellungseinstellungen (Breite, Pfeilgroessen, Farben).
pub(super) fn render_connections(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    lang: Language,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptConnectionWidthMain));
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
        r.on_hover_text(t(lang, I18nKey::OptConnectionWidthMainHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptConnectionWidthSubprio));
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
        r.on_hover_text(t(lang, I18nKey::OptConnectionWidthSubprioHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptArrowLength));
        let r = ui.add(
            egui::DragValue::new(&mut opts.arrow_length_world)
                .range(0.1..=5.0)
                .speed(0.05),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.arrow_length_world, 0.5, 0.1..=5.0);
        r.on_hover_text(t(lang, I18nKey::OptArrowLengthHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptArrowWidth));
        let r = ui.add(
            egui::DragValue::new(&mut opts.arrow_width_world)
                .range(0.1..=5.0)
                .speed(0.05),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.arrow_width_world, 0.5, 0.1..=5.0);
        r.on_hover_text(t(lang, I18nKey::OptArrowWidthHelp));
    });
    changed |= color_edit(
        ui,
        t(lang, I18nKey::OptConnectionColorRegular),
        &mut opts.connection_color_regular,
    );
    changed |= color_edit(
        ui,
        t(lang, I18nKey::OptConnectionColorDual),
        &mut opts.connection_color_dual,
    );
    changed |= color_edit(
        ui,
        t(lang, I18nKey::OptConnectionColorReverse),
        &mut opts.connection_color_reverse,
    );
    changed
}

/// Rendert die Marker-Darstellungseinstellungen (Pin-Groesse, Farben, Umrissstaerke).
pub(super) fn render_markers(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(egui::include_image!(
                "../../../assets/icons/icon_map_pin.svg"
            ))
            .fit_to_exact_size(egui::Vec2::new(14.0, 14.0)),
        );
        ui.label(t(lang, I18nKey::OptMarkerSize));
        let r = ui.add(
            egui::DragValue::new(&mut opts.marker_size_world)
                .range(0.5..=10.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.marker_size_world, 1.0, 0.5..=10.0);
        r.on_hover_text(t(lang, I18nKey::OptMarkerSizeHelp));
    });
    changed |= color_edit(ui, t(lang, I18nKey::OptMarkerColor), &mut opts.marker_color);
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptMarkerOutlineWidth));
        let r = ui.add(
            egui::DragValue::new(&mut opts.marker_outline_width)
                .range(0.01..=0.3)
                .speed(0.005)
                .fixed_decimals(3),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.marker_outline_width, 0.01, 0.01..=0.3);
        r.on_hover_text(t(lang, I18nKey::OptMarkerOutlineWidthHelp));
    });
    changed
}

/// Rendert die Kamera-Einstellungen (Zoom-Grenzen, Scroll-Schritt, Kompensation).
pub(super) fn render_camera(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraZoomMin));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_min)
                .range(0.01..=10.0)
                .speed(0.01),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.camera_zoom_min, 0.1, 0.01..=10.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraZoomMinHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraZoomMax));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_max)
                .range(1.0..=1000.0)
                .speed(1.0),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.camera_zoom_max, 5.0, 1.0..=1000.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraZoomMaxHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraZoomStep));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_zoom_step)
                .range(1.01..=3.0)
                .speed(0.01),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.camera_zoom_step, 0.05, 1.01..=3.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraZoomStepHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCameraScrollZoomStep));
        let r = ui.add(
            egui::DragValue::new(&mut opts.camera_scroll_zoom_step)
                .range(1.01..=2.0)
                .speed(0.01),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.camera_scroll_zoom_step, 0.05, 1.01..=2.0);
        r.on_hover_text(t(lang, I18nKey::OptCameraScrollZoomStepHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptZoomCompensationMax));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.zoom_compensation_max, 1.0..=8.0)
                    .step_by(0.1)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptZoomCompensationMaxHelp));
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.zoom_compensation_max, 0.1, 1.0..=8.0);
    });
    changed
}

/// Rendert die LOD/Mindestgroessen-Einstellungen (Pixel-Untergrenzen + Node-Decimation).
pub(super) fn render_lod(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    ui.label(t(lang, I18nKey::OptLodMinSizes));
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodNodes));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_node_size_px, 0.0..=20.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptLodNodesHelp));
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_node_size_px, 1.0, 0.0..=20.0);
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodConnections));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_connection_width_px, 0.0..=10.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptLodConnectionsHelp));
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.min_connection_width_px, 0.5, 0.0..=10.0);
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodArrows));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_arrow_size_px, 0.0..=20.0)
                    .step_by(0.5)
                    .fixed_decimals(1),
            )
            .on_hover_text(t(lang, I18nKey::OptLodArrowsHelp));
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_arrow_size_px, 1.0, 0.0..=20.0);
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodMarkers));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.min_marker_size_px, 0.0..=30.0)
                    .step_by(1.0)
                    .fixed_decimals(0),
            )
            .on_hover_text(t(lang, I18nKey::OptLodMarkersHelp));
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.min_marker_size_px, 1.0, 0.0..=30.0);
    });
    ui.separator();
    ui.label(t(lang, I18nKey::OptLodNodeDecimation));
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptLodDecimationSpacing));
        let r = ui
            .add(
                egui::Slider::new(&mut opts.node_decimation_spacing_px, 0.0..=50.0)
                    .step_by(1.0)
                    .fixed_decimals(0),
            )
            .on_hover_text(t(lang, I18nKey::OptLodDecimationSpacingHelp));
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
pub(super) fn render_background(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    lang: Language,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptBgOpacity));
        let r = ui.add(
            egui::Slider::new(&mut opts.bg_opacity, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |= r.changed() | apply_wheel_step(ui, &r, &mut opts.bg_opacity, 0.05, 0.0..=1.0);
        r.on_hover_text(t(lang, I18nKey::OptBgOpacityHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptBgOpacityAtMinZoom));
        let r = ui.add(
            egui::Slider::new(&mut opts.bg_opacity_at_min_zoom, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |= r.changed()
            | apply_wheel_step(ui, &r, &mut opts.bg_opacity_at_min_zoom, 0.05, 0.0..=1.0);
        r.on_hover_text(t(lang, I18nKey::OptBgOpacityAtMinZoomHelp));
    });
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptBgFadeStartZoom));
        let r = ui.add(
            egui::DragValue::new(&mut opts.bg_fade_start_zoom)
                .range(0.1..=50.0)
                .speed(0.1),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.bg_fade_start_zoom, 0.5, 0.1..=50.0);
        r.on_hover_text(t(lang, I18nKey::OptBgFadeStartZoomHelp));
    });
    changed
}

/// Rendert die Uebersichtskarten-Layer-Einstellungen (Hillshade, Farmlands, POIs).
pub(super) fn render_overview_layers(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    lang: Language,
) -> bool {
    let mut changed = false;
    changed |= ui
        .checkbox(
            &mut opts.overview_layers.hillshade,
            t(lang, I18nKey::OptOverviewHillshade),
        )
        .on_hover_text(t(lang, I18nKey::OptOverviewHillshadeHelp))
        .changed();
    changed |= ui
        .checkbox(
            &mut opts.overview_layers.farmlands,
            t(lang, I18nKey::OptOverviewFarmlands),
        )
        .on_hover_text(t(lang, I18nKey::OptOverviewFarmlandsHelp))
        .changed();
    changed |= ui
        .checkbox(
            &mut opts.overview_layers.farmland_ids,
            t(lang, I18nKey::OptOverviewFarmlandIds),
        )
        .on_hover_text(t(lang, I18nKey::OptOverviewFarmlandIdsHelp))
        .changed();
    changed |= ui
        .checkbox(
            &mut opts.overview_layers.pois,
            t(lang, I18nKey::OptOverviewPois),
        )
        .on_hover_text(t(lang, I18nKey::OptOverviewPoisHelp))
        .changed();
    changed |= ui
        .checkbox(
            &mut opts.overview_layers.legend,
            t(lang, I18nKey::OptOverviewLegend),
        )
        .on_hover_text(t(lang, I18nKey::OptOverviewLegendHelp))
        .changed();
    changed
}

/// Rendert die Node-Verhalten-Einstellungen (Reconnect beim Loeschen, Verbindung teilen).
pub(super) fn render_node_behavior(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    lang: Language,
) -> bool {
    let mut changed = false;
    if ui
        .checkbox(
            &mut opts.reconnect_on_delete,
            t(lang, I18nKey::OptReconnectOnDelete),
        )
        .on_hover_text(t(lang, I18nKey::OptReconnectOnDeleteHelp))
        .changed()
    {
        changed = true;
    }
    if ui
        .checkbox(
            &mut opts.split_connection_on_place,
            t(lang, I18nKey::OptSplitConnectionOnPlace),
        )
        .on_hover_text(t(lang, I18nKey::OptSplitConnectionOnPlaceHelp))
        .changed()
    {
        changed = true;
    }
    changed
}

/// Rendert die Copy/Paste-Einstellungen (Vorschau-Deckung).
pub(super) fn render_copy_paste(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    lang: Language,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(t(lang, I18nKey::OptCopyPastePreviewOpacity));
        let r = ui.add(
            egui::Slider::new(&mut opts.copy_preview_opacity, 0.0..=1.0)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        changed |=
            r.changed() | apply_wheel_step(ui, &r, &mut opts.copy_preview_opacity, 0.05, 0.0..=1.0);
        r.on_hover_text(t(lang, I18nKey::OptCopyPastePreviewOpacityHelp));
    });
    changed
}
