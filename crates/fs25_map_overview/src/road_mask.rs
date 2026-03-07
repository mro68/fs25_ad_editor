//! Straßenmasken-Extraktion aus Weight-Maps.

use image::{imageops, DynamicImage, GrayImage};

/// Dateinamen-Stems von Straßen-relevanten Weight-Map-Layern.
const ROAD_STEMS: &[&str] = &[
    "tarmac",
    "asphalt",
    "sideRoadTarmac",
    "asphaltDirt",
    "asphaltGravel",
    "asphaltCracks",
    "asphaltCracksDusty",
    "asphaltDusty",
    "sidewalkTiles",
    "concreteIndustrial",
];

/// Prueft ob ein Weight-Map-Dateiname zu einem Straßen-Layer gehoert.
///
/// Entfernt Dateiendung und numerische Suffixe vor dem Vergleich.
fn is_road_weight_map(name: &str) -> bool {
    // Dateiendungen entfernen
    let without_ext = name
        .strip_suffix("_weight.png")
        .or_else(|| name.rsplit_once('.').map(|(s, _)| s))
        .unwrap_or(name);
    // Numerische Suffixe entfernen (z.B. "asphalt2" → "asphalt")
    let stem = without_ext.trim_end_matches(|c: char| c.is_ascii_digit());
    ROAD_STEMS.iter().any(|&s| s.eq_ignore_ascii_case(stem))
}

/// Extrahiert eine kombinierte Straßenmaske aus Weight-Map-Bildern.
///
/// Filtert die uebergebenen Weight-Map-Layer nach Straßen-Pattern und
/// kombiniert alle passenden Bilder pixelweise per `max()` zu einer
/// einzigen Grayscale-Maske. Pixel mit Wert > 0 markieren Straßenflaechen.
///
/// # Parameter
/// - `images`: Liste der geladenen Weight-Maps als `(Dateiname, DynamicImage)`
/// - `target_size`: Zielgroesse der Maske (quadratisch) in Pixeln
///
/// # Rueckgabe
/// `Some(GrayImage)` wenn mindestens ein Straßen-Layer gefunden wurde, sonst `None`.
pub fn extract_road_mask(images: &[(String, DynamicImage)], target_size: u32) -> Option<GrayImage> {
    let road_images: Vec<GrayImage> = images
        .iter()
        .filter(|(name, _)| is_road_weight_map(name))
        .map(|(_, img)| {
            let gray = img.to_luma8();
            if gray.width() != target_size || gray.height() != target_size {
                imageops::resize(
                    &gray,
                    target_size,
                    target_size,
                    imageops::FilterType::Triangle,
                )
            } else {
                gray
            }
        })
        .collect();

    if road_images.is_empty() {
        log::info!("Keine Road-Weight-Maps gefunden – Straßenmaske nicht verfuegbar");
        return None;
    }

    log::info!(
        "{} Road-Weight-Map(s) zur Masken-Extraktion gefunden",
        road_images.len()
    );

    // Alle Straßen-Layer per max() kombinieren (vermeidet Overflow bei Ueberlappungen)
    let mut mask = GrayImage::new(target_size, target_size);
    for img in &road_images {
        for (x, y, pixel) in img.enumerate_pixels() {
            let current = mask.get_pixel(x, y)[0];
            mask.put_pixel(x, y, image::Luma([current.max(pixel[0])]));
        }
    }

    Some(mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_road_weight_map_bekannte_stems() {
        // Exakte Stems
        assert!(is_road_weight_map("tarmac_weight.png"));
        assert!(is_road_weight_map("asphalt_weight.png"));
        assert!(is_road_weight_map("sideRoadTarmac_weight.png"));
        assert!(is_road_weight_map("sidewalkTiles_weight.png"));
        assert!(is_road_weight_map("concreteIndustrial_weight.png"));
    }

    #[test]
    fn test_is_road_weight_map_mit_zahlen_suffix() {
        // Numerische Suffixe werden ignoriert
        assert!(is_road_weight_map("asphalt2_weight.png"));
        assert!(is_road_weight_map("tarmac1_weight.png"));
    }

    #[test]
    fn test_is_road_weight_map_keine_strasse() {
        // Nicht-Straßen-Texturen
        assert!(!is_road_weight_map("grass_weight.png"));
        assert!(!is_road_weight_map("dirt_weight.png"));
        assert!(!is_road_weight_map("fieldGrass_weight.png"));
    }

    #[test]
    fn test_extract_road_mask_keine_bilder() {
        let images = vec![(
            "grass_weight.png".to_string(),
            DynamicImage::new_luma8(64, 64),
        )];
        assert!(extract_road_mask(&images, 64).is_none());
    }

    #[test]
    fn test_extract_road_mask_kombiniert_per_max() {
        // Zwei Asphalt-Layer mit unterschiedlichen Werten → max() erwartet
        let mut img1 = GrayImage::new(4, 4);
        img1.put_pixel(0, 0, image::Luma([100]));
        img1.put_pixel(1, 0, image::Luma([50]));

        let mut img2 = GrayImage::new(4, 4);
        img2.put_pixel(0, 0, image::Luma([80]));
        img2.put_pixel(1, 0, image::Luma([200]));

        let images = vec![
            (
                "tarmac_weight.png".to_string(),
                DynamicImage::ImageLuma8(img1),
            ),
            (
                "asphalt_weight.png".to_string(),
                DynamicImage::ImageLuma8(img2),
            ),
        ];

        let mask = extract_road_mask(&images, 4).expect("Maske erwartet");
        assert_eq!(mask.get_pixel(0, 0)[0], 100); // max(100, 80)
        assert_eq!(mask.get_pixel(1, 0)[0], 200); // max(50, 200)
    }
}
