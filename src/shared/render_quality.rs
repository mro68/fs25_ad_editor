/// Qualitätsstufe für Render-Anti-Aliasing.
///
/// Lebt im shared-Modul, da sowohl `app` als auch `render` darauf zugreifen.
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
