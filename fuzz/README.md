# Fuzzing-Targets für fs25_auto_drive_editor

## Überblick

Dieses Verzeichnis enthält Fuzzing-Tests für die wichtigsten Parser und externe Input-Verarbeitung:

- **fuzz_xml_parser**: Fuzzt den AutoDrive-Config XML-Parser (`parse_autodrive_config`)
- **fuzz_curseplay_parser**: Fuzzt den CursePlay-XML-Parser (`parse_curseplay`)

Die Fuzzing-Ziele überprüfen, ob die Parser mit adversarialem Input robust sind und nicht zu Panics oder Out-of-Memory-Bedingungen führen.

## Lokale Ausführung

### Voraussetzung: cargo-fuzz installieren

```bash
cargo install cargo-fuzz
```

### 1. XML-Parser fuzzen

```bash
# Unbegrenztes Fuzzing ausführen (mit Ctrl+C beenden)
cargo +nightly fuzz run fuzz_xml_parser

# Mit Timeout (z.B. 60 Sekunden)
cargo +nightly fuzz run fuzz_xml_parser -- -max_len=10000 -timeout=1

# Mit limitiertem Corpus
cargo +nightly fuzz run fuzz_xml_parser -- -max_total_time=60

# Gefundene Crash-Artefakte inspizieren
ls fuzz/artifacts/fuzz_xml_parser/
```

### 2. CursePlay-Parser fuzzen

```bash
# Unbegrenztes Fuzzing ausführen
cargo +nightly fuzz run fuzz_curseplay_parser

# Mit Timeout
cargo +nightly fuzz run fuzz_curseplay_parser -- -timeout=1 -max_len=10000
```

### 3. Corpus seeding (optional)

Die initiale Corpus wird unter `fuzz/corpus/` gespeichert. Um zusätzliche Test-Inputs hinzuzufügen:

```bash
# AutoDrive-Config Sample hinzufügen
cp ad_sample_data/AutoDrive_config.xml fuzz/corpus/fuzz_xml_parser/sample.xml

# CursePlay Sample (falls vorhanden)
cp ad_sample_data/some_cursplay.xml fuzz/corpus/fuzz_curseplay_parser/sample.xml

# Fuzzing erneut ausführen
cargo +nightly fuzz run fuzz_xml_parser -- -timeout=1
```

## Ausgabe interpretieren

### Erfolgreiche Ausführung
```
#123456 NEW   cov: 3456 ft: 4567 exec/s: 1234 rss: 45Mb L: 1024 ...
```
- `cov`: Coverage-Anzahl (erreichte Code-Pfade)
- `ft`: Feature-Tokenanzahl (Code-Variationen)
- `exec/s`: Ausführungen pro Sekunde

### Crash gefunden

```
ERROR: libFuzzer: out-of-memory: allocating 1GB more than limit 2GB
artifact_prefix='fuzz/artifacts/fuzz_xml_parser/'; Test unit written to 'fuzz/artifacts/fuzz_xml_parser/oom-<hash>'
```

Crash-Artefakte werden in `fuzz/artifacts/fuzz_xml_parser/` gespeichert und können mit regulären Tests reproduziert werden.

## CI-Integration

Diese Fuzzing-Targets werden aktuell in der CI noch nicht automatisch ausgeführt (zu zeitaufwändig für jeden Commit). Sie können manuell oder in nächtlichen Builds aktiviert werden:

```yaml
# .github/workflows/scheduled-fuzz.yml (optional)
- name: Run fuzzing (timeout 5 min per target)
  run: |
    cargo +nightly fuzz run fuzz_xml_parser -- -max_total_time=300
    cargo +nightly fuzz run fuzz_curseplay_parser -- -max_total_time=300
```

## Hintergrund

**Fuzzing** ist eine automated security-testing Methode, die zufällige und mutierte Eingaben an die Programm-Funktionen sendet, um unerwartete Fehler, Panics oder Speicherlecks zu finden.

Für Parser ist Fuzzing besonders wichtig, da externe Dateien von untrusted Quellen kommen können (User-Upload, Mod-Repositories, etc.).

## Weitere Ressourcen

- [cargo-fuzz Dokumentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer Flags](https://llvm.org/docs/LibFuzzer/)
- [OWASP Fuzzing Guide](https://owasp.org/www-community/attacks/Fuzzing)
