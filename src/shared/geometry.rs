//! Gemeinsame Geometrie-Hilfsfunktionen fuer layer-uebergreifende Nutzung.

/// Berechnet die Abweichung zwischen Einlauf- und Auslaufwinkel (0 = geradeaus, PI = Umkehr).
///
/// Misst, wie stark die Richtung abknickt. Der Rueckgabewert liegt im Bereich [0, PI].
pub fn angle_deviation(incoming: f32, outgoing: f32) -> f32 {
    let diff = (outgoing - incoming + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    diff.abs()
}

#[cfg(test)]
mod tests {
    use super::angle_deviation;

    #[test]
    fn angle_deviation_straight() {
        // Geradeaus: gleicher Winkel → Abweichung 0
        let dev = angle_deviation(0.0, 0.0);
        assert!(dev.abs() < 1e-6, "Geradeaus sollte 0 sein, ist {dev}");
    }

    #[test]
    fn angle_deviation_right_angle() {
        // 90°-Abweichung
        let dev = angle_deviation(0.0, std::f32::consts::FRAC_PI_2);
        let expected = std::f32::consts::FRAC_PI_2;
        assert!((dev - expected).abs() < 1e-5, "PI/2 erwartet, ist {dev}");
    }

    #[test]
    fn angle_deviation_wraparound() {
        // -170° und +170° liegen nahe beieinander → Abweichung ~20° (nicht 340°)
        let incoming = (-170_f32).to_radians();
        let outgoing = 170_f32.to_radians();
        let dev = angle_deviation(incoming, outgoing);
        let expected = 20_f32.to_radians();
        assert!(
            (dev - expected).abs() < 1e-4,
            "~20° erwartet, ist {}°",
            dev.to_degrees()
        );
    }
}
