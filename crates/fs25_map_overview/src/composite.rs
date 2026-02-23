//! Endmontage: Farmland-Grenzen, POI-Marker und Legende.
//!
//! Kombiniert das Terrain-Bild mit optionalen Overlays
//! zu einer fertigen Overview-Map.

use anyhow::Result;
use image::{GrayImage, Rgb, RgbImage};
use std::collections::HashMap;

use crate::grle;
use crate::text;

/// Erkannter Point of Interest.
#[derive(Debug, Clone)]
pub struct Poi {
    /// Pixel-X-Koordinate
    pub x: u32,
    /// Pixel-Y-Koordinate
    pub y: u32,
    /// Anzeigename
    pub label: String,
}

/// Farmland-Daten für Overlay.
pub struct FarmlandData {
    /// Grenzen-Maske (true = Grenzpixel)
    pub boundaries: Vec<bool>,
    /// Farmland-ID pro Pixel (0 = kein Farmland)
    pub ids: GrayImage,
    /// Dimensionen
    pub width: u32,
    pub height: u32,
}

/// Optionen für die Overview-Generierung.
#[derive(Debug, Clone, PartialEq)]
pub struct OverviewOptions {
    /// Hillshade anwenden
    pub hillshade: bool,
    /// Farmland-Grenzen einzeichnen
    pub farmlands: bool,
    /// Farmland-ID-Nummern einzeichnen
    pub farmland_ids: bool,
    /// POI-Marker einzeichnen
    pub pois: bool,
    /// Legende einzeichnen
    pub legend: bool,
}

impl Default for OverviewOptions {
    fn default() -> Self {
        Self {
            hillshade: true,
            farmlands: true,
            farmland_ids: true,
            pois: true,
            legend: true,
        }
    }
}

/// Berechnet Farmland-Grenzen aus einer GRLE-InfoLayer-Datei.
///
/// Grenzen entstehen dort, wo benachbarte Pixel unterschiedliche
/// Farmland-IDs haben und mindestens eines davon > 0 ist.
pub fn extract_farmland_boundaries(grle_data: &[u8], target_size: u32) -> Result<FarmlandData> {
    let decoded = grle::decode_grle(grle_data)?;
    let ids = GrayImage::from_raw(decoded.width as u32, decoded.height as u32, decoded.pixels)
        .ok_or_else(|| anyhow::anyhow!("Fehler beim Erstellen des Farmland-Bildes"))?;

    // Auf Zielgröße skalieren (Nearest-Neighbor für IDs)
    let ids = if ids.width() != target_size || ids.height() != target_size {
        image::imageops::resize(
            &ids,
            target_size,
            target_size,
            image::imageops::FilterType::Nearest,
        )
    } else {
        ids
    };

    let w = ids.width() as usize;
    let h = ids.height() as usize;
    let raw = ids.as_raw();

    let mut boundaries = vec![false; w * h];

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let val = raw[idx];

            // Rechter Nachbar
            if x + 1 < w {
                let right = raw[idx + 1];
                if val != right && (val > 0 || right > 0) {
                    boundaries[idx] = true;
                }
            }
            // Unterer Nachbar
            if y + 1 < h {
                let below = raw[(y + 1) * w + x];
                if val != below && (val > 0 || below > 0) {
                    boundaries[idx] = true;
                }
            }
        }
    }

    Ok(FarmlandData {
        boundaries,
        ids,
        width: target_size,
        height: target_size,
    })
}

/// Zeichnet Farmland-Grenzen auf ein RGB-Bild.
///
/// Grenzen werden als halbtransparentes Gelb gezeichnet.
pub fn draw_farmland_boundaries(image: &mut RgbImage, farmlands: &FarmlandData) {
    let w = farmlands.width as usize;
    let boundary_color = Rgb([255, 220, 50]);

    for (i, &is_boundary) in farmlands.boundaries.iter().enumerate() {
        if is_boundary {
            let x = (i % w) as u32;
            let y = (i / w) as u32;
            if x < image.width() && y < image.height() {
                // Halbtransparentes Blending
                let pixel = image.get_pixel(x, y);
                let blended = Rgb([
                    blend_channel(pixel[0], boundary_color[0], 0.5),
                    blend_channel(pixel[1], boundary_color[1], 0.5),
                    blend_channel(pixel[2], boundary_color[2], 0.5),
                ]);
                image.put_pixel(x, y, blended);
            }
        }
    }
}

