//! Skelett-Extraktion fuer das ColorPathTool.
//!
//! Pipeline: Bool-Maske → Zhang-Suen-Thinning → Verbundene Komponenten →
//! Geordnete Polylines → Weltkoordinaten.

use glam::Vec2;
use std::collections::{HashMap, VecDeque};

use super::sampling::{morphological_close, morphological_open, pixel_to_world_f32};
use crate::core::zhang_suen_thinning;

/// Mindest-Pixelanzahl eines Pfades — kuerzere Fragmente werden verworfen.
const MIN_PATH_LENGTH: usize = 5;

// ---------------------------------------------------------------------------
// Verbundene Komponenten (Flood-Fill)
// ---------------------------------------------------------------------------

/// Findet alle zusammenhaengenden Skelett-Pixel-Gruppen (8-Connectivity).
///
/// Iteriert ueber alle `true`-Pixel der Maske und fuehrt pro Gruppe eine
/// BFS durch. Gibt die Gruppen sortiert nach Groesse zurueck (laengste zuerst).
pub(crate) fn find_connected_components(
    mask: &[bool],
    width: usize,
    height: usize,
) -> Vec<Vec<(usize, usize)>> {
    let mut visited = vec![false; mask.len()];
    let mut components = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if !mask[idx] || visited[idx] {
                continue;
            }

            // BFS fuer diese zusammenhaengende Gruppe
            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back((x, y));
            visited[idx] = true;

            while let Some((cx, cy)) = queue.pop_front() {
                component.push((cx, cy));
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = cx as i32 + dx;
                        let ny = cy as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            continue;
                        }
                        let nidx = ny as usize * width + nx as usize;
                        if mask[nidx] && !visited[nidx] {
                            visited[nidx] = true;
                            queue.push_back((nx as usize, ny as usize));
                        }
                    }
                }
            }
            components.push(component);
        }
    }

    // Laengste Gruppe zuerst
    components.sort_by_key(|c: &Vec<(usize, usize)>| std::cmp::Reverse(c.len()));
    components
}

// ---------------------------------------------------------------------------
// Skelett-Pfad ordnen (Durchmesser-BFS)
// ---------------------------------------------------------------------------

/// Ordnet eine Menge von Skelett-Pixeln in eine lineare Sequenz.
///
/// Algorithmus: Zweifache BFS (Durchmesser-Methode).
/// 1. BFS vom Startpunkt (Hint-Pixel oder beliebig) → findet Endpunkt A.
/// 2. BFS von A → findet Endpunkt B und rekonstruiert den laengsten Pfad A→B.
///
/// Ist `hint` angegeben, wird als erster Startpunkt der Pixel aus `pixels`
/// gewaehlt der dem Hint am naechsten liegt. Dadurch laeuft der Pfad von
/// der Lasso-Startseite aus, nicht vom geometrischen Durchmesser-Endpunkt.
///
/// Bei Verzweigungen wird automatisch der laengste Teilpfad gewaehlt,
/// da der Graphdurchmesser immer die zwei weitesten Endpunkte verbindet.
pub(crate) fn order_skeleton_pixels(
    pixels: &[(usize, usize)],
    hint: Option<(usize, usize)>,
) -> Vec<(usize, usize)> {
    if pixels.is_empty() {
        return Vec::new();
    }
    if pixels.len() == 1 {
        return vec![pixels[0]];
    }

    let pixel_set: std::collections::HashSet<(usize, usize)> = pixels.iter().copied().collect();

    // Startpunkt: Pixel am naechsten zum Hint (oder erstes Element als Fallback)
    let initial_start = if let Some((hx, hy)) = hint {
        pixels
            .iter()
            .copied()
            .min_by_key(|&(px, py)| {
                let dx = px as i64 - hx as i64;
                let dy = py as i64 - hy as i64;
                dx * dx + dy * dy
            })
            .unwrap_or(pixels[0])
    } else {
        pixels[0]
    };

    // Rueckgabetyp-Alias fuer die BFS-Hilfsclosure (farthest_node + parent_map)
    type BfsResult = (
        (usize, usize),
        HashMap<(usize, usize), Option<(usize, usize)>>,
    );

    // BFS von einem Startknoten: gibt (farthest_node, parent_map) zurueck.
    // Die parent_map erlaubt die Pfad-Rekonstruktion vom farthest_node
    // zurueck zum Startknoten.
    let bfs_from = |start: (usize, usize)| -> BfsResult {
        let mut queue = VecDeque::new();
        let mut parent: HashMap<(usize, usize), Option<(usize, usize)>> = HashMap::new();
        queue.push_back(start);
        parent.insert(start, None);
        let mut farthest = start;

        while let Some(current) = queue.pop_front() {
            farthest = current;
            let (cx, cy) = current;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx < 0 || ny < 0 {
                        continue;
                    }
                    let nbr = (nx as usize, ny as usize);
                    if pixel_set.contains(&nbr) && !parent.contains_key(&nbr) {
                        parent.insert(nbr, Some(current));
                        queue.push_back(nbr);
                    }
                }
            }
        }
        (farthest, parent)
    };

    // Schritt 1: BFS vom Startpunkt → Endpunkt A (einer der Durchmesser-Enden)
    let (far_a, _) = bfs_from(initial_start);

    // Schritt 2: BFS von A → Endpunkt B + Parent-Map fuer Pfad-Rekonstruktion
    let (far_b, parent_map) = bfs_from(far_a);

    // Pfad von B zurueck zu A rekonstruieren
    let mut path = Vec::new();
    let mut current = far_b;
    loop {
        path.push(current);
        match parent_map[&current] {
            Some(p) => current = p,
            None => break, // Startknoten A erreicht
        }
    }

    // Pfad laeuft B→A; umkehren fuer A→B
    path.reverse();
    path
}

