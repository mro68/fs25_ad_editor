//! Terrain-Farbpalette für FS25 Weight-Maps.
//!
//! Ordnet Weight-Map-Dateinamen RGB-Farben zu.
//! Basierend auf tatsächlichen FS25-Terrain-Texturen.

/// RGB-Farbwert
pub type Rgb = [u8; 3];

/// Gibt die Terrain-Farbe für einen Weight-Map-Dateinamen zurück.
///
/// Sucht zuerst exakt, dann Prefix-Match, dann Keyword-Fallback.
/// Gibt bei keinem Match ein neutrales Grau zurück.
pub fn terrain_color(weight_map_name: &str) -> Rgb {
    let stem = weight_map_name
        .strip_suffix("_weight.png")
        .unwrap_or(weight_map_name);
    // Trailing-Ziffern entfernen (z.B. "grass2" → "grass")
    let stem = stem.trim_end_matches(|c: char| c.is_ascii_digit());

    // 1. Exakter Match
    if let Some(color) = exact_match(stem) {
        return color;
    }

    // 2. Längster Prefix-Match
    if let Some(color) = prefix_match(stem) {
        return color;
    }

    // 3. Keyword-Fallback (case-insensitive)
    if let Some(color) = keyword_match(stem) {
        return color;
    }

    // 4. Neutrales Grau
    [130, 130, 128]
}

fn exact_match(stem: &str) -> Option<Rgb> {
    // Sortiert nach Häufigkeit / Wichtigkeit
    Some(match stem {
        // Straßen
        "asphalt" => [100, 100, 105],
        "asphaltCracks" => [95, 95, 100],
        "asphaltCracksDusty" => [105, 100, 95],
        "asphaltDirt" => [110, 105, 95],
        "asphaltDusty" => [115, 110, 105],
        "asphaltGravel" => [120, 115, 110],
        "tarmac" => [90, 90, 95],
        "tarmacDirt" => [105, 100, 90],
        "sideRoadTarmac" => [95, 95, 98],

        // Beton / Zement
        "cement" => [165, 165, 160],
        "cementCracked" => [160, 160, 155],
        "cementCrackedLighter" => [175, 175, 170],
        "cementDiscoloration" => [155, 155, 150],
        "cementDiscolorationLighter" => [170, 170, 165],
        "cementPlates" => [160, 160, 158],
        "cementPlatesLighter" => [175, 175, 172],
        "concreteIndustrial" => [150, 150, 148],
        "concreteSmallTiles" => [162, 162, 160],
        "concreteTiles" => [158, 158, 155],
        "hConcreteTiles" => [155, 155, 152],

        // Gras
        "grass" => [90, 150, 60],
        "grassDry" => [140, 155, 70],
        "grassPavement" => [110, 140, 80],
        "grassDirt" => [110, 135, 65],

        // Wald
        "forestGrass" => [50, 110, 45],
        "forestLeaves" => [45, 95, 35],
        "forestMossGround" => [40, 100, 40],
        "forestMossGroundLeaves" => [42, 95, 38],
        "forestNeedels" | "forestNeedles" => [35, 85, 30],
        "forestRoots" => [55, 90, 40],

        // Erde
        "dirt" => [140, 110, 70],
        "dirtDark" => [110, 85, 55],
        "dirtMedium" => [125, 100, 65],

        // Kies
        "gravel" => [160, 150, 130],
        "gravelDirt" => [145, 130, 100],

        // Schlamm
        "mud" => [95, 75, 50],

        // Fels
        "rock" => [140, 140, 135],
        "rockDark" => [120, 120, 115],
        "rockLight" => [170, 170, 165],

        // Sand
        "sand" => [195, 180, 140],
        "sandDark" => [170, 155, 115],

        // Wasser
        "waterGround" => [70, 95, 120],
        "waterGroundRocky" => [80, 100, 115],
        "water" => [55, 85, 130],

        // Gehweg / Pflaster
        "pavement" => [150, 145, 140],
        "pavementBrick" | "pavementBrickOld" => [155, 130, 110],
        "pavementBrickRed" => [160, 110, 90],
        "pavementBrickRedNew" => [170, 115, 95],
        "sidewalkTiles" => [170, 168, 165],
        "sidewalk" => [168, 166, 162],

        // Sonstiges
        "field" => [160, 140, 100],
        "snow" => [230, 235, 240],
        "ice" => [200, 215, 230],
        "grainStorage" => [170, 155, 120],

        _ => return None,
    })
}

fn prefix_match(stem: &str) -> Option<Rgb> {
    // Längste Prefixes zuerst prüfen
    static PREFIXES: &[(&str, Rgb)] = &[
        ("forestMossGroundLeaves", [42, 95, 38]),
        ("forestMossGround", [40, 100, 40]),
        ("forestLeaves", [45, 95, 35]),
        ("forestNeedle", [35, 85, 30]),
        ("forestGrass", [50, 110, 45]),
        ("forestRoots", [55, 90, 40]),
        ("asphaltCracks", [95, 95, 100]),
        ("asphaltGravel", [120, 115, 110]),
        ("asphaltDirt", [110, 105, 95]),
        ("asphalt", [100, 100, 105]),
        ("cement", [165, 165, 160]),
        ("concrete", [155, 155, 150]),
        ("pavement", [150, 145, 140]),
        ("sidewalk", [168, 166, 162]),
        ("grass", [90, 150, 60]),
        ("gravel", [160, 150, 130]),
        ("dirt", [140, 110, 70]),
        ("sand", [195, 180, 140]),
        ("rock", [140, 140, 135]),
        ("water", [65, 90, 125]),
        ("tarmac", [90, 90, 95]),
    ];

    for (prefix, color) in PREFIXES {
        if stem.starts_with(prefix) {
            return Some(*color);
        }
    }
    None
}

fn keyword_match(stem: &str) -> Option<Rgb> {
    let lower = stem.to_ascii_lowercase();

    static KEYWORDS: &[(&str, Rgb)] = &[
        ("asphalt", [100, 100, 105]),
        ("tarmac", [90, 90, 95]),
        ("concrete", [155, 155, 150]),
        ("cement", [162, 162, 158]),
        ("sidewalk", [168, 166, 162]),
        ("pavement", [150, 145, 140]),
        ("forest", [45, 100, 40]),
        ("grass", [90, 150, 60]),
        ("gravel", [160, 150, 130]),
        ("dirt", [140, 110, 70]),
        ("mud", [95, 75, 50]),
        ("sand", [195, 180, 140]),
        ("rock", [140, 140, 135]),
        ("water", [65, 90, 125]),
        ("snow", [230, 235, 240]),
        ("field", [160, 140, 100]),
    ];

    for (keyword, color) in KEYWORDS {
        if lower.contains(keyword) {
            return Some(*color);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert_eq!(terrain_color("grass_weight.png"), [90, 150, 60]);
        assert_eq!(terrain_color("asphalt"), [100, 100, 105]);
    }

    #[test]
    fn test_strips_trailing_digits() {
        // "grass2_weight.png" → stem "grass" → match
        assert_eq!(terrain_color("grass2_weight.png"), [90, 150, 60]);
    }

    #[test]
    fn test_prefix_match() {
        // "asphaltNewType" → prefix "asphalt"
        assert_eq!(terrain_color("asphaltNewType"), [100, 100, 105]);
    }

    #[test]
    fn test_keyword_fallback() {
        assert_eq!(terrain_color("myCustomForestThing"), [45, 100, 40]);
    }

    #[test]
    fn test_unknown_returns_grey() {
        assert_eq!(terrain_color("xyzUnknown"), [130, 130, 128]);
    }
}
