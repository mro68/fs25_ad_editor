//! Stage C — Sampling-Preview (Flood-Fill-Maske + Randsegmente) berechnen und cachen.

use std::sync::Arc;

use super::bump_revision;
use crate::app::tools::color_path::sampling::{
    extract_boundary_segments_from_mask, flood_fill_color_mask_from_rgb, world_to_pixel,
};
use crate::app::tools::color_path::state::{
    CachedRgbImage, ColorPathMask, ColorPathTool, SamplingPreviewCacheKey, SamplingPreviewData,
    Vec2CacheKey,
};

impl ColorPathTool {
    /// Berechnet Stage C aus Sampling-Input und Matching-Spezifikation neu.
    pub(in crate::app::tools::color_path) fn rebuild_sampling_preview(&mut self) {
        self.refresh_matching_spec();

        let Some(key) = self.sampling_preview_cache_key() else {
            self.clear_sampling_preview();
            return;
        };

        if self.cache.sampling_preview_key == Some(key) && self.sampling_preview.is_some() {
            return;
        }

        let Some(sampling_preview) = self.compute_sampling_preview() else {
            self.clear_sampling_preview();
            return;
        };

        self.sampling_preview = Some(sampling_preview);
        self.cache.sampling_preview_key = Some(key);
        bump_revision(&mut self.cache.sampling_preview_revision);
        self.clear_preview_pipeline();
    }

    /// Verwirft Stage C und alle Folgestufen (D-F).
    pub(in crate::app::tools::color_path) fn clear_sampling_preview(&mut self) {
        self.sampling_preview = None;
        self.cache.sampling_preview_key = None;
        self.clear_preview_pipeline();
    }

    /// Liefert den Cache-Schluessel fuer Stage C.
    pub(super) fn sampling_preview_cache_key(&self) -> Option<SamplingPreviewCacheKey> {
        let background_image_id = self.background_image_id()?;
        let lasso_start_world = Vec2CacheKey::from(self.sampling.lasso_start_world?);
        if self.matching.is_empty() {
            return None;
        }

        Some(SamplingPreviewCacheKey {
            background_image_id,
            map_size_bits: self.map_size.to_bits(),
            lasso_start_world,
            matching_revision: self.cache.matching_revision,
        })
    }

    /// Erzeugt eine stabile ID fuer das aktuelle Hintergrundbild (Pointer-basiert).
    pub(super) fn background_image_id(&self) -> Option<usize> {
        self.background_image
            .as_ref()
            .map(|image| Arc::as_ptr(image) as usize)
    }

    /// Liefert das RGB-Bild aus dem Cache oder baut es einmalig auf.
    pub(super) fn cached_rgb_image(&mut self) -> Option<Arc<image::RgbImage>> {
        let background_image_id = self.background_image_id()?;

        if let Some(cached) = self.cache.rgb_image.as_ref()
            && cached.background_image_id == background_image_id
        {
            return Some(Arc::clone(&cached.image));
        }

        let image = self.background_image.as_ref()?;
        let rgb_image = Arc::new(image.to_rgb8());
        self.cache.rgb_image = Some(CachedRgbImage {
            background_image_id,
            image: Arc::clone(&rgb_image),
        });
        Some(rgb_image)
    }

    /// Fuehrt den Flood-Fill und die Randextraktion fuer Stage C aus.
    pub(super) fn compute_sampling_preview(&mut self) -> Option<SamplingPreviewData> {
        let (img_w, img_h) = {
            let image = self.background_image.as_ref()?;
            (image.width(), image.height())
        };
        let rgb_image = self.cached_rgb_image()?;
        let lasso_start = self.sampling.lasso_start_world?;
        if self.matching.is_empty() {
            return None;
        }

        let start_pixel = world_to_pixel(lasso_start, self.map_size, img_w, img_h);
        let (mask, width, height) = flood_fill_color_mask_from_rgb(
            &rgb_image,
            &self.matching.palette,
            self.matching.tolerance,
            start_pixel,
        );
        let input_mask = ColorPathMask::new(mask, width, height);
        let boundary_segments = extract_boundary_segments_from_mask(
            &input_mask.pixels,
            input_mask.width,
            input_mask.height,
            self.map_size,
        );

        Some(SamplingPreviewData {
            input_mask,
            boundary_segments,
            start_pixel,
        })
    }
}
