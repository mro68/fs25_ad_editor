//! Stage B — Matching-Spezifikation aus den Sampling-Eingaben ableiten.

use super::bump_revision;
use crate::app::tools::color_path::sampling::{build_color_palette, build_exact_color_set};
use crate::app::tools::color_path::state::{ColorPathTool, MatchingCacheKey, MatchingSpec};

impl ColorPathTool {
    /// Berechnet Stage B aus den aktuellen Sampling-Eingaben neu.
    pub(in crate::app::tools::color_path) fn refresh_matching_spec(&mut self) {
        let key = self.matching_cache_key();
        if self.cache.matching_key == Some(key) {
            return;
        }

        let palette = if self.config.exact_color_match {
            build_exact_color_set(&self.sampling.sampled_colors)
        } else {
            build_color_palette(&self.sampling.sampled_colors, 8)
        };
        let tolerance = if self.config.exact_color_match {
            0.0
        } else {
            self.config.color_tolerance
        };

        let next_matching = MatchingSpec { tolerance, palette };
        let matching_changed = self.matching != next_matching;

        self.matching = next_matching;
        self.cache.matching_key = Some(key);

        if matching_changed {
            bump_revision(&mut self.cache.matching_revision);
            self.clear_sampling_preview();
        }
    }

    /// Liefert den Cache-Schluessel fuer Stage B.
    pub(super) fn matching_cache_key(&self) -> MatchingCacheKey {
        MatchingCacheKey {
            sampling_revision: self.cache.sampling_revision,
            exact_color_match: self.config.exact_color_match,
            color_tolerance_bits: if self.config.exact_color_match {
                0
            } else {
                self.config.color_tolerance.to_bits()
            },
        }
    }
}
