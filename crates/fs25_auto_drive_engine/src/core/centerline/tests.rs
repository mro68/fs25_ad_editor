use super::helpers::chain_pixels;
use super::{
    compute_polygon_centerline, compute_segment_centerline, compute_voronoi_bfs,
    extract_boundary_centerline, extract_corridor_centerline,
};
use crate::core::farmland::FarmlandGrid;
use glam::Vec2;

/// BFS auf einem kleinen Grid mit 2 Feldern — Voronoi-Grenze muss existieren.
#[test]
fn test_voronoi_bfs_basic() {
    // 6×1 Grid: [1,1,0,0,2,2]
    let ids = vec![1u8, 1, 0, 0, 2, 2];
    let grid = FarmlandGrid::new(ids, 6, 1, 6.0);
    let voronoi = compute_voronoi_bfs(&grid);

    // Linke Haelfte muss ID 1 haben
    assert_eq!(voronoi.nearest_id[0], 1);
    assert_eq!(voronoi.nearest_id[1], 1);
    // Rechte Haelfte muss ID 2 haben
    assert_eq!(voronoi.nearest_id[4], 2);
    assert_eq!(voronoi.nearest_id[5], 2);
    // Void-Pixel erhalten eine der beiden IDs
    assert!(voronoi.nearest_id[2] == 1 || voronoi.nearest_id[2] == 2);
    assert!(voronoi.nearest_id[3] == 1 || voronoi.nearest_id[3] == 2);
}

/// Centerline zwischen zwei benachbarten vertikalen Feldern.
#[test]
fn test_corridor_centerline_vertical() {
    // 5×3 Grid: Spalte 0-1 = ID 1, Spalte 3-4 = ID 2, Mitte = Void
    let w = 5u32;
    let h = 3u32;
    let mut ids = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if x < 2 {
                ids[i] = 1;
            } else if x >= 3 {
                ids[i] = 2;
            }
        }
    }
    let grid = FarmlandGrid::new(ids, w, h, 5.0);
    let voronoi = compute_voronoi_bfs(&grid);
    let line = extract_corridor_centerline(&voronoi, &[1], &[2], &grid);
    // Mittellinie muss mindestens einen Punkt haben
    assert!(!line.is_empty());
}

/// chain_pixels mit einfacher 3-Pixel-Linie.
#[test]
fn test_chain_pixels_simple() {
    let pixels = vec![(0u32, 0u32), (2, 0), (1, 0)];
    let chained = chain_pixels(&pixels);
    assert_eq!(chained.len(), 3);
    // Erste pixel muss (0,0) sein (Startpunkt)
    assert_eq!(chained[0], (0, 0));
}

/// Leere Eingabe ergibt leere Ausgabe.
#[test]
fn test_chain_pixels_empty() {
    let chained = chain_pixels(&[]);
    assert!(chained.is_empty());
}

/// `extract_boundary_centerline` mit zwei parallelen vertikalen Linien.
///
/// Die Mittellinie muss zwischen den beiden Seiten liegen (x ≈ 0).
#[test]
fn test_extract_boundary_centerline_basic() {
    let w = 20u32;
    let h = 20u32;
    let map_size = 20.0_f32;
    // scale = map_size / w = 1.0, half = 10.0
    // pixel x=5  → world x = 5*1 - 10 = -5.0
    // pixel x=15 → world x = 15*1 - 10 =  5.0
    let ids = vec![0u8; (w * h) as usize];
    let grid = FarmlandGrid::new(ids, w, h, map_size);

    let side1 = vec![Vec2::new(-5.0, -8.0), Vec2::new(-5.0, 8.0)];
    let side2 = vec![Vec2::new(5.0, -8.0), Vec2::new(5.0, 8.0)];

    let result = extract_boundary_centerline(&[side1], &[side2], &grid);

    assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
    for pt in &result {
        assert!(
            pt.x.abs() < 3.0,
            "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
            pt.x
        );
    }
}

/// Polygon-basierte Centerline zwischen zwei parallelen Rechtecken.
#[test]
fn test_polygon_centerline_two_rects() {
    // Feld 1: Rechteck links (x = -20..-10, y = -30..30)
    let poly1 = vec![
        Vec2::new(-20.0, -30.0),
        Vec2::new(-10.0, -30.0),
        Vec2::new(-10.0, 30.0),
        Vec2::new(-20.0, 30.0),
    ];
    // Feld 2: Rechteck rechts (x = 10..20, y = -30..30)
    let poly2 = vec![
        Vec2::new(10.0, -30.0),
        Vec2::new(20.0, -30.0),
        Vec2::new(20.0, 30.0),
        Vec2::new(10.0, 30.0),
    ];

    let result = compute_polygon_centerline(&[poly1.as_slice()], &[poly2.as_slice()], 2.0);

    assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
    // Mittellinie muss zwischen x=-10 und x=10 liegen (nahe x=0)
    for pt in &result {
        assert!(
            pt.x.abs() < 5.0,
            "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
            pt.x
        );
    }
}

/// Polygon-Centerline mit 2 Feldern auf Seite 1, 1 Feld auf Seite 2.
#[test]
fn test_polygon_centerline_two_vs_one() {
    // Seite 1: Zwei Felder nebeneinander (links oben + links unten)
    let poly1a = vec![
        Vec2::new(-20.0, 0.0),
        Vec2::new(-10.0, 0.0),
        Vec2::new(-10.0, 30.0),
        Vec2::new(-20.0, 30.0),
    ];
    let poly1b = vec![
        Vec2::new(-20.0, -30.0),
        Vec2::new(-10.0, -30.0),
        Vec2::new(-10.0, 0.0),
        Vec2::new(-20.0, 0.0),
    ];
    // Seite 2: Ein Feld rechts
    let poly2 = vec![
        Vec2::new(10.0, -30.0),
        Vec2::new(20.0, -30.0),
        Vec2::new(20.0, 30.0),
        Vec2::new(10.0, 30.0),
    ];

    let result = compute_polygon_centerline(
        &[poly1a.as_slice(), poly1b.as_slice()],
        &[poly2.as_slice()],
        2.0,
    );

    assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
    for pt in &result {
        assert!(
            pt.x.abs() < 5.0,
            "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
            pt.x
        );
    }
}

/// Segment-basierte Centerline.
#[test]
fn test_segment_centerline() {
    let side1 = vec![Vec2::new(-5.0, -20.0), Vec2::new(-5.0, 20.0)];
    let side2 = vec![Vec2::new(5.0, -20.0), Vec2::new(5.0, 20.0)];

    let result = compute_segment_centerline(&[side1], &[side2], 2.0);

    assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
    for pt in &result {
        assert!(
            pt.x.abs() < 3.0,
            "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
            pt.x
        );
    }
}
