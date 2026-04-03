//! Benchmark fuer Route-Tool-Preview-Hotpaths.
//!
//! Misst die Kernberechnungen der Preview-Erzeugung in den aufwaendigeren Tools:
//! - Bypass: `compute_bypass_positions`
//! - RouteOffset: `compute_offset_positions`
//! - FieldPath: `compute_polygon_centerline` / `compute_segment_centerline`
//! - ColorPath: echte Sampling-/Preview-Rebuilds des ColorPathTool
//! - SmoothCurve: `solve_route`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fs25_auto_drive_editor::app::tools::bypass::compute_bypass_positions;
use fs25_auto_drive_editor::app::tools::color_path::{
    ColorPathBenchmarkAction, ColorPathBenchmarkHarness,
};
use fs25_auto_drive_editor::app::tools::route_offset::compute_offset_positions;
use fs25_auto_drive_editor::app::tools::smooth_curve::{solve_route, SmoothCurveInput};
use fs25_auto_drive_editor::core::{compute_polygon_centerline, compute_segment_centerline};
use glam::Vec2;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

fn pixel_rect_to_world_polygon(
    size: u32,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
) -> Vec<Vec2> {
    let half = size as f32 / 2.0;
    vec![
        Vec2::new(min_x as f32 - half, min_y as f32 - half),
        Vec2::new(max_x as f32 + 1.0 - half, min_y as f32 - half),
        Vec2::new(max_x as f32 + 1.0 - half, max_y as f32 + 1.0 - half),
        Vec2::new(min_x as f32 - half, max_y as f32 + 1.0 - half),
    ]
}

fn make_color_path_lasso(size: u32) -> Vec<Vec2> {
    let band = (size / 18).max(6);
    let center = size / 2;
    let max = size.saturating_sub(1);
    pixel_rect_to_world_polygon(
        size,
        center.saturating_sub(band),
        center.saturating_sub(band),
        (center + band).min(max),
        (center + band).min(max),
    )
}

fn measure_color_path_action<F>(
    harness: &ColorPathBenchmarkHarness,
    iters: u64,
    prepare: F,
) -> Duration
where
    F: Fn(&ColorPathBenchmarkHarness) -> ColorPathBenchmarkAction,
{
    let mut elapsed = Duration::default();
    for _ in 0..iters {
        let action = prepare(harness);
        let start = Instant::now();
        let stats = action.run();
        elapsed += start.elapsed();
        black_box(stats);
    }
    elapsed
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
        let start_neighbors = [Vec2::new(1.0, 0.0), Vec2::new(0.7, 0.3)];
        let end_neighbors = [Vec2::new(-1.0, 0.0), Vec2::new(-0.7, -0.3)];

        let input = SmoothCurveInput {
            start,
            end,
            control_nodes: control_nodes.as_slice(),
            max_segment_length_m: 6.0,
            max_direction_change_deg: 45.0,
            start_neighbor_directions: &start_neighbors,
            end_neighbor_directions: &end_neighbors,
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

fn bench_color_path_stage_rebuilds(c: &mut Criterion) {
    let mut group = c.benchmark_group("tool_preview_color_path");

    for &size in &[128u32, 256, 512] {
        let harness = ColorPathBenchmarkHarness::new(
            Arc::new(make_color_path_image(size)),
            make_color_path_lasso(size),
        )
        .expect("ColorPath-Benchmark-Harness sollte aufgebaut werden");

        group.bench_with_input(
            BenchmarkId::new("sampling_preview_rebuild", size),
            &harness,
            |b, harness| {
                b.iter_custom(|iters| {
                    measure_color_path_action(
                        harness,
                        iters,
                        ColorPathBenchmarkHarness::sampling_preview_rebuild_action,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("compute_pipeline", size),
            &harness,
            |b, harness| {
                b.iter_custom(|iters| {
                    measure_color_path_action(
                        harness,
                        iters,
                        ColorPathBenchmarkHarness::compute_pipeline_action,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("preview_core_rebuild", size),
            &harness,
            |b, harness| {
                b.iter_custom(|iters| {
                    measure_color_path_action(
                        harness,
                        iters,
                        ColorPathBenchmarkHarness::preview_core_rebuild_action,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("prepared_segments_rebuild", size),
            &harness,
            |b, harness| {
                b.iter_custom(|iters| {
                    measure_color_path_action(
                        harness,
                        iters,
                        ColorPathBenchmarkHarness::prepared_segments_rebuild_action,
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    tool_preview_hotpath_benches,
    bench_bypass_preview,
    bench_route_offset_preview,
    bench_field_path_polygon_centerline,
    bench_field_path_segment_centerline,
    bench_color_path_stage_rebuilds,
    bench_smooth_curve_preview
);
criterion_main!(tool_preview_hotpath_benches);
