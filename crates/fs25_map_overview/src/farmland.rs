//! Farmland-Polygon-Extraktion via Moore-Neighbor-Boundary-Tracing.
//!
//! Extrahiert fuer jede Farmland-ID einen geordneten Umriss-Polygon
//! aus rohen GRLE-Daten. Die Pixel-Koordinaten werden im Raster-Raum
//! zurueckgegeben; die Umrechnung in Weltkoordinaten erfolgt in der aufrufenden Schicht.

use std::collections::{HashMap, VecDeque};

use anyhow::Result;

use crate::grle;

/// Clockwise-Neighbor-Offsets fuer Moore-Boundary-Tracing (W, NW, N, NE, E, SE, S, SW).
///
/// In Bildkoordinaten (Y nach unten) entspricht diese Reihenfolge dem Uhrzeigersinn.
const CLOCKWISE: [(i32, i32); 8] = [
    (-1, 0),  // 0: W
    (-1, -1), // 1: NW
    (0, -1),  // 2: N
    (1, -1),  // 3: NE
    (1, 0),   // 4: E
    (1, 1),   // 5: SE
    (0, 1),   // 6: S
    (-1, 1),  // 7: SW
];

/// Ein geordnetes Umriss-Polygon fuer ein einzelnes Farmland-Feld.
///
/// Die Vertices sind Pixel-Koordinaten im GRLE-Raster. Fuer Weltkoordinaten
/// muss der Aufrufer `(pixel_x * scale, pixel_y * scale)` berechnen.
pub struct FarmlandPolygon {
    /// Farmland-ID (1–255, 0 = kein Feld)
    pub id: u32,
    /// Geordnete Rand-Pixel als (x, y) in Pixel-Koordinaten
    pub vertices: Vec<(f32, f32)>,
}

/// Extrahiert Farmland-Polygone aus bereits decodierten Pixel-Daten.
///
/// Jeder Bytewert ist eine Farmland-ID (0 = kein Feld, 255 = Hintergrund/Restflaeche).
/// Diese Funktion ist formatunabhaengig und wird sowohl fuer GRLE- als auch PNG-Daten
/// verwendet.
pub fn extract_farmland_polygons_from_ids(
    pixels: &[u8],
    width: usize,
    height: usize,
) -> Vec<FarmlandPolygon> {
    // Ersten Vorkommen jeder ID in Scan-Reihenfolge (top-left) sammeln.
    // ID 0 = kein Feld, ID 255 = Hintergrund/Restflaeche (FS25 GRLE Default-Wert).
    let mut start_pixels: HashMap<u8, (i32, i32)> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let id = pixels[y * width + x];
            if id != 0 && id != 255 {
                start_pixels.entry(id).or_insert((x as i32, y as i32));
            }
        }
    }

    let mut polygons = Vec::with_capacity(start_pixels.len());

    for (&id, &start) in &start_pixels {
        let raw_contour = trace_moore_contour(pixels, width, height, id, start);

        // Aufeinander folgende Duplikate entfernen (entstehen bei sehr duennen Regionen)
        let vertices = dedup_consecutive(raw_contour);

        if vertices.len() >= 2 {
            polygons.push(FarmlandPolygon {
                id: id as u32,
                vertices,
            });
        }
    }

    log::debug!(
        "Farmland-Polygone extrahiert: {} Felder aus {}x{} Raster",
        polygons.len(),
        width,
        height
    );

    polygons
}

/// Extrahiert Farmland-Polygone aus GRLE-Rohdaten per Moore-Neighbor-Boundary-Tracing.
///
/// Gibt alle gefundenen Polygone sowie die Raster-Dimensionen (width, height)
/// zurueck. Die Koordinaten der Polygone liegen im GRLE-Pixel-Raum.
///
/// # Fehler
/// Schlaegt fehl wenn die GRLE-Daten ungueltiges Format haben.
pub fn extract_farmland_polygons(grle_data: &[u8]) -> Result<(Vec<FarmlandPolygon>, u32, u32)> {
    let decoded = grle::decode_grle(grle_data)?;
    let width = decoded.width;
    let height = decoded.height;
    let polygons = extract_farmland_polygons_merged(&decoded.pixels, width, height);
    Ok((polygons, width as u32, height as u32))
}

