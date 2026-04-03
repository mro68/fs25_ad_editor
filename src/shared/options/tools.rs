//! Tool- und UI-Eingabeparameter fuer den Editor.

use serde::{Deserialize, Serialize};

/// Standard-Snap-Radius-Skalierung in Prozent der Node-Groesse.
pub const SNAP_SCALE_PERCENT: f32 = 100.0;
/// Standard-Hitbox-Skalierung in Prozent der Node-Groesse.
pub const HITBOX_SCALE_PERCENT: f32 = 100.0;
/// Schrittweite fuer Distanz-Felder bei Mausrad-Anpassung in Metern.
pub const MOUSE_WHEEL_DISTANCE_STEP_M: f32 = 0.1;

/// Praeferenz fuer die primaere Interaktion an numerischen DragValue-Feldern.
///
/// Hinweis: Mausrad-Anpassungen in numerischen Feldern bleiben zusaetzlich aktiv.
/// Dieser Modus dient nicht als globaler Schalter fuer Wheel-Unterstuetzung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValueAdjustInputMode {
    /// Standard-egui-DragValue-Verhalten: LMT nach links/rechts.
    DragHorizontal,
    /// Mausrad ueber dem Feld: hoch = erhoehen, runter = verringern.
    #[default]
    MouseWheel,
}