/// POI-Erkennungsregeln: (Keyword im Dateinamen, Anzeigename).
const POI_RULES: &[(&str, &str)] = &[
    ("gasStation", "Tankstelle"),
    ("livestockMarket", "Viehmarkt"),
    ("bakery", "Bäckerei"),
    ("farmersMarket", "Bauernmarkt"),
    ("groceryStore", "Lebensmittelladen"),
    ("grainMill", "Getreidemühle"),
    ("grainmill", "Getreidemühle"),
    ("grainBarge", "Getreideterminal"),
    ("grainElevator", "Getreidesilo"),
    ("getreidesilo", "Getreidesilo"),
    ("sawmill", "Sägewerk"),
    ("dairy", "Molkerei"),
    ("canned", "Konservenfabrik"),
    ("konservenfabrik", "Konservenfabrik"),
    ("sugarMill", "Zuckerfabrik"),
    ("brewery", "Brauerei"),
    ("cottonMill", "Baumwollspinnerei"),
    ("distillery", "Destillerie"),
    ("buyingStationManure", "Dunghändler"),
    ("buyingStationLiquidManure", "Güllehändler"),
    ("buyingStationSeeds", "Saathändler"),
    ("buyingStation", "Ankaufstation"),
    ("sellingStation", "Verkaufsstelle"),
    ("weighingStation", "Waage"),
    ("spinningMill", "Spinnerei"),
    ("carpenterShop", "Schreinerei"),
    ("bga", "BGA"),
    ("spinnery", "Spinnerei"),
    ("oilMill", "Ölmühle"),
    ("flourMill", "Mehlmühle"),
];

/// Extrahiert POIs aus einer placeables.xml.
pub fn extract_pois(xml_data: &[u8], map_size: u32) -> Vec<Poi> {
    let mut pois = Vec::new();

    let content = match std::str::from_utf8(xml_data) {
        Ok(s) => s,
        Err(_) => return pois,
    };

    let mut reader = quick_xml::Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e) | quick_xml::events::Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "placeable" {
                    let mut position = None;
                    let mut filename = String::new();

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "position" => position = Some(val),
                            "filename" => filename = val,
                            _ => {}
                        }
                    }

                    if let Some(pos_str) = position {
                        if let Some(poi) = parse_poi(&pos_str, &filename, map_size) {
                            pois.push(poi);
                        }
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    pois
}

/// Parst eine einzelne Placeable-Position und prüft POI-Regeln.
fn parse_poi(position: &str, filename: &str, map_size: u32) -> Option<Poi> {
    let parts: Vec<&str> = position.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let x: f32 = parts[0].parse().ok()?;
    let z: f32 = parts[2].parse().ok()?;

    let combined = filename.to_ascii_lowercase();

    let label = POI_RULES
        .iter()
        .find(|(keyword, _)| combined.contains(&keyword.to_ascii_lowercase()))
        .map(|(_, name)| name.to_string())?;

    let half = map_size as f32 / 2.0;
    let px =
        ((x + half) / map_size as f32 * map_size as f32).clamp(0.0, (map_size - 1) as f32) as u32;
    let py =
        ((z + half) / map_size as f32 * map_size as f32).clamp(0.0, (map_size - 1) as f32) as u32;

    Some(Poi {
        x: px,
        y: py,
        label,
    })
}

/// Zeichnet POI-Marker auf ein RGB-Bild.
pub fn draw_pois(image: &mut RgbImage, pois: &[Poi]) {
    let marker_color = Rgb([220, 50, 50]);
    let outline_color = Rgb([255, 255, 255]);
    let radius = (image.width() / 600).max(6) as i32;

    for poi in pois {
        let cx = poi.x as i32;
        let cy = poi.y as i32;

        // Kreis zeichnen (gefüllt + Outline)
        draw_filled_circle(image, cx, cy, radius + 1, outline_color);
        draw_filled_circle(image, cx, cy, radius, marker_color);
    }
}

