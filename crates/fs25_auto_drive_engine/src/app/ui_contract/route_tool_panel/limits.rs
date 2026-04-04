//! Gemeinsame Eingabegrenzen fuer Route-Tool-Panelwerte.

use std::ops::RangeInclusive;

/// Gemeinsame Eingabegrenzen fuer Gleitkomma-Felder im Route-Tool-Panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatInputLimits {
    min: f32,
    max: f32,
}

impl FloatInputLimits {
    /// Erstellt einen neuen Gleitkomma-Grenzwertbereich.
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    /// Klemmt einen Wert in den gueltigen Bereich.
    pub fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }

    /// Liefert den Bereich fuer egui-Widgets.
    pub fn range(self) -> RangeInclusive<f32> {
        self.min..=self.max
    }
}

/// Gemeinsame Eingabegrenzen fuer Ganzzahl-Felder im Route-Tool-Panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsizeInputLimits {
    min: usize,
    max: usize,
}

impl UsizeInputLimits {
    /// Erstellt einen neuen Ganzzahl-Grenzwertbereich.
    pub const fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }

    /// Klemmt einen Wert in den gueltigen Bereich.
    pub fn clamp(self, value: usize) -> usize {
        value.clamp(self.min, self.max)
    }

    /// Liefert den Bereich fuer egui-Widgets.
    pub fn range(self) -> RangeInclusive<usize> {
        self.min..=self.max
    }
}

/// Eingabegrenzen fuer den seitlichen Versatz des Bypass-Tools.
pub const BYPASS_OFFSET_LIMITS: FloatInputLimits = FloatInputLimits::new(-200.0, 200.0);
/// Eingabegrenzen fuer die Basisschrittweite des Bypass-Tools.
pub const BYPASS_BASE_SPACING_LIMITS: FloatInputLimits = FloatInputLimits::new(1.0, 50.0);
/// Eingabegrenzen fuer die Distanz des Route-Offset-Tools.
pub const ROUTE_OFFSET_DISTANCE_LIMITS: FloatInputLimits = FloatInputLimits::new(0.5, 200.0);
/// Eingabegrenzen fuer die Basisschrittweite des Route-Offset-Tools.
pub const ROUTE_OFFSET_BASE_SPACING_LIMITS: FloatInputLimits =
    FloatInputLimits::new(1.0, 50.0);
/// Eingabegrenzen fuer den maximalen Lenkwinkel der geglaetteten Kurve.
pub const SMOOTH_CURVE_MAX_ANGLE_LIMITS: FloatInputLimits =
    FloatInputLimits::new(5.0, 135.0);
/// Eingabegrenzen fuer den Minimalabstand der geglaetteten Kurve.
pub const SMOOTH_CURVE_MIN_DISTANCE_LIMITS: FloatInputLimits =
    FloatInputLimits::new(0.5, 20.0);
/// Eingabegrenzen fuer die Anzahl von Parkplatzreihen.
pub const PARKING_NUM_ROWS_LIMITS: UsizeInputLimits = UsizeInputLimits::new(1, 10);
/// Eingabegrenzen fuer den Reihenabstand beim Parkplatz-Tool.
pub const PARKING_ROW_SPACING_LIMITS: FloatInputLimits = FloatInputLimits::new(4.0, 20.0);
/// Eingabegrenzen fuer die Parkbuchtlaenge.
pub const PARKING_BAY_LENGTH_LIMITS: FloatInputLimits = FloatInputLimits::new(10.0, 100.0);
/// Eingabegrenzen fuer den maximalen Node-Abstand im Parkplatz-Tool.
pub const PARKING_MAX_NODE_DISTANCE_LIMITS: FloatInputLimits =
    FloatInputLimits::new(2.0, 20.0);
/// Eingabegrenzen fuer die Ein-/Ausfahrtsparameter der Parkplatz-Rampen.
pub const PARKING_ENTRY_EXIT_T_LIMITS: FloatInputLimits = FloatInputLimits::new(0.0, 1.0);
/// Eingabegrenzen fuer die Rampenlaenge des Parkplatz-Tools.
pub const PARKING_RAMP_LENGTH_LIMITS: FloatInputLimits = FloatInputLimits::new(2.0, 20.0);
/// Eingabegrenzen fuer den Rotationsschritt des Parkplatz-Tools.
pub const PARKING_ROTATION_STEP_LIMITS: FloatInputLimits = FloatInputLimits::new(0.5, 45.0);
