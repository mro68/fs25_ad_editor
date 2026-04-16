use crate::shared::{t, EditorOptions, I18nKey, Language};

/// Rendert die Uebersichtskarten-Layer-Einstellungen (Terrain, Hillshade, Farmlands, POIs).
pub fn render_overview_layers(ui: &mut egui::Ui, opts: &mut EditorOptions, lang: Language) -> bool {
    let mut changed = false;
    changed |= ui
        .checkbox(
            &mut opts.overview_layers.terrain,
            t(lang, I18nKey::OptOverviewTerrain),
        )
        .on_hover_text(t(lang, I18nKey::OptOverviewTerrainHelp))
        .changed();
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