/// Zeichnet einen gefüllten Kreis auf ein RGB-Bild.
fn draw_filled_circle(image: &mut RgbImage, cx: i32, cy: i32, radius: i32, color: Rgb<u8>) {
    let w = image.width() as i32;
    let h = image.height() as i32;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && x < w && y >= 0 && y < h {
                    image.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

/// Blendet zwei Farbkanäle zusammen.
fn blend_channel(base: u8, overlay: u8, alpha: f32) -> u8 {
    let result = base as f32 * (1.0 - alpha) + overlay as f32 * alpha;
    result.clamp(0.0, 255.0) as u8
}

// ── Farmland-ID-Labels ──────────────────────────────────────────────

/// Zeichnet Farmland-ID-Nummern in die Mitte jedes Farmlands.
///
/// Berechnet den Schwerpunkt aller Pixel pro Farmland-ID
/// und zeichnet die ID-Nummer dort hin.
/// Kleine Farmlands (< 50 Pixel) werden übersprungen.
pub fn draw_farmland_ids(image: &mut RgbImage, farmlands: &FarmlandData) {
    let w = farmlands.width as usize;
    let raw = farmlands.ids.as_raw();

    // Schwerpunkt pro Farmland-ID berechnen
    let mut centroids: HashMap<u8, (u64, u64, u64)> = HashMap::new(); // id → (sum_x, sum_y, count)
    for (i, &id) in raw.iter().enumerate() {
        if id == 0 {
            continue;
        }
        let x = (i % w) as u64;
        let y = (i / w) as u64;
        let entry = centroids.entry(id).or_insert((0, 0, 0));
        entry.0 += x;
        entry.1 += y;
        entry.2 += 1;
    }

    // Font-Scale basierend auf Bildgröße
    let scale = (image.width() / 1200).clamp(1, 4);

    let label_color = Rgb([255, 255, 200]);

    for (id, (sum_x, sum_y, count)) in &centroids {
        if *count < 50 {
            continue; // Zu klein für ein Label
        }
        let cx = (*sum_x / *count) as i32;
        let cy = (*sum_y / *count) as i32;

        let label = id.to_string();
        let tw = text::text_width(&label, scale) as i32;
        let th = text::text_height(scale) as i32;

        text::draw_text_outlined(image, cx - tw / 2, cy - th / 2, &label, label_color, scale);
    }
}

// ── POI-Labels ──────────────────────────────────────────────────────

/// Zeichnet POI-Marker MIT Text-Labels auf ein RGB-Bild.
///
/// Labels werden rechts neben dem Marker platziert.
/// Überlappende Labels werden übersprungen (Mindestabstand).
pub fn draw_pois_with_labels(image: &mut RgbImage, pois: &[Poi]) {
    let marker_color = Rgb([220, 50, 50]);
    let outline_color = Rgb([255, 255, 255]);
    let label_color = Rgb([255, 255, 255]);
    let radius = (image.width() / 600).max(6) as i32;
    let scale = (image.width() / 1200).clamp(1, 4);
    let min_dist = (image.width() / 22) as i32;

    // Labels mit Überlappungsprüfung platzieren
    let mut placed_labels: Vec<(i32, i32)> = Vec::new();

    for poi in pois {
        let cx = poi.x as i32;
        let cy = poi.y as i32;

        // Marker immer zeichnen
        draw_filled_circle(image, cx, cy, radius + 1, outline_color);
        draw_filled_circle(image, cx, cy, radius, marker_color);

        // Label nur wenn genug Abstand zu bereits platzierten
        let too_close = placed_labels
            .iter()
            .any(|&(lx, ly)| (cx - lx).abs() < min_dist && (cy - ly).abs() < 40);

        if !too_close {
            let tx = cx + radius + 4;
            let ty = cy - text::text_height(scale) as i32 / 2;
            text::draw_text_outlined(image, tx, ty, &poi.label, label_color, scale);
            placed_labels.push((cx, cy));
        }
    }
}

// ── Legende ─────────────────────────────────────────────────────────

/// Legende-Einträge: (Farbe, Beschreibung).
const LEGEND_ITEMS: &[([u8; 3], &str)] = &[
    ([100, 100, 105], "Asphalt / Straße"),
    ([165, 165, 160], "Beton / Zement"),
    ([90, 150, 60], "Grasland"),
    ([50, 110, 45], "Wald"),
    ([140, 110, 70], "Erde / Boden"),
    ([160, 150, 130], "Kies"),
    ([95, 75, 50], "Schlamm"),
    ([195, 180, 140], "Sand"),
    ([70, 95, 120], "Wasser"),
    ([150, 145, 140], "Gehweg / Pflaster"),
];

/// Zeichnet die Farbschlüssel-Legende auf das Bild.
///
/// Die Legende wird unten links als halbtransparente Box dargestellt.
/// Enthält Terrain-Farben, POI-Markierung und Farmland-Grenzfarbe.
pub fn draw_legend(image: &mut RgbImage, options: &OverviewOptions) {
    let scale = (image.width() / 1200).clamp(1, 4);
    let padding = 15i32 * scale as i32 / 2;
    let row_h = 20i32 * scale as i32 / 2 + padding;
    let swatch_size = 14u32 * scale / 2;

    // Anzahl Zeilen berechnen
    let mut rows = LEGEND_ITEMS.len();
    if options.pois {
        rows += 1; // POI-Marker
    }
    if options.farmlands {
        rows += 1; // Farmland-Grenze
    }
    rows += 1; // Überschrift "Legende"

    let legend_w = (200 * scale / 2).max(180);
    let legend_h = (rows as i32 * row_h + padding * 2) as u32;

    let lx = 20i32;
    let ly = image.height() as i32 - legend_h as i32 - 20;

    // Halbtransparenter Hintergrund
    text::draw_rect_blended(image, lx, ly, legend_w, legend_h, Rgb([30, 30, 30]), 0.78);

    // Überschrift
    let mut yo = ly + padding;
    text::draw_text(
        image,
        lx + padding,
        yo,
        "Legende",
        Rgb([255, 255, 255]),
        scale,
    );
    yo += row_h + padding / 2;

    // Terrain-Farben
    for &(color, label) in LEGEND_ITEMS {
        text::draw_rect_filled(
            image,
            lx + padding,
            yo,
            swatch_size,
            swatch_size,
            Rgb(color),
        );
        text::draw_text(
            image,
            lx + padding + swatch_size as i32 + 8,
            yo + 2,
            label,
            Rgb([255, 255, 255]),
            scale.max(1), // mind. 1
        );
        yo += row_h;
    }

    // POI-Marker
    if options.pois {
        let r = (swatch_size / 2) as i32;
        let center_x = lx + padding + r;
        let center_y = yo + r;
        draw_filled_circle(image, center_x, center_y, r, Rgb([220, 50, 50]));
        text::draw_text(
            image,
            lx + padding + swatch_size as i32 + 8,
            yo + 2,
            "Gebäude / POI",
            Rgb([255, 255, 255]),
            scale,
        );
        yo += row_h;
    }

    // Farmland-Grenze
    if options.farmlands {
        text::draw_rect_filled(
            image,
            lx + padding,
            yo + swatch_size as i32 / 2 - 1,
            swatch_size,
            3,
            Rgb([255, 220, 50]),
        );
        text::draw_text(
            image,
            lx + padding + swatch_size as i32 + 8,
            yo + 2,
            "Farmland-Grenze",
            Rgb([255, 255, 255]),
            scale,
        );
    }
}

// ── Titel-Bar ───────────────────────────────────────────────────────

/// Zeichnet eine Titel-Bar am oberen Bildrand.
///
/// Halbtransparenter Hintergrund mit dem Kartennamen.
pub fn draw_title_bar(image: &mut RgbImage, title: &str) {
    let scale = (image.width() / 800).clamp(2, 6);
    let bar_h = text::text_height(scale) + scale * 6;
    let bar_w = image.width();

    // Halbtransparenter Hintergrund
    text::draw_rect_blended(image, 0, 0, bar_w, bar_h, Rgb([30, 30, 30]), 0.7);

    let label = format!("{} - Overview", title);
    let tx = (scale * 4) as i32;
    let ty = (scale * 3) as i32;
    text::draw_text(image, tx, ty, &label, Rgb([255, 255, 255]), scale);
}
