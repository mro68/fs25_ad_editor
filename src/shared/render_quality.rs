//! Render-Qualitaetsstufen fuer Anti-Aliasing (shared zwischen App und Renderer).

/// Qualitaetsstufe fuer Render-Anti-Aliasing.
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