/// Fuehrt Connected-Component-Labeling auf einer binaeren Farmland-Maske durch.
///
/// Erstellt eine binaere Maske (pixel > 0 && pixel != 255) und weist jedem
/// 4-verbundenen Bereich eine neue synthetische ID (1..=254) zu.
/// Benachbarte Farmland-Regionen mit unterschiedlichen Original-IDs werden
/// so zu einer einzigen Region zusammengefasst (Merge).
///
/// Regionen die an ein gemeinsames Farmland-Pixel angrenzen, aber getrennte
/// Original-IDs haben (FS25-Splitting), erhalten dieselbe synthetische ID
/// und werden danach als ein einziges Polygon extrahiert.
///
/// Gibt einen neuen Pixel-Puffer mit synthetischen IDs zurueck; 0 = Hintergrund.
pub fn merge_adjacent_farmland_regions(pixels: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut labels = vec![0u8; pixels.len()];
    let mut component_count: u32 = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let pixel = pixels[idx];

            // Hintergrund (0) und reservierten Wert (255) ueberspringen
            if pixel == 0 || pixel == 255 || labels[idx] != 0 {
                continue;
            }

            // Neue zusammenhaengende Farmland-Komponente — BFS starten
            component_count += 1;
            // Saettige bei 254: alle weiteren Regionen bekommen dieselbe ID
            let label = component_count.min(254) as u8;

            let mut queue = VecDeque::new();
            queue.push_back((x, y));
            labels[idx] = label;

            while let Some((cx, cy)) = queue.pop_front() {
                // 4-verbundene Nachbarn (oben, unten, links, rechts)
                let neighbors = [
                    (cx.wrapping_sub(1), cy),
                    (cx + 1, cy),
                    (cx, cy.wrapping_sub(1)),
                    (cx, cy + 1),
                ];
                for (nx, ny) in neighbors {
                    if nx >= width || ny >= height {
                        continue;
                    }
                    let nidx = ny * width + nx;
                    let npixel = pixels[nidx];
                    if npixel != 0 && npixel != 255 && labels[nidx] == 0 {
                        labels[nidx] = label;
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }

    if component_count > 254 {
        log::warn!(
            "merge_adjacent_farmland_regions: {} Regionen gefunden — ab Komponente 255 wird ID 254 verwendet",
            component_count
        );
    }

    log::debug!(
        "Farmland-Merge: {} zusammenhaengende Regionen gefunden",
        component_count.min(254)
    );

    labels
}

/// Extrahiert Farmland-Polygone aus Pixel-Daten nach vorherigem Region-Merge.
///
/// Fuehrt zunaechst ein Connected-Component-Labeling durch (beachbarte
/// Farmland-Regionen mit unterschiedlichen IDs werden zusammengefasst),
/// dann den Moore-Boundary-Tracer fuer jede zusammengefasste Komponente.
///
/// Dies behebt das FS25-Splitting-Problem, bei dem ein "visuelles Feld"
/// auf mehrere Farmland-IDs aufgeteilt ist und `trace_all_fields` sonst
/// mehrere ueberlagerte Ringe erzeugen wuerde.
pub fn extract_farmland_polygons_merged(
    pixels: &[u8],
    width: usize,
    height: usize,
) -> Vec<FarmlandPolygon> {
    let merged = merge_adjacent_farmland_regions(pixels, width, height);
    extract_farmland_polygons_from_ids(&merged, width, height)
}

// ---------------------------------------------------------------------------
// Interne Hilfsfunktionen
// ---------------------------------------------------------------------------

/// Liest den Pixelwert an (x, y); gibt 0 zurueck wenn ausserhalb des Rasters.
fn get_pixel(pixels: &[u8], width: usize, height: usize, x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        0
    } else {
        pixels[y as usize * width + x as usize]
    }
}

/// Berechnet den Startindex fuer die Clockwise-Suche aus der Richtung current → prev.
///
/// Gibt den CLOCKWISE-Index zurueck, der der Richtung von `current` nach `prev` entspricht.
fn clockwise_start_index(current: (i32, i32), prev: (i32, i32)) -> usize {
    let dx = prev.0 - current.0;
    let dy = prev.1 - current.1;
    CLOCKWISE
        .iter()
        .position(|&(odx, ody)| odx == dx && ody == dy)
        .unwrap_or(0)
}

/// Moore-Neighbor-Boundary-Tracing mit Jacob's Stopping-Criterion (korrigiert).
///
/// Tracer den aeusseren Rand der Komponente mit ID `target_id` und gibt
/// einen geordneten Umriss als Pixel-Koordinaten zurueck.
///
/// `start` muss der top-leftmost Pixel der Komponente sein
/// (garantiert durch die Scan-Reihenfolge in `extract_farmland_polygons`).
///
/// ## Stopping-Criterion
/// Der Tracer beendet sich beim ersten Besuch des Startpixels, sofern der
/// Backtrack-Pixel (`b`) identisch mit dem initialen Backtrack (`initial_b`)
/// ist. Dies stellt sicher, dass genau **ein vollstaendiger Umlauf** erzeugt
/// wird — ohne doppelte Vertices (Bug: fruehere Implementierung erzeugte 2
/// Umlaeufe, was `point_in_polygon` fuer alle 254 Felder korrumpierte).
///
/// Fuer schmale 1px-Streifen greift ein Fallback (`b_at_first_return`),
/// der beim zweiten identischen Eintrittspunkt abbricht.
fn trace_moore_contour(
    pixels: &[u8],
    width: usize,
    height: usize,
    target_id: u8,
    start: (i32, i32),
) -> Vec<(f32, f32)> {
    // Initialer Backtrack: Pixel westlich vom Start (immer Hintergrund,
    // da Start der leftmost-Pixel in der topmost-Zeile der Komponente ist).
    let initial_b = (start.0 - 1, start.1);

    let mut contour = vec![(start.0 as f32, start.1 as f32)];
    let mut current = start;
    let mut b = initial_b;

    // Fallback fuer schmale 1px-Regionen: Dort trifft b==initial_b nie zu,
    // weil der Tracer den Start stets aus einer anderen Richtung erreicht.
    // Beim ersten Besuch des Starts wird b gespeichert; beim zweiten identischen
    // Besuch bricht der Tracer ab (entspricht 2 Umlaeufen, sicher terminierend).
    let mut b_at_first_return: Option<(i32, i32)> = None;

    // Sicherheitslimit: maximal 4 × Rasterpixel Schritte
    let max_steps = width * height * 4;

    for _ in 0..max_steps {
        // Naechsten Vordergrund-Pixel im Uhrzeigersinn ab Backtrack suchen
        let start_idx = clockwise_start_index(current, b);
        let mut found_next: Option<(i32, i32)> = None;
        let mut new_b = b;

        for i in 0..8_usize {
            let idx = (start_idx + i) % 8;
            let (dx, dy) = CLOCKWISE[idx];
            let nx = current.0 + dx;
            let ny = current.1 + dy;
            if get_pixel(pixels, width, height, nx, ny) == target_id {
                found_next = Some((nx, ny));
                break;
            } else {
                // Letzten Hintergrund-Nachbar merken
                new_b = (nx, ny);
            }
        }

        let next = match found_next {
            Some(p) => p,
            // Isolierter Einzelpixel – kein weiterer Nachbar
            None => break,
        };

        b = new_b;
        current = next;

        // Jacob's Stopping-Criterion (Haupt-Bedingung):
        // Stoppe beim ersten Besuch des Starts, wenn der Backtrack identisch
        // mit dem initialen Backtrack ist. Dies garantiert genau einen Umlauf
        // fuer normale 2D-Polygone.
        if current == start {
            if b == initial_b {
                // Einmal vollstaendig umlaufen → fertig
                break;
            }
            // Fallback: anderer Eintrittswinkel (z.B. bei 1px-breiten Streifen).
            // Beim ersten Auftreten speichern; beim zweiten identischen b abbrechen.
            match b_at_first_return {
                None => {
                    b_at_first_return = Some(b);
                }
                Some(stored) if b == stored => {
                    break;
                }
                _ => {}
            }
            // Start nicht doppelt in den Contour einfuegen
        } else {
            contour.push((current.0 as f32, current.1 as f32));
        }
    }

    contour
}

/// Entfernt aufeinander folgende doppelte Vertices aus dem Contour.
fn dedup_consecutive(mut vertices: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    vertices.dedup();
    vertices
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Erstellt ein einfaches Pixelraster fuer Tests.
    ///
    /// `grid` ist zeilenweise (y-major), Eintraege sind 0 (Hintergrund) oder
    /// eine Farmland-ID.
    fn make_pixels(grid: &[&[u8]]) -> (Vec<u8>, usize, usize) {
        let height = grid.len();
        let width = if height > 0 { grid[0].len() } else { 0 };
        let pixels: Vec<u8> = grid.iter().flat_map(|row| row.iter().copied()).collect();
        (pixels, width, height)
    }

    /// Regression: Ein Doppel-Umlauf darf nicht auftreten.
    ///
    /// Mit dem fehlerhaften `b_at_first_return`-Vergleich enthielt jeder
    /// Vertex genau 2× in der Contour-Liste. Nach dem Fix darf jeder Vertex
    /// nur einmal vorkommen.
    #[test]
    fn test_no_double_traversal_3x3_square() {
        // 5×5 Raster mit einem 3×3-Quadrat (ID=1) in der Mitte
        #[rustfmt::skip]
        let (pixels, w, h) = make_pixels(&[
            &[0, 0, 0, 0, 0],
            &[0, 1, 1, 1, 0],
            &[0, 1, 1, 1, 0],
            &[0, 1, 1, 1, 0],
            &[0, 0, 0, 0, 0],
        ]);

        let start = (1, 1); // top-left Pixel des Quadrats
        let raw = trace_moore_contour(&pixels, w, h, 1, start);
        let contour = dedup_consecutive(raw);

        // Das 3×3-Quadrat hat 8 Rand-Pixel; kein Vertex darf doppelt vorkommen
        assert!(
            contour.len() <= 8,
            "Erwartete max. 8 Vertices fuer 3x3-Quadrat, bekam {}",
            contour.len()
        );

        // Uniqueness-Pruefung: kein Punkt darf doppelt auftreten
        let mut seen = std::collections::HashSet::new();
        for &pt in &contour {
            let key = (pt.0 as i32, pt.1 as i32);
            assert!(
                seen.insert(key),
                "Doppelter Vertex {:?} – Doppel-Umlauf-Regression!",
                pt
            );
        }
    }

    /// Ein horizontaler Balken (3×1 Pixel) – prueft schmale Regionen.
    ///
    /// Bekannte Einschraenkung des Moore-Tracers: 1px-breite Regionen erzeugen
    /// unvermeidlich doppelte Vertices (Hin- und Rueckweg sind identisch).
    /// Der Fallback-Mechanismus muss sicherstellen, dass der Tracer schnell
    /// terminiert und nicht bis max_steps laeuft.
    #[test]
    fn test_horizontal_stripe_terminates_quickly() {
        #[rustfmt::skip]
        let (pixels, w, h) = make_pixels(&[
            &[0, 0, 0, 0, 0],
            &[0, 1, 1, 1, 0],
            &[0, 0, 0, 0, 0],
        ]);

        let start = (1, 1);
        let raw = trace_moore_contour(&pixels, w, h, 1, start);
        let contour = dedup_consecutive(raw);

        // Muss weit unter dem max_steps-Limit liegen (w*h*4 = 60).
        // Mit dem Fallback-Stopp nach 2 Umlaeufen: maximal ~2 * 3 Pixel = ~6 Vertices.
        assert!(!contour.is_empty(), "Contour darf nicht leer sein");
        assert!(
            contour.len() <= 10,
            "Streifen-Contour zu lang ({} Vertices) – Fallback-Stopp hat nicht gegriffen",
            contour.len()
        );
    }

    /// Ein einzelner Pixel – Sonderfall; darf nicht abstuerzen.
    #[test]
    fn test_single_pixel() {
        #[rustfmt::skip]
        let (pixels, w, h) = make_pixels(&[
            &[0, 0, 0],
            &[0, 1, 0],
            &[0, 0, 0],
        ]);

        let start = (1, 1);
        let raw = trace_moore_contour(&pixels, w, h, 1, start);
        let contour = dedup_consecutive(raw);

        // Einzelpixel: genau 1 Vertex
        assert_eq!(contour.len(), 1, "Einzelpixel sollte genau 1 Vertex haben");
    }

    /// `extract_farmland_polygons_from_ids` darf keine doppelten Vertices
    /// zurueckgeben (end-to-end Regressions-Test).
    #[test]
    fn test_extract_no_double_vertices_end_to_end() {
        #[rustfmt::skip]
        let (pixels, w, h) = make_pixels(&[
            &[0, 0, 0, 0, 0],
            &[0, 2, 2, 2, 0],
            &[0, 2, 2, 2, 0],
            &[0, 2, 2, 2, 0],
            &[0, 0, 0, 0, 0],
        ]);

        let polygons = extract_farmland_polygons_from_ids(&pixels, w, h);
        assert_eq!(polygons.len(), 1);

        let poly = &polygons[0];
        let mut seen = std::collections::HashSet::new();
        for &pt in &poly.vertices {
            let key = (pt.0 as i32, pt.1 as i32);
            assert!(
                seen.insert(key),
                "Doppelter Vertex {:?} in end-to-end Test",
                pt
            );
        }
    }

    /// Zwei getrennte Felder mit unterschiedlichen IDs nebeneinander
    /// werden nach dem Merge zu einer einzigen Region zusammengefasst.
    #[test]
    fn test_merge_adjacent_regions_merges_neighbors() {
        // ID 1 und ID 2 teilen eine gemeinsame Kante → werden zu einer Region
        #[rustfmt::skip]
        let (pixels, w, h) = make_pixels(&[
            &[0, 0, 0, 0, 0],
            &[0, 1, 1, 2, 0],
            &[0, 1, 1, 2, 0],
            &[0, 1, 1, 2, 0],
            &[0, 0, 0, 0, 0],
        ]);

        let merged = merge_adjacent_farmland_regions(&pixels, w, h);

        // Alle Farmland-Pixel muessen dieselbe synthetische ID haben (eine Region)
        let farmland_labels: Vec<u8> = merged.iter().copied().filter(|&l| l != 0).collect();
        let first = farmland_labels[0];
        assert!(
            farmland_labels.iter().all(|&l| l == first),
            "Benachbarte Felder wurden nicht zur selben Region gemergt"
        );
        assert_eq!(first, 1, "Erste Region sollte Label 1 haben");
    }

    /// Zwei isolierte Felder (nicht benachbart) bleiben getrennte Regionen.
    #[test]
    fn test_merge_isolated_regions_stay_separate() {
        // ID 1 oben links, ID 2 unten rechts — durch Hintergrund getrennt
        #[rustfmt::skip]
        let (pixels, w, h) = make_pixels(&[
            &[1, 1, 0, 0, 0],
            &[1, 1, 0, 0, 0],
            &[0, 0, 0, 0, 0],
            &[0, 0, 0, 2, 2],
            &[0, 0, 0, 2, 2],
        ]);

        let merged = merge_adjacent_farmland_regions(&pixels, w, h);

        // Labelsatz muss genau {1, 2} ergeben (zwei getrennte Regionen)
        let mut label_set: std::collections::HashSet<u8> =
            merged.iter().copied().filter(|&l| l != 0).collect();
        assert_eq!(
            label_set.len(),
            2,
            "Isolierte Felder sollen 2 getrennte Regionen bleiben, aber Labels: {:?}",
            label_set
        );
        // Reihenfolge invariant: kleinste Label ist 1
        label_set.remove(&1);
        label_set.remove(&2);
        assert!(label_set.is_empty(), "Unerwartete Labels im Ergebnis");
    }

    /// Merged-Extraktion fasst benachbarte IDs zu einem Polygon zusammen.
    #[test]
    fn test_extract_merged_produces_single_polygon_for_adjacent_ids() {
        // Zwei nebeneinander liegende Felder mit unterschiedlichen IDs
        #[rustfmt::skip]
        let (pixels, w, h) = make_pixels(&[
            &[0, 0, 0, 0, 0, 0],
            &[0, 1, 1, 2, 2, 0],
            &[0, 1, 1, 2, 2, 0],
            &[0, 1, 1, 2, 2, 0],
            &[0, 0, 0, 0, 0, 0],
        ]);

        let polygons = extract_farmland_polygons_merged(&pixels, w, h);

        // Ohne Merge wuerden 2 Polygone entstehen; mit Merge nur 1
        assert_eq!(
            polygons.len(),
            1,
            "Benachbarte IDs sollen ein einziges Polygon ergeben, bekam {}",
            polygons.len()
        );
    }
}
