//! Stage D/E — Preview-Core: Maske vorbereiten und Skelett-Netzwerk aufbauen.

use super::bump_revision;
use crate::app::tools::color_path::sampling::prepare_mask_for_skeleton;
use crate::app::tools::color_path::skeleton::extract_network_from_mask;
use crate::app::tools::color_path::state::{
    ColorPathMask, ColorPathTool, PreviewCoreCacheKey, PreviewData,
};

impl ColorPathTool {
    /// Baut Stage D/E (Maskenvorbereitung + Skelett-Extraktion) aus Stage C neu auf.
    pub(super) fn ensure_preview_core(&mut self) -> bool {
        self.rebuild_sampling_preview();

        let Some(key) = self.preview_core_cache_key() else {
            self.clear_preview_pipeline();
            return false;
        };

        if self.cache.preview_core_key == Some(key) && self.preview_data.is_some() {
            return true;
        }

        let Some(sampling_preview) = self.sampling_preview.as_ref() else {
            self.clear_preview_pipeline();
            return false;
        };
        if sampling_preview.input_mask.is_empty() {
            self.clear_preview_pipeline();
            return false;
        }

        let prepared_pixels = prepare_mask_for_skeleton(
            &sampling_preview.input_mask.pixels,
            sampling_preview.input_mask.width as usize,
            sampling_preview.input_mask.height as usize,
            self.config.noise_filter,
        );
        let prepared_mask = ColorPathMask::new(
            prepared_pixels,
            sampling_preview.input_mask.width,
            sampling_preview.input_mask.height,
        );
        let start_hint = Some((
            sampling_preview.start_pixel.0 as usize,
            sampling_preview.start_pixel.1 as usize,
        ));
        let network = extract_network_from_mask(
            &prepared_mask.pixels,
            prepared_mask.width,
            prepared_mask.height,
            self.map_size,
            start_hint,
        );

        log::info!(
            "Pipeline complete: {} junctions, {} open ends, {} segments",
            network.junction_count(),
            network.open_end_count(),
            network.segments.len()
        );

        if network.is_empty() {
            self.clear_preview_pipeline();
            return false;
        }

        self.preview_data = Some(PreviewData {
            prepared_mask,
            network,
            prepared_segments: Vec::new(),
        });
        self.cache.preview_core_key = Some(key);
        bump_revision(&mut self.cache.preview_core_revision);
        self.cache.prepared_segments_key = None;
        true
    }

    /// Liefert den Cache-Schluessel fuer Stage D/E.
    pub(super) fn preview_core_cache_key(&self) -> Option<PreviewCoreCacheKey> {
        self.sampling_preview.as_ref()?;
        Some(PreviewCoreCacheKey {
            sampling_preview_revision: self.cache.sampling_preview_revision,
            noise_filter: self.config.noise_filter,
        })
    }
}
