//! Gemeinsame Route-Tool-Gruppierungen fuer Katalog und Floating-Menue.

/// Gemeinsame Anzeige-Gruppe eines Route-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolGroup {
    /// Grundlegende Streckenwerkzeuge.
    Basics,
    /// Abschnitts- und Generator-Werkzeuge.
    Section,
    /// Analyse-Werkzeuge.
    Analysis,
}
