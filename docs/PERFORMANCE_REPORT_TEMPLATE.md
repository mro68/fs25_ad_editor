# Performance Report Template

## Kontext

- Datum:
- Commit:
- Branch:
- Build-Profil (`debug`/`release`):
- Datensatz (Datei/Generator, Node-/Connection-Anzahl):

## Umgebung

- OS:
- CPU:
- RAM:
- GPU + Treiber:
- Rust-Version:

## Ausgefuehrte Kommandos

- `cargo bench`
- `cargo bench --bench core_bench -- spatial_queries/nearest_batch/100000`
- `cargo bench --bench core_bench -- spatial_queries/rect_query/100000`

## Ergebnisse (Criterion)

| Benchmark | Datensatzgroesse | Mittelwert | p95 | Throughput/ops | Kommentar |
|---|---:|---:|---:|---:|---|
| `xml_parse_simple_config` | n/a |  |  |  |  |
| `spatial_queries/nearest_batch/10000` | 10k |  |  |  |  |
| `spatial_queries/nearest_batch/100000` | 100k |  |  |  |  |
| `spatial_queries/rect_query/10000` | 10k |  |  |  |  |
| `spatial_queries/rect_query/100000` | 100k |  |  |  |  |

## Beobachtungen

- Hotspots:
- Auffaellige Regressionen:
- Erwartung vs. Ergebnis:

## Akzeptanzkriterien (Vorschlag)

- `nearest_batch/100000` bleibt stabil ueber 3 Laeufe (Abweichung < 10%).
- Kein signifikanter Regressionssprung (> 15%) gegenueber letzter Baseline.
- `cargo check` und `cargo test` bleiben gruen.

## Naechste Massnahmen

1.
2.
3.

---

## Baseline 2026-02-17

## Kontext

- Datum: 2026-02-17
- Commit: lokaler Arbeitsstand (nicht getaggt)
- Branch: lokaler Arbeitsstand
- Build-Profil (`debug`/`release`): `bench` (release-optimiert)
- Datensatz (Datei/Generator, Node-/Connection-Anzahl):
 	- XML: `tests/fixtures/simple_config.xml`
 	- Spatial: synthetisch, 10k und 100k Nodes

## Ausgefuehrte Kommandos

- `cargo check`
- `cargo test`
- `cargo bench --bench core_bench -- --noplot`

## Ergebnisse (Baseline)

| Benchmark | Datensatzgroesse | Mittelwert | p95 | Throughput/ops | Kommentar |
|---|---:|---:|---:|---:|---|
| `xml_parse_simple_config` | n/a | 3.7438 µs | 3.7806 µs | n/a | stabil |
| `spatial_queries/nearest_batch/10000` | 10k | 6.1266 ms | 6.1827 ms | n/a | stabil |
| `spatial_queries/nearest_batch/100000` | 100k | 65.067 ms | 65.283 ms | n/a | Criterion: Sample-Zeit knapp |
| `spatial_queries/rect_query/10000` | 10k | 45.090 µs | 45.892 µs | n/a | stabil |
| `spatial_queries/rect_query/100000` | 100k | 702.39 µs | 709.17 µs | n/a | stabil |

## Regressions-Check (Kommandoabfolge)

1. `cargo check`
2. `cargo test`
3. `cargo bench --bench core_bench -- --noplot`
4. `cargo bench --bench core_bench -- spatial_queries/nearest_batch/100000 --noplot`
5. `cargo bench --bench core_bench -- spatial_queries/rect_query/100000 --noplot`

Kurzregel fuer Bewertung:

- Regressionsverdacht bei > 15% langsamer gegenueber Baseline.
- Bei Ausreissern Bench 3x wiederholen und Median vergleichen.

---

## Baseline 2026-02-17 (3x Wiederholung + Median)

## Ausgefuehrte Kommandos

- `cargo bench --bench core_bench -- --noplot` (zweimal wiederholt; zusammen mit der initialen Baseline = 3 Laeufe)

## 3-Lauf-Werte (Punkt-Schaetzung)

| Benchmark | Lauf A (initial) | Lauf B | Lauf C | Median (3 Laeufe) |
|---|---:|---:|---:|---:|
| `xml_parse_simple_config` | 3.7438 µs | 3.9584 µs | 3.9313 µs | 3.9313 µs |
| `spatial_queries/nearest_batch/10000` | 6.1266 ms | 6.0737 ms | 6.4868 ms | 6.1266 ms |
| `spatial_queries/rect_query/10000` | 45.090 µs | 45.452 µs | 45.262 µs | 45.262 µs |
| `spatial_queries/nearest_batch/100000` | 65.067 ms | 63.972 ms | 65.299 ms | 65.067 ms |
| `spatial_queries/rect_query/100000` | 702.39 µs | 693.36 µs | 727.49 µs | 702.39 µs |

## Medianvergleich zur ersten Baseline

| Benchmark | Erste Baseline | Median (3 Laeufe) | Delta |
|---|---:|---:|---:|
| `xml_parse_simple_config` | 3.7438 µs | 3.9313 µs | +5.01% |
| `spatial_queries/nearest_batch/10000` | 6.1266 ms | 6.1266 ms | +0.00% |
| `spatial_queries/rect_query/10000` | 45.090 µs | 45.262 µs | +0.38% |
| `spatial_queries/nearest_batch/100000` | 65.067 ms | 65.067 ms | +0.00% |
| `spatial_queries/rect_query/100000` | 702.39 µs | 702.39 µs | +0.00% |

## Kurzfazit

- Keine >15%-Regression gegenueber der ersten Baseline.
- `xml_parse_simple_config` schwankt sichtbar, bleibt aber deutlich unter kritischer Regressionsschwelle.
- Fuer `nearest_batch/100000` weiterhin laengere Benchmark-Zeit einplanen (Criterion-Hinweis auf Sample-Zeit).
