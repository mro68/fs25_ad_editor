//! Benchmark fuer Route-Tool-Preview-Hotpaths.
//!
//! Misst die Kernberechnungen der Preview-Erzeugung in den aufwaendigeren Tools:
//! - Bypass: `compute_bypass_positions`
//! - RouteOffset: `compute_offset_positions`
//! - FieldPath: `compute_polygon_centerline` / `compute_segment_centerline`
//! - ColorPath: Flood-Fill + Netzextraktion
//! - SmoothCurve: `solve_route`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fs25_auto_drive_editor::app::tools::bypass::compute_bypass_positions;
use fs25_auto_drive_editor::app::tools::color_path::compute_color_path_network_stats;
use fs25_auto_drive_editor::app::tools::route_offset::compute_offset_positions;
use fs25_auto_drive_editor::app::tools::smooth_curve::{solve_route, SmoothCurveInput};
use fs25_auto_drive_editor::core::{compute_polygon_centerline, compute_segment_centerline};
use glam::Vec2;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use std::hint::black_box;

fn make_chain(count: usize, step: f32) -> Vec<Vec2> {
    (0..count)
        .map(|i| {
            let x = i as f32 * step;
            let y = (i as f32 * 0.3).sin() * 7.5;
            Vec2::new(x, y)
        })
        .collect()
}

fn make_corridor_boundary(count: usize, step: f32, y_base: f32, phase: f32) -> Vec<Vec2> {
    (0..count)
        .map(|i| {
            let x = i as f32 * step;
            let wave = (i as f32 * 0.18 + phase).sin() * 4.0;
            Vec2::new(x, y_base + wave)
        })
        .collect()
}

fn make_field_polygon(
    count: usize,
    step: f32,
    corridor_y: f32,
    outer_y: f32,
    phase: f32,
) -> Vec<Vec2> {
    let inner = make_corridor_boundary(count, step, corridor_y, phase);
    let outer = make_corridor_boundary(count, step, outer_y, phase * 0.7 + 0.5);
    let mut polygon = outer;
    polygon.extend(inner.into_iter().rev());
    polygon
}

fn make_color_path_image(size: u32) -> DynamicImage {
    let half = (size / 2) as i32;
    let thickness = (size / 18).max(6) as i32;
    let img: RgbImage = ImageBuffer::from_fn(size, size, |x, y| {
        let xi = x as i32;
        let yi = y as i32;
        let on_horizontal = (yi - half).abs() <= thickness && x > size / 8 && x < size * 7 / 8;
        let on_vertical = (xi - half).abs() <= thickness && y > size / 6 && y < size * 5 / 6;
        let on_diagonal = ((xi - yi) - half / 3).abs() <= thickness / 2 && x > size / 5;

        if on_horizontal || on_vertical || on_diagonal {
            let tint = ((x + y) % 7) as u8;
            Rgb([150 + tint, 120, 72u8.saturating_sub(tint / 2)])
        } else {
            let noise = ((x * 13 + y * 7) % 9) as u8;
            Rgb([58 + noise, 94 + noise, 56 + noise / 2])
        }
    });
    DynamicImage::ImageRgb8(img)
}

fn bench_bypass_preview(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_bypass");

    for &count in &[10usize, 50, 200] {
        let chain = make_chain(count, 6.0);

        group.bench_with_input(
            BenchmarkId::new("compute_positions", count),
            &chain,
            |b, pts| {
                b.iter(|| {
                    let result = compute_bypass_positions(black_box(pts), 8.0, 6.0);
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn bench_smooth_curve_preview(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_smooth_curve");

    for &controls in &[0usize, 3, 8] {
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(120.0, 30.0);
        let control_nodes: Vec<Vec2> = (0..controls)
            .map(|i| Vec2::new(15.0 + i as f32 * 12.0, (i as f32 * 0.8).sin() * 18.0))
            .collect();

        let input = SmoothCurveInput {
            start,
            end,
            control_nodes,
            max_segment_length_m: 6.0,
            max_direction_change_deg: 45.0,
            start_neighbor_directions: vec![Vec2::new(1.0, 0.0), Vec2::new(0.7, 0.3)],
            end_neighbor_directions: vec![Vec2::new(-1.0, 0.0), Vec2::new(-0.7, -0.3)],
            min_distance: 1.0,
        };

        group.bench_with_input(
            BenchmarkId::new("solve_route", controls),
            &input,
            |b, route_input| {
                b.iter(|| {
                    let result = solve_route(black_box(route_input));
                    black_box(result.positions.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_route_offset_preview(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_route_offset");

    for &count in &[10usize, 50, 200] {
        let chain = make_chain(count, 6.0);

        group.bench_with_input(
            BenchmarkId::new("compute_positions", count),
            &chain,
            |b, pts| {
                b.iter(|| {
                    let result = compute_offset_positions(black_box(pts), 8.0, 6.0);
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn bench_field_path_polygon_centerline(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_field_path_polygon");

    for &count in &[32usize, 128, 512] {
        let left = make_field_polygon(count, 4.0, -10.0, -70.0, 0.0);
        let right = make_field_polygon(count, 4.0, 10.0, 70.0, 0.8);
        let left_refs = [left.as_slice()];
        let right_refs = [right.as_slice()];

        group.bench_with_input(
            BenchmarkId::new("compute_centerline", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let result = compute_polygon_centerline(
                        black_box(&left_refs),
                        black_box(&right_refs),
                        2.0,
                    );
                    black_box(result.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_field_path_segment_centerline(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_field_path_segment");

    for &count in &[32usize, 128, 512] {
        let left = vec![make_corridor_boundary(count, 4.0, -10.0, 0.0)];
        let right = vec![make_corridor_boundary(count, 4.0, 10.0, 0.7)];

        group.bench_with_input(
            BenchmarkId::new("compute_centerline", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let result =
                        compute_segment_centerline(black_box(&left), black_box(&right), 2.0);
                    black_box(result.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_color_path_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_color_path");

    for &size in &[128u32, 256, 512] {
        let image = make_color_path_image(size);
        let palette = vec![[150, 120, 72], [154, 120, 70], [156, 120, 69]];
        let start_pixel = (size / 2, size / 2);

        group.bench_with_input(BenchmarkId::new("compute_network", size), &size, |b, _| {
            b.iter(|| {
                let stats = compute_color_path_network_stats(
                    black_box(&image),
                    black_box(&palette),
                    18.0,
                    start_pixel,
                    true,
                    size as f32,
                );
                black_box(stats)
            })
        });
    }

    group.finish();
}

criterion_group!(
    tool_preview_hotpath_benches,
    bench_bypass_preview,
    bench_route_offset_preview,
    bench_field_path_polygon_centerline,
    bench_field_path_segment_centerline,
    bench_color_path_pipeline,
    bench_smooth_curve_preview
);
criterion_main!(tool_preview_hotpath_benches);
