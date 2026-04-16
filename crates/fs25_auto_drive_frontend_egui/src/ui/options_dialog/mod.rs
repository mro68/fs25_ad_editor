//! Optionen-Dialog fuer Farben, Groessen und Breiten.

mod sections;

use crate::app::ui_contract::{OptionsPanelAction, PanelAction};
use crate::shared::{t, EditorOptions, I18nKey, Language};

/// Navigationsbereiche im Optionen-Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionsSection {
    General,
    Nodes,
    Tools,
    Connections,
    Behavior,
    Overview,
}

impl OptionsSection {
    const ALL: [Self; 6] = [
        Self::General,
        Self::Nodes,
        Self::Tools,
        Self::Connections,
        Self::Behavior,
        Self::Overview,
    ];

    fn title(self, lang: Language) -> &'static str {
        t(
            lang,
            match self {
                Self::General => I18nKey::OptSectionGeneral,
                Self::Nodes => I18nKey::OptSectionNodes,
                Self::Tools => I18nKey::OptSectionTools,
                Self::Connections => I18nKey::OptSectionConnections,
                Self::Behavior => I18nKey::OptSectionBehavior,
                Self::Overview => I18nKey::OptSectionOverview,
            },
        )
    }

    fn subtitle(self, lang: Language) -> &'static str {
        t(
            lang,
            match self {
                Self::General => I18nKey::OptSubtitleGeneral,
                Self::Nodes => I18nKey::OptSubtitleNodes,
                Self::Tools => I18nKey::OptSubtitleTools,
                Self::Connections => I18nKey::OptSubtitleConnections,
                Self::Behavior => I18nKey::OptSubtitleBehavior,
                Self::Overview => I18nKey::OptSubtitleOverview,
            },
        )
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
    lang: Language,
) -> bool {
    let mut changed = false;

    match section {
        OptionsSection::General => {
            changed |=
                render_subsection(ui, t(lang, I18nKey::OptSubSectionSelection), None, |ui| {
                    sections::render_selection(ui, opts, lang)
                });
            changed |= render_subsection(ui, t(lang, I18nKey::OptSubSectionMarker), None, |ui| {
                sections::render_markers(ui, opts, lang)
            });
            changed |= render_subsection(ui, t(lang, I18nKey::OptSubSectionCamera), None, |ui| {
                sections::render_camera(ui, opts, lang)
            });
            changed |= render_subsection(
                ui,
                t(lang, I18nKey::OptSubSectionLod),
                Some(t(lang, I18nKey::OptSubSectionLodDesc)),
                |ui| sections::render_lod(ui, opts, lang),
            );
            changed |=
                render_subsection(ui, t(lang, I18nKey::OptSubSectionBackground), None, |ui| {
                    sections::render_background(ui, opts, lang)
                });
            changed |=
                render_subsection(ui, t(lang, I18nKey::OptSubSectionCopyPaste), None, |ui| {
                    sections::render_copy_paste(ui, opts, lang)
                });
        }
        OptionsSection::Nodes => {
            changed |= sections::render_nodes(ui, opts, lang);
        }
        OptionsSection::Tools => {
            changed |= sections::render_tools(ui, opts, lang);
        }
        OptionsSection::Connections => {
            changed |= sections::render_connections(ui, opts, lang);
        }
        OptionsSection::Behavior => {
            changed |= sections::render_node_behavior(ui, opts, lang);
        }
        OptionsSection::Overview => {
            changed |=
                render_subsection(ui, t(lang, I18nKey::OptOverviewDefaultLayers), None, |ui| {
                    sections::render_overview_layers(ui, opts, lang)
                });
            changed |=
                render_subsection(ui, t(lang, I18nKey::OptOverviewPolygonSource), None, |ui| {
                    sections::render_overview_source(ui, opts, lang)
                });
        }
    }

    changed
}

/// Zeigt den Options-Dialog und gibt erzeugte Events zurueck.
pub fn show_options_dialog(
    ctx: &egui::Context,
    show: bool,
    options: &EditorOptions,
) -> Vec<PanelAction> {
    let mut events = Vec::new();

    if !show {
        return events;
    }

    // Arbeitskopie der Optionen fuer Live-Bearbeitung
    let mut opts = options.clone();
    let lang = opts.language;
    let mut changed = false;
    let selected_section_id = egui::Id::new("options_dialog_selected_section");
    let mut selected_section = ctx.data_mut(|data| {
        data.get_temp::<OptionsSection>(selected_section_id)
            .unwrap_or(OptionsSection::General)
    });

    egui::Window::new(t(lang, I18nKey::OptDialogTitle))
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
                    ui.label(egui::RichText::new(t(lang, I18nKey::OptNavHeader)).strong());
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("options_dialog_nav")
                        .max_height(460.0)
                        .show(ui, |ui| {
                            for section in OptionsSection::ALL {
                                let selected = section == selected_section;
                                if ui.selectable_label(selected, section.title(lang)).clicked() {
                                    selected_section = section;
                                }
                            }
                        });
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.set_width(560.0);
                    ui.label(egui::RichText::new(selected_section.title(lang)).heading());
                    ui.label(selected_section.subtitle(lang));
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("options_dialog_content")
                        .max_height(430.0)
                        .show(ui, |ui| {
                            changed |=
                                render_selected_section(ui, &mut opts, selected_section, lang);
                        });
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(t(lang, I18nKey::OptLanguageLabel));
                egui::ComboBox::from_id_salt("language_select")
                    .selected_text(opts.language.display_name())
                    .show_ui(ui, |ui| {
                        for &l in Language::all() {
                            if ui
                                .selectable_value(&mut opts.language, l, l.display_name())
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(t(lang, I18nKey::DialogDefaults)).clicked() {
                    events.push(PanelAction::Options(OptionsPanelAction::ResetToDefaults));
                }
                if ui.button(t(lang, I18nKey::DialogClose)).clicked() {
                    events.push(PanelAction::Options(OptionsPanelAction::Close));
                }
            });
        });

    ctx.data_mut(|data| {
        data.insert_temp(selected_section_id, selected_section);
    });

    // Aenderungen sofort anwenden (Live-Preview)
    if changed {
        events.push(PanelAction::Options(OptionsPanelAction::Apply(Box::new(
            opts,
        ))));
    }

    events
}
