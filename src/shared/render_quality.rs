//! Render-Qualitätsstufen für Anti-Aliasing (shared zwischen App und Renderer).

/// Qualitätsstufe für Render-Anti-Aliasing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderQuality {
    /// Minimales Anti-Aliasing (schnellste Darstellung)
    Low,
    /// Mittleres Anti-Aliasing
    Medium,
    /// Maximales Anti-Aliasing (beste Darstellung)
    #[default]
    High,
}
