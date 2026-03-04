//! Optionen-Dialog für Farben, Größen und Breiten.

mod sections;

use crate::app::AppIntent;
use crate::shared::EditorOptions;

/// Navigationsbereiche im Optionen-Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionsSection {
    Nodes,
    Tools,
    Selection,
    Connections,
    Markers,
    Camera,
    Background,
    OverviewLayers,
    NodeBehavior,
}

impl OptionsSection {
    const ALL: [Self; 9] = [
        Self::Nodes,
        Self::Tools,
        Self::Selection,
        Self::Connections,
        Self::Markers,
        Self::Camera,
        Self::Background,
        Self::OverviewLayers,
        Self::NodeBehavior,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Nodes => "Nodes",
            Self::Tools => "Tools",
            Self::Selection => "Selektion",
            Self::Connections => "Verbindungen",
            Self::Markers => "Marker",
            Self::Camera => "Kamera",
            Self::Background => "Hintergrund",
            Self::OverviewLayers => "Übersichtskarte",
            Self::NodeBehavior => "Node-Verhalten",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Nodes => "Farben, Größe und Hitbox-Einstellungen für Nodes.",
            Self::Tools => "Snap-Radius und Eingabeverhalten für Tool-Parameter.",
            Self::Selection => "Darstellung und Stil der aktiven Selektion.",
            Self::Connections => "Linienbreiten, Pfeile und Verbindungsfarben.",
            Self::Markers => "Pin-Größe und Marker-Farben.",
            Self::Camera => "Zoom-Grenzen und Zoom-Schrittweiten.",
            Self::Background => "Deckkraft und Zoom-basiertes Hintergrund-Fading.",
            Self::OverviewLayers => "Standard-Layer für die Übersichtskarten-Generierung.",
            Self::NodeBehavior => "Verhalten beim Löschen und Platzieren von Nodes.",
        }
    }
}

fn render_selected_section(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    section: OptionsSection,
) -> bool {
    match section {
        OptionsSection::Nodes => sections::render_nodes(ui, opts),
        OptionsSection::Tools => sections::render_tools(ui, opts),
        OptionsSection::Selection => sections::render_selection(ui, opts),
        OptionsSection::Connections => sections::render_connections(ui, opts),
        OptionsSection::Markers => sections::render_markers(ui, opts),
        OptionsSection::Camera => sections::render_camera(ui, opts),
        OptionsSection::Background => sections::render_background(ui, opts),
        OptionsSection::OverviewLayers => sections::render_overview_layers(ui, opts),
        OptionsSection::NodeBehavior => sections::render_node_behavior(ui, opts),
    }
}

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
    let selected_section_id = egui::Id::new("options_dialog_selected_section");
    let mut selected_section = ctx.data_mut(|data| {
        data.get_temp::<OptionsSection>(selected_section_id)
            .unwrap_or(OptionsSection::Nodes)
    });

    egui::Window::new("Optionen")
        .collapsible(true)
        .resizable(true)
        .default_width(820.0)
        .default_height(560.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.set_height(500.0);

                ui.vertical(|ui| {
                    ui.set_width(220.0);
                    ui.label(egui::RichText::new("Bereiche").strong());
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("options_dialog_nav")
                        .max_height(460.0)
                        .show(ui, |ui| {
                            for section in OptionsSection::ALL {
                                let selected = section == selected_section;
                                if ui.selectable_label(selected, section.title()).clicked() {
                                    selected_section = section;
                                }
                            }
                        });
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.set_width(560.0);
                    ui.label(egui::RichText::new(selected_section.title()).heading());
                    ui.label(selected_section.subtitle());
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("options_dialog_content")
                        .max_height(430.0)
                        .show(ui, |ui| {
                            changed |= render_selected_section(ui, &mut opts, selected_section);
                        });
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

    ctx.data_mut(|data| {
        data.insert_temp(selected_section_id, selected_section);
    });

    // Änderungen sofort anwenden (Live-Preview)
    if changed {
        events.push(AppIntent::OptionsChanged { options: opts });
    }

    events
}
