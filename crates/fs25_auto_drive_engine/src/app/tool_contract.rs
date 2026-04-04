//! App-weiter Vertrag fuer Route-Tool-Identitaeten, Anker- und Tangentendaten.

use glam::Vec2;

/// Stabile Identitaet eines Route-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteToolId {
    /// Gerade Strecke.
    Straight,
    /// Quadratische Bezier-Kurve.
    CurveQuad,
    /// Kubische Bezier-Kurve.
    CurveCubic,
    /// Catmull-Rom-Spline.
    Spline,
    /// Ausweichstrecke.
    Bypass,
    /// Geglaettete Kurve.
    SmoothCurve,
    /// Parkplatz-Generator.
    Parking,
    /// Feldgrenzen-Analyse.
    FieldBoundary,
    /// Feldweg-Analyse.
    FieldPath,
    /// Strecken-Versatz.
    RouteOffset,
    /// Farb-Pfad-Analyse.
    ColorPath,
}

impl RouteToolId {
    /// Alle registrierten Route-Tools in kanonischer Slot-Reihenfolge.
    pub const ALL: [Self; 11] = [
        Self::Straight,
        Self::CurveQuad,
        Self::CurveCubic,
        Self::Spline,
        Self::Bypass,
        Self::SmoothCurve,
        Self::Parking,
        Self::FieldBoundary,
        Self::FieldPath,
        Self::RouteOffset,
        Self::ColorPath,
    ];
}

/// Anker-Punkt: entweder ein existierender Node oder eine freie Position.
#[derive(Debug, Clone, Copy)]
pub enum ToolAnchor {
    /// Snap auf existierenden Node.
    ExistingNode(u64, Vec2),
    /// Freie Position, an der spaeter ein neuer Node erstellt wird.
    NewPosition(Vec2),
}

impl ToolAnchor {
    /// Gibt die Welt-Position des Ankers zurueck.
    pub fn position(&self) -> Vec2 {
        match self {
            Self::ExistingNode(_, pos) | Self::NewPosition(pos) => *pos,
        }
    }
}

/// Quelle einer Tangente am Start- oder Endpunkt eines Route-Tools.
///
/// Wird von Curve- und Spline-Tool verwendet, um Kontroll- oder Phantom-Punkte
/// tangential an einer bestehenden Verbindung auszurichten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TangentSource {
    /// Kein Tangenten-Vorschlag — Punkt wird manuell gesetzt.
    None,
    /// Tangente aus bestehender Verbindung.
    Connection { neighbor_id: u64, angle: f32 },
}
