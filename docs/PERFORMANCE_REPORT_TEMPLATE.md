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

## Ausgeführte Kommandos
- `cargo bench`
- `cargo bench --bench core_bench -- spatial_queries/nearest_batch/100000`
- `cargo bench --bench core_bench -- spatial_queries/rect_query/100000`

## Ergebnisse (Criterion)
| Benchmark | Datensatzgröße | Mittelwert | p95 | Throughput/ops | Kommentar |
|---|---:|---:|---:|---:|---|
| `xml_parse_simple_config` | n/a |  |  |  |  |
| `spatial_queries/nearest_batch/10000` | 10k |  |  |  |  |
| `spatial_queries/nearest_batch/100000` | 100k |  |  |  |  |
| `spatial_queries/rect_query/10000` | 10k |  |  |  |  |
| `spatial_queries/rect_query/100000` | 100k |  |  |  |  |

## Beobachtungen
- Hotspots:
- Auffällige Regressionen:
- Erwartung vs. Ergebnis:

## Akzeptanzkriterien (Vorschlag)
- `nearest_batch/100000` bleibt stabil über 3 Läufe (Abweichung < 10%).
- Kein signifikanter Regressionssprung (> 15%) gegenüber letzter Baseline.
- `cargo check` und `cargo test` bleiben grün.

## Nächste Maßnahmen
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

## Ausgeführte Kommandos
- `cargo check`
- `cargo test`
- `cargo bench --bench core_bench -- --noplot`

## Ergebnisse (Baseline)
| Benchmark | Datensatzgröße | Mittelwert | p95 | Throughput/ops | Kommentar |
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

Kurzregel für Bewertung:
- Regressionsverdacht bei > 15% langsamer gegenüber Baseline.
- Bei Ausreißern Bench 3x wiederholen und Median vergleichen.

---

## Baseline 2026-02-17 (3x Wiederholung + Median)

## Ausgeführte Kommandos
- `cargo bench --bench core_bench -- --noplot` (zweimal wiederholt; zusammen mit der initialen Baseline = 3 Läufe)

## 3-Lauf-Werte (Punkt-Schätzung)
| Benchmark | Lauf A (initial) | Lauf B | Lauf C | Median (3 Läufe) |
|---|---:|---:|---:|---:|
| `xml_parse_simple_config` | 3.7438 µs | 3.9584 µs | 3.9313 µs | 3.9313 µs |
| `spatial_queries/nearest_batch/10000` | 6.1266 ms | 6.0737 ms | 6.4868 ms | 6.1266 ms |
| `spatial_queries/rect_query/10000` | 45.090 µs | 45.452 µs | 45.262 µs | 45.262 µs |
| `spatial_queries/nearest_batch/100000` | 65.067 ms | 63.972 ms | 65.299 ms | 65.067 ms |
| `spatial_queries/rect_query/100000` | 702.39 µs | 693.36 µs | 727.49 µs | 702.39 µs |

## Medianvergleich zur ersten Baseline
| Benchmark | Erste Baseline | Median (3 Läufe) | Delta |
|---|---:|---:|---:|
| `xml_parse_simple_config` | 3.7438 µs | 3.9313 µs | +5.01% |
| `spatial_queries/nearest_batch/10000` | 6.1266 ms | 6.1266 ms | +0.00% |
| `spatial_queries/rect_query/10000` | 45.090 µs | 45.262 µs | +0.38% |
| `spatial_queries/nearest_batch/100000` | 65.067 ms | 65.067 ms | +0.00% |
| `spatial_queries/rect_query/100000` | 702.39 µs | 702.39 µs | +0.00% |

## Kurzfazit
- Keine >15%-Regression gegenüber der ersten Baseline.
- `xml_parse_simple_config` schwankt sichtbar, bleibt aber deutlich unter kritischer Regressionsschwelle.
- Für `nearest_batch/100000` weiterhin längere Benchmark-Zeit einplanen (Criterion-Hinweis auf Sample-Zeit).
