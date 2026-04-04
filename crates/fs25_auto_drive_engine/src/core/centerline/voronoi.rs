use crate::core::farmland::FarmlandGrid;
use std::collections::VecDeque;

/// Ergebnis der Voronoi-BFS: Fuer jeden Void-Pixel die naechste Farmland-ID und Distanz.
pub struct VoronoiGrid {
    /// Naechste Farmland-ID pro Pixel (0 = nicht initialisiert)
    pub nearest_id: Vec<u8>,
    /// Distanz zum naechsten Farmland-Pixel (skaliert ×10; gerade=10, diagonal=14)
    pub distance: Vec<u16>,
    /// Rasterbreite in Pixeln
    pub width: u32,
    /// Rasterhoehe in Pixeln
    pub height: u32,
}

/// Berechnet Multi-Source BFS auf dem Farmland-Grid.
///
/// Alle Farmland-Pixel (`grid.ids[i] > 0`) sind Seeds mit Distanz 0.
/// Jeder Void-Pixel erhaelt die ID und Distanz des naechsten Farmland-Pixels.
/// 8-Konnektivitaet (diagonal ≈ 14, gerade = 10 fuer ganzzahlige Approximation).
pub fn compute_voronoi_bfs(grid: &FarmlandGrid) -> VoronoiGrid {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let n = w * h;

    let mut nearest_id = vec![0u8; n];
    let mut distance = vec![u16::MAX; n];
    let mut queue: VecDeque<usize> = VecDeque::new();

    // Alle Farmland-Pixel als Multi-Source-Seeds initialisieren
    for (i, &id) in grid.ids.iter().enumerate() {
        if id > 0 {
            nearest_id[i] = id;
            distance[i] = 0;
            queue.push_back(i);
        }
    }

    // BFS mit 8 Nachbarn
    let offsets: [(i32, i32, u16); 8] = [
        (-1, 0, 10),
        (1, 0, 10),
        (0, -1, 10),
        (0, 1, 10),
        (-1, -1, 14),
        (1, -1, 14),
        (-1, 1, 14),
        (1, 1, 14),
    ];

    while let Some(idx) = queue.pop_front() {
        let cx = (idx % w) as i32;
        let cy = (idx / w) as i32;
        let cur_dist = distance[idx];
        let cur_id = nearest_id[idx];

        for (dx, dy, step) in offsets {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let nidx = (ny as usize) * w + (nx as usize);
            let new_dist = cur_dist.saturating_add(step);
            if new_dist < distance[nidx] {
                distance[nidx] = new_dist;
                nearest_id[nidx] = cur_id;
                queue.push_back(nidx);
            }
        }
    }

    VoronoiGrid {
        nearest_id,
        distance,
        width: grid.width,
        height: grid.height,
    }
}
