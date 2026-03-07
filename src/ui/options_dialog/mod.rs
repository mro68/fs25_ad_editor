//! Optionen-Dialog fuer Farben, Groessen und Breiten.

mod sections;

use crate::app::AppIntent;
use crate::shared::EditorOptions;

/// Navigationsbereiche im Optionen-Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionsSection {
    General,
    Nodes,
    Tools,
    Connections,
    Behavior,
}

impl OptionsSection {
    const ALL: [Self; 5] = [
        Self::General,
        Self::Nodes,
        Self::Tools,
        Self::Connections,
        Self::Behavior,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::General => "Allgemein",
            Self::Nodes => "Nodes",
            Self::Tools => "Tools",
            Self::Connections => "Verbindungen",
            Self::Behavior => "Verhalten",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::General => "Globale Anzeige- und Karten-Einstellungen.",
            Self::Nodes => "Farben, Groesse und Hitbox-Einstellungen fuer Nodes.",
            Self::Tools => "Snap-Radius und Eingabeverhalten fuer Tool-Parameter.",
            Self::Connections => "Linienbreiten, Pfeile und Verbindungsfarben.",
            Self::Behavior => "Verhalten beim Loeschen und Platzieren von Nodes.",
        }
    }
}

fn render_subsection(
    ui: &mut egui::Ui,
    title: &str,
    description: Option<&str>,
    render: impl FnOnce(&mut egui::Ui) -> bool,
) -> bool {
    let mut changed = false;
    ui.label(egui::RichText::new(title).strong());
    if let Some(text) = description {
        ui.label(text);
    }
    ui.add_space(4.0);
    changed |= render(ui);
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
    changed
}

fn render_selected_section(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    section: OptionsSection,
) -> bool {
    let mut changed = false;

    match section {
        OptionsSection::General => {
            changed |= render_subsection(ui, "Selektion", None, |ui| {
                sections::render_selection(ui, opts)
            });
            changed |=
                render_subsection(ui, "Marker", None, |ui| sections::render_markers(ui, opts));
            changed |=
                render_subsection(ui, "Kamera", None, |ui| sections::render_camera(ui, opts));
            changed |= render_subsection(
                ui,
                "LOD / Mindestgroessen",
                Some("Pixel-Untergrenzen und Node-Ausdünnung beim Herauszoomen."),
                |ui| sections::render_lod(ui, opts),
            );
            changed |= render_subsection(ui, "Hintergrund", None, |ui| {
                sections::render_background(ui, opts)
            });
            changed |= render_subsection(ui, "Copy/Paste-Vorschau", None, |ui| {
                sections::render_copy_paste(ui, opts)
            });
            changed |= render_subsection(ui, "Uebersichtskarte (Standard-Layer)", None, |ui| {
                sections::render_overview_layers(ui, opts)
            });
        }
        OptionsSection::Nodes => {
            changed |= sections::render_nodes(ui, opts);
        }
        OptionsSection::Tools => {
            changed |= sections::render_tools(ui, opts);
        }
        OptionsSection::Connections => {
            changed |= sections::render_connections(ui, opts);
        }
        OptionsSection::Behavior => {
            changed |= sections::render_node_behavior(ui, opts);
        }
    }

    changed
}

/// Zeigt den Options-Dialog und gibt erzeugte Events zurueck.
pub fn show_options_dialog(
    ctx: &egui::Context,
    show: bool,
    options: &EditorOptions,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !show {
        return events;
    }

    // Arbeitskopie der Optionen fuer Live-Bearbeitung
    let mut opts = options.clone();
    let mut changed = false;
    let selected_section_id = egui::Id::new("options_dialog_selected_section");
    let mut selected_section = ctx.data_mut(|data| {
        data.get_temp::<OptionsSection>(selected_section_id)
            .unwrap_or(OptionsSection::General)
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
                if ui.button("Schliessen").clicked() {
                    events.push(AppIntent::CloseOptionsDialogRequested);
                }
            });
        });

    ctx.data_mut(|data| {
        data.insert_temp(selected_section_id, selected_section);
    });

    // Aenderungen sofort anwenden (Live-Preview)
    if changed {
        events.push(AppIntent::OptionsChanged { options: opts });
    }

    events
}
