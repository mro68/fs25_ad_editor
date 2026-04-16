use crate::shared::{EditorOptions, Language};
use crate::ui::common::{
    overview_field_detection_source_label, OVERVIEW_FIELD_DETECTION_SOURCE_ORDER,
};

pub fn render_overview_source(
    ui: &mut egui::Ui,
    opts: &mut EditorOptions,
    _lang: Language,
) -> bool {
    let mut changed = false;

    for source in OVERVIEW_FIELD_DETECTION_SOURCE_ORDER {
        changed |= ui
            .radio_value(
                &mut opts.overview_field_detection_source,
                source,
                overview_field_detection_source_label(source),
            )
            .changed();
    }

    changed
}