// ---------------------------------------------------------------------------
// Medial-Axis-Korrektur
// ---------------------------------------------------------------------------

/// Sucht den Abstand zum naechsten Rand-Pixel in einer Richtung (nx, ny).
///
/// Schrittweise Abtastung entlang (nx, ny) ab (x, y). Gibt die Distanz (in
/// Pixeln − 0.5) zurueck, an der erstmals ein `false`-Pixel oder der
/// Bildrand erreicht wird.
fn find_boundary_distance(
    x: usize,
    y: usize,
    nx: f32,
    ny: f32,
    mask: &[bool],
    width: usize,
    height: usize,
) -> f32 {
    for step in 1..=30i32 {
        let ix = (x as f32 + nx * step as f32).round() as i32;
        let iy = (y as f32 + ny * step as f32).round() as i32;
        if ix < 0 || iy < 0 || ix >= width as i32 || iy >= height as i32 {
            return step as f32 - 0.5;
        }
        if !mask[iy as usize * width + ix as usize] {
            return step as f32 - 0.5;
        }
    }
    30.0
}

/// Korrigiert geordnete Skelett-Pixel auf die geometrische Mittelachse.
///
/// Fuer jeden Skelett-Pixel wird die lokale Tangente aus Vorgaenger und
/// Nachfolger berechnet. Senkrecht dazu wird auf beiden Seiten der naechste
/// Rand-Pixel in `original_mask` gesucht. Der korrigierte Punkt liegt auf
/// dem geometrischen Mittelpunkt zwischen beiden Raendern.
pub(crate) fn refine_medial_axis(
    ordered: &[(usize, usize)],
    original_mask: &[bool],
    width: usize,
    height: usize,
) -> Vec<(f32, f32)> {
    let n = ordered.len();
    ordered
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            let (prev_x, prev_y) = if i > 0 {
                ordered[i - 1]
            } else if i + 1 < n {
                ordered[i + 1]
            } else {
                (x, y)
            };
            let (next_x, next_y) = if i + 1 < n {
                ordered[i + 1]
            } else if i > 0 {
                ordered[i - 1]
            } else {
                (x, y)
            };

            let dx = next_x as f32 - prev_x as f32;
            let dy = next_y as f32 - prev_y as f32;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 0.001 {
                return (x as f32, y as f32);
            }

            // Normierte Tangente; Normale = 90°-Rotation
            let (tx, ty) = (dx / len, dy / len);
            let (nx_f, ny_f) = (-ty, tx);

            // Abstand zum Rand in beiden Normalenrichtungen
            let d_pos = find_boundary_distance(x, y, nx_f, ny_f, original_mask, width, height);
            let d_neg = find_boundary_distance(x, y, -nx_f, -ny_f, original_mask, width, height);

            // Mittelachsen-Offset: positiv = in Richtung +Normale
            let offset = (d_pos - d_neg) / 2.0;
            (x as f32 + nx_f * offset, y as f32 + ny_f * offset)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pixel → Weltkoordinaten
// ---------------------------------------------------------------------------

/// Konvertiert korrigierte Sub-Pixel-Positionen in Weltkoordinaten.
///
/// Wird nach `refine_medial_axis` verwendet, wo Pixel-Positionen nicht
/// ganzzahlig sein koennen.
fn refined_pixels_to_world(
    refined: &[(f32, f32)],
    map_size: f32,
    img_width: u32,
    img_height: u32,
) -> Vec<Vec2> {
    refined
        .iter()
        .map(|&(px, py)| pixel_to_world_f32(px, py, map_size, img_width, img_height))
        .collect()
}

// ---------------------------------------------------------------------------
// Haupt-Pipeline
// ---------------------------------------------------------------------------

/// Fuehrt die komplette Pipeline aus:
/// Bool-Maske → Zhang-Suen → Komponenten finden → Pfade ordnen → Medial-Axis → Weltkoords.
///
/// Gibt alle gefundenen Pfade sortiert nach Laenge (laengster zuerst) zurueck.
/// Fragmente mit weniger als 5 Pixeln werden verworfen.
///
/// - `noise_filter`: Wenn `true`, wird vor dem Thinning morphologisches
///   Opening (Erosion+Dilation) und Closing (Dilation+Erosion) angewendet
///   um Einzelpixel-Rauschen zu entfernen und kleine Luecken zu schliessen.
/// - `start_hint`: Optionaler Pixel-Punkt in der Naehe des Lasso-Startpunkts.
///   Steuert den Startpunkt der Skelett-Ordnung (vgl. `order_skeleton_pixels`).
pub(crate) fn extract_paths_from_mask(
    mask: &mut Vec<bool>,
    width: u32,
    height: u32,
    noise_filter: bool,
    map_size: f32,
    start_hint: Option<(usize, usize)>,
) -> Vec<Vec<Vec2>> {
    let w = width as usize;
    let h = height as usize;

    // Optional: Rauschfilter — Opening entfernt isolierte Pixel,
    // Closing schliesst kleine Luecken
    if noise_filter {
        let opened = morphological_open(mask, w, h);
        let closed = morphological_close(&opened, w, h);
        *mask = closed;
    }

    // Original-Maske vor Zhang-Suen sichern (fuer Medial-Axis-Korrektur)
    let original_mask = mask.clone();

    // Zhang-Suen: Maske auf 1-Pixel-breites Skelett reduzieren
    zhang_suen_thinning(mask, w, h);

    // Zusammenhaengende Skelett-Gruppen extrahieren
    let components = find_connected_components(mask, w, h);

    // Alle Komponenten ab MIN_PATH_LENGTH zusammenfuehren — sie stammen aus
    // demselben Flood-Fill-Bereich und wurden nur durch Thinning-Artefakte
    // oder morphologische Operationen getrennt.
    let merged_pixels: Vec<(usize, usize)> = components
        .iter()
        .filter(|comp| comp.len() >= MIN_PATH_LENGTH)
        .flat_map(|comp| comp.iter().copied())
        .collect();

    if merged_pixels.is_empty() {
        return Vec::new();
    }

    log::info!(
        "Skelett: {} Komponenten ({} Pixel total) zu einem Pfad zusammengefuehrt",
        components
            .iter()
            .filter(|c| c.len() >= MIN_PATH_LENGTH)
            .count(),
        merged_pixels.len()
    );

    let ordered = order_skeleton_pixels(&merged_pixels, start_hint);
    let refined = refine_medial_axis(&ordered, &original_mask, w, h);
    let path = refined_pixels_to_world(&refined, map_size, width, height);

    if path.is_empty() {
        Vec::new()
    } else {
        vec![path]
    }
}

// ---------------------------------------------------------------------------
// Unit-Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_mask(width: usize, height: usize) -> Vec<bool> {
        vec![false; width * height]
    }

    fn set_pixel(mask: &mut [bool], x: usize, y: usize, width: usize) {
        mask[y * width + x] = true;
    }

    /// Zwei getrennte L-foermige Gruppen werden als separate Komponenten erkannt.
    #[test]
    fn connected_components_zwei_getrennte_gruppen() {
        let width = 10usize;
        let height = 5usize;
        let mut mask = empty_mask(width, height);

        // Gruppe 1: L-Form bei (0,0)
        set_pixel(&mut mask, 0, 0, width);
        set_pixel(&mut mask, 0, 1, width);
        set_pixel(&mut mask, 0, 2, width);
        set_pixel(&mut mask, 1, 2, width);

        // Gruppe 2: L-Form bei (7,0) — weit genug entfernt fuer keine 8-Nachbarschaft
        set_pixel(&mut mask, 7, 0, width);
        set_pixel(&mut mask, 7, 1, width);
        set_pixel(&mut mask, 7, 2, width);
        set_pixel(&mut mask, 8, 2, width);

        let components = find_connected_components(&mask, width, height);
        assert_eq!(components.len(), 2, "Zwei Gruppen erwartet");
        assert_eq!(components[0].len(), 4, "Gruppe 1: 4 Pixel");
        assert_eq!(components[1].len(), 4, "Gruppe 2: 4 Pixel");
    }

    /// Leere Maske ergibt keine Komponenten.
    #[test]
    fn connected_components_leere_maske() {
        let mask = empty_mask(5, 5);
        let components = find_connected_components(&mask, 5, 5);
        assert!(components.is_empty(), "Keine Komponenten in leerer Maske");
    }

    /// Einzelner Pixel ergibt eine Komponente mit einem Element.
    #[test]
    fn connected_components_einzelner_pixel() {
        let mut mask = empty_mask(5, 5);
        set_pixel(&mut mask, 2, 2, 5);
        let components = find_connected_components(&mask, 5, 5);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 1);
    }

    /// Linearer 5-Pixel-Pfad wird korrekt geordnet (alle Pixel enthalten, richtige Endpunkte).
    #[test]
    fn order_linear_pfad_fuenf_pixel() {
        let pixels = vec![(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)];
        let ordered = order_skeleton_pixels(&pixels, None);
        assert_eq!(ordered.len(), 5, "Alle 5 Pixel muessen enthalten sein");

        // Endpunkte muessen (0,2) und (4,2) sein (Reihenfolge egal)
        let ends: std::collections::HashSet<(usize, usize)> =
            [ordered[0], ordered[4]].iter().copied().collect();
        assert!(
            ends.contains(&(0, 2)),
            "Endpunkt (0,2) muss im Ergebnis sein"
        );
        assert!(
            ends.contains(&(4, 2)),
            "Endpunkt (4,2) muss im Ergebnis sein"
        );
    }

    /// Bei einer Y-Form (Stamm + kurzer Ast) wird der laengste Teilpfad gewaehlt.
    ///
    /// Geometrie:
    /// - Vertikaler Stamm: (2,0) bis (2,5) — 6 Pixel
    /// - Kurzer Ast am Knoten (2,3): Pixel (3,3) — 1 Pixel
    ///
    /// Erwartung: Ergebnis = 6 Pixel (Stamm), Ast (3,3) nicht im Hauptpfad.
    #[test]
    fn order_verzweigung_laengster_pfad() {
        // Vertikaler Stamm: 6 Pixel
        let mut pixels = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (2, 5)];
        // Kurzer Ast — per 8-Connectivity mit (2,2), (2,3) und (2,4) verbunden
        pixels.push((3, 3));

        let ordered = order_skeleton_pixels(&pixels, None);
        assert_eq!(
            ordered.len(),
            6,
            "Nur der Stamm (6 Pixel) soll im Pfad sein; Ast (3,3) wird ausgeschlossen"
        );

        // Endpunkte muessen (2,0) und (2,5) sein
        let ends: std::collections::HashSet<(usize, usize)> =
            [ordered[0], ordered[5]].iter().copied().collect();
        assert!(
            ends.contains(&(2, 0)),
            "Stamm-Endpunkt (2,0) muss Pfad-Endpunkt sein"
        );
        assert!(
            ends.contains(&(2, 5)),
            "Stamm-Endpunkt (2,5) muss Pfad-Endpunkt sein"
        );
    }

    /// Eine 3-Pixel-breite horizontale Linie ergibt nach Thinning einen einzelnen Pfad.
    #[test]
    fn extract_paths_horizontale_linie_3px_breit() {
        let width = 12u32;
        let height = 7u32;
        let w = width as usize;

        // 3 Pixel breites Band: y=2,3,4; innere Pixel x=1..=10 (Rand bleibt false)
        let mut mask = vec![false; (width * height) as usize];
        for y in 2usize..=4 {
            for x in 1usize..=10 {
                mask[y * w + x] = true;
            }
        }

        let paths = extract_paths_from_mask(&mut mask, width, height, false, 1000.0, None);

        assert_eq!(paths.len(), 1, "Genau ein Pfad nach Thinning erwartet");
        assert!(
            paths[0].len() >= 5,
            "Pfad muss mindestens 5 Punkte haben, hat: {}",
            paths[0].len()
        );
    }
}
