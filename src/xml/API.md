# XML API Documentation

## Überblick

Das `xml`-Modul implementiert Import/Export von AutoDrive XML-Konfigurationen im Structure of Arrays (SoA) Format.

## Funktionen

### `parse_autodrive_config`

Parsed eine AutoDrive-Config aus einem XML-String.

```rust
pub fn parse_autodrive_config(xml_content: &str) -> Result<RoadMap>
```

**Beispiel:**
```rust
let xml = std::fs::read_to_string("config.xml")?;
let road_map = parse_autodrive_config(&xml)?;

println!("Nodes: {}", road_map.node_count());
```

**Features:**
- Parst SoA-Format (parallele Listen: `<id>`, `<x>`, `<z>`, etc.)
- Delimiter: Komma (`,`) für Listen, Semikolon (`;`) für verschachtelt
- **Flag-Bereinigung:** Flags 2 und 4 werden automatisch zu 0 konvertiert
- Robustes ID-Mapping über HashMap
- Rekonstruiert Connections aus `out`/`incoming`-Listen

**Fehler:**
- `anyhow::Error` bei Parsing-Fehler oder fehlenden Pflichtfeldern

---

### `write_autodrive_config`

Schreibt eine RoadMap als AutoDrive XML-Config.

```rust
pub fn write_autodrive_config(road_map: &RoadMap, heightmap: Option<&Heightmap>) -> Result<String>
```

**Beispiel:**
```rust
// Ohne Heightmap (Y-Werte = 0.0)
let xml = write_autodrive_config(&road_map, None)?;
std::fs::write("output.xml", xml)?;

// Mit Heightmap (Y-Werte aus PNG berechnet)
let heightmap = Heightmap::load("map_heightmap.png", WorldBounds::default_fs25())?;
let xml = write_autodrive_config(&road_map, Some(&heightmap))?;
std::fs::write("output.xml", xml)?;
```

**Features:**
- Sortierte Node-IDs (stabil)
- Berechnet `out`/`incoming`-Listen aus Connections
- Schreibt MapMarkers als `<mm>`-Elemente
- Float-Formatierung: 3 Dezimalstellen für Koordinaten
- XML-Escaping für Strings
- Exakte Replikation des Original-Formats (encoding, standalone)

**Output-Format:**
```xml
<?xml version="1.0" encoding="utf-8" standalone="no"?>
<AutoDrive>
    <MapName>Example Farm</MapName>
    <waypoints>
        <id>1,2,3</id>
        <x>100.500,200.300,150.700</x>
        <y>0.000,0.000,0.000</y>
        <z>300.100,350.800,320.500</z>
        <out>2,3;3;;1</out>
        <incoming>3;1;2</incoming>
        <flags>0,0,1</flags>
    </waypoints>
    <mapmarker>
        <mm id="1" name="Loading Point" group="All" />
    </mapmarker>
</AutoDrive>
```

## Datenformat-Details

### Structure of Arrays (SoA)

Daten werden als parallele Listen gespeichert:
- **Listen:** Durch Komma getrennt (`1,2,3`)
- **Verschachtelt:** Durch Semikolon getrennt (`2,3;4;`)
- **Leer:** Leere Liste zwischen Semikola

### Connections-Ableitung

```rust
// out: "2,3" -> Node 1 verbindet zu 2 und 3
// incoming: "1" -> Node 2 empfängt von 1

// Direction bestimmen:
// - Dual: Wenn A->B UND B->A
// - Reverse: Wenn nur A->B, aber B kennt A nicht in incoming
// - Regular: Sonst
```

### Version-Support

- **Version 3:** FS25 (primär)
- **Legacy:** Flags 2/4 werden beim Import bereinigt

## Roundtrip-Garantie

Import → Export → Import sollte identische Daten liefern (außer Whitespace).
