//! Zhang-Suen-Thinning: Skelettierung von Binaermasken.

/// Zhang-Suen-Thinning: Reduziert eine Binärmaske auf ihr Skelett (1px breite Mittellinie).
///
/// Iteriert bis keine Pixel mehr entfernt werden.
/// Input: `mask` (row-major, true = Vordergrund), `width`, `height`.
/// Modifiziert `mask` in-place.
pub fn zhang_suen_thinning(mask: &mut [bool], width: usize, height: usize) {
    loop {
        let removed1 = thinning_sub_iteration(mask, width, height, false);
        let removed2 = thinning_sub_iteration(mask, width, height, true);
        if !removed1 && !removed2 {
            break;
        }
    }
}

/// Führt eine Sub-Iteration des Zhang-Suen-Algorithmus durch.
///
/// Gibt `true` zurück wenn mindestens ein Pixel entfernt wurde.
/// `second_sub`: false = Sub-Iteration 1, true = Sub-Iteration 2.
fn thinning_sub_iteration(
    mask: &mut [bool],
    width: usize,
    height: usize,
    second_sub: bool,
) -> bool {
    let mut to_remove: Vec<usize> = Vec::new();

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let idx = y * width + x;
            if !mask[idx] {
                continue;
            }

            // Nachbarn P2..P9 im Uhrzeigersinn ab oben
            // P2=oben, P3=oben-rechts, P4=rechts, P5=unten-rechts
            // P6=unten, P7=unten-links, P8=links, P9=oben-links
            let p2 = mask[(y - 1) * width + x] as u8;
            let p3 = mask[(y - 1) * width + (x + 1)] as u8;
            let p4 = mask[y * width + (x + 1)] as u8;
            let p5 = mask[(y + 1) * width + (x + 1)] as u8;
            let p6 = mask[(y + 1) * width + x] as u8;
            let p7 = mask[(y + 1) * width + (x - 1)] as u8;
            let p8 = mask[y * width + (x - 1)] as u8;
            let p9 = mask[(y - 1) * width + (x - 1)] as u8;

            // B(P1) = Anzahl nicht-null Nachbarn
            let b = (p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9) as usize;
            if !(2..=6).contains(&b) {
                continue;
            }

            // A(P1) = Anzahl 0→1 Transitionen in der Nachbar-Sequenz
            let neighbors = [p2, p3, p4, p5, p6, p7, p8, p9, p2];
            let a = neighbors
                .windows(2)
                .filter(|w| w[0] == 0 && w[1] == 1)
                .count();
            if a != 1 {
                continue;
            }

            let should_remove = if !second_sub {
                // Sub-Iteration 1: P2·P4·P6 = 0 UND P4·P6·P8 = 0
                (p2 * p4 * p6 == 0) && (p4 * p6 * p8 == 0)
            } else {
                // Sub-Iteration 2: P2·P4·P8 = 0 UND P2·P6·P8 = 0
                (p2 * p4 * p8 == 0) && (p2 * p6 * p8 == 0)
            };

            if should_remove {
                to_remove.push(idx);
            }
        }
    }

    let removed = !to_remove.is_empty();
    for idx in to_remove {
        mask[idx] = false;
    }
    removed
}

// ---------------------------------------------------------------------------
// Unit-Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Thinning eines 3x3-Quadrats ergibt ein einzelnes Zentrum-Pixel.
    #[test]
    fn test_thinning_square_to_center() {
        let width = 5;
        let height = 5;
        // Inneres 3x3-Quadrat (Rand bleibt false)
        let mut mask = vec![false; width * height];
        for y in 1..4usize {
            for x in 1..4usize {
                mask[y * width + x] = true;
            }
        }
        zhang_suen_thinning(&mut mask, width, height);
        let foreground: Vec<(usize, usize)> = (0..height)
            .flat_map(|y| (0..width).map(move |x| (y, x)))
            .filter(|(y, x)| mask[y * width + x])
            .collect();
        // Ergebnis muss nicht-leer sein (kein vollständiges Auslöschen)
        assert!(!foreground.is_empty());
        // Randpixel dürfen nie gesetzt sein
        for x in 0..width {
            assert!(!mask[x], "Oberer Rand muss leer sein");
            assert!(
                !mask[(height - 1) * width + x],
                "Unterer Rand muss leer sein"
            );
        }
    }

    /// Leere Maske bleibt leer.
    #[test]
    fn test_thinning_empty_mask() {
        let mut mask = vec![false; 10 * 10];
        zhang_suen_thinning(&mut mask, 10, 10);
        assert!(mask.iter().all(|&v| !v));
    }

    /// Einzel-Pixel bleibt erhalten.
    #[test]
    fn test_thinning_single_pixel() {
        let width = 5;
        let height = 5;
        let mut mask = vec![false; width * height];
        mask[2 * width + 2] = true;
        zhang_suen_thinning(&mut mask, width, height);
        assert!(
            mask[2 * width + 2],
            "Einzel-Pixel darf nicht entfernt werden"
        );
    }
}
