# Extras: Streckenteilung, Duplikate, Optionen

← [Karte & Hintergrund](05-karte.md) | [Zurueck zur Uebersicht](index.md)

## Streckenteilung (Distanzen-Neuverteilung)

Mit der Streckenteilung lassen sich Wegpunkt-Abstaende gleichmaessig neu verteilen. Das ist nuetzlich, wenn AutoDrive zu grobe Abstaende fuer bestimmte Manoever hat.

### Zugang

In der **Toolbar** oder ueber **Bearbeiten → Strecke aufteilen**

### Verwendung

1. Einen zusammenhaengenden Pfad selektieren (Start- und Endpunkt auswaehlen)
2. **Zielabstand** in der Optionsleiste eingeben (z. B. 5 Meter)
3. "Strecke aufteilen" ausfuehren

### Ergebnis

Die Nodes entlang des Pfades werden so neu platziert, dass ihre Abstaende dem Zielwert entsprechen. Vorhandene Verbindungsrichtungen und Prioritaeten werden soweit moeglich beibehalten.

> **Hinweis:** Die Operation ist via Undo rueckgaengig zu machen.

---

## Duplikat-Bereinigung

AutoDrive-Courses aus verschiedenen Quellen koennen doppelte Nodes (selbe oder sehr aehnliche Koordinaten) enthalten. Diese koennen zu Routing-Problemen fuehren.

### Zugang

**Bearbeiten → Duplikate bereinigen**

### Vorgehensweise

1. Ein **Toleranz-Radius** wird eingestellt (Standard: 0,5 m)
2. Nodes innerhalb des Radius gelten als Duplikate
3. Von jedem Duplikat-Cluster wird ein Node behalten, alle anderen geloescht
4. Verbindungen werden auf den verbleibenden Node umgeleitet

### Ergebnis-Anzeige

Die Statusleiste zeigt an, wie viele Duplikate entfernt wurden.

> **Hinweis:** Die Operation ist via Undo rueckgaengig zu machen.

---

## Optionen

Ueber **Datei → Einstellungen** oder die **Optionen-Toolbar** erreichbar.

### Anzeige-Optionen

| Option | Beschreibung |
|--------|--------------|
| **Node-Groesse** | Groesse der Wegpunkt-Kreise im Viewport |
| **Verbindungs-Breite** | Linienbreite der Verbindungen |
| **Marker anzeigen** | Map-Marker ein-/ausblenden |
| **Node-IDs anzeigen** | Numerische IDs ueber Nodes einblenden |
| **Grid anzeigen** | Hilfsgitter im Viewport |

### Render-Qualitaet

| Qualitaet | Beschreibung |
|----------|--------------|
| **Low** | Einfache Kreise, keine Anti-Aliasing (maximale Performance) |
| **Medium** | Geglaettete Kreise, Standard |
| **High** | Volles Anti-Aliasing, Spline-Kurven geglaettet |

Einstellung ueber **Ansicht → Render-Qualitaet** oder Dropdown in der Toolbar.

### Snap-Optionen

| Option | Standardwert | Beschreibung |
|--------|-------------|--------------|
| **Snap-Radius** | 5 m | Wie nah ein Klick an einen Node muss, um zu snappen |
| **Grid-Snap** | Aus | An Grid-Schnittpunkte snappen |
| **Grid-Groesse** | 10 m | Rasterweite bei aktiviertem Grid-Snap |

### Kurven-Optionen

| Option | Standardwert | Beschreibung |
|--------|-------------|--------------|
| **Segmentanzahl** | 16 | Anzahl der Liniensegmente pro Kurve (hoeher = glatter) |
| **Kurventyp** | Cubic | Standard-Bézier-Grad (Linear / Quadratic / Cubic) |

### Persistenz

Alle Optionen werden in der **`fs25_auto_drive_editor.toml`** im Anwendungs-Konfigurationsverzeichnis gespeichert und beim naechsten Start automatisch geladen.

---

## Farbcodierung Referenz

| Farbe | Element | Bedeutung |
|-------|---------|-----------|
| **Gruen** | Verbindungspfeil | Regular (Einrichtung) |
| **Blau** | Verbindungspfeil | Dual (bidirektional) |
| **Orange** | Verbindungspfeil | Reverse |
| **Gelb** | Verbindung | SubPriority (duenner) |
| **Weiss** | Node-Kreis | Normal (nicht selektiert) |
| **Gelb** | Node-Kreis | Selektiert |
| **Hellblau** | Node-Kreis | Hover |
| **Rot** | Map-Marker | Benanntes Ziel |
| **Cyan** | Vorschau | Tool-Vorschau (noch nicht bestaetigt) |

---

## Dateiformat (Referenz)

Der Editor liest und schreibt **AutoDrive-XML** im FS25-Format:

```xml
<AutoDrive>
  <waypoints count="N">
    <wp id="1" x="..." y="..." z="..." angle="..." out="2,3" incoming="4" flags="0"/>
    ...
  </waypoints>
  <mapmarker count="M">
    <mm id="1" name="Hof" group="Farm" wpId="5"/>
    ...
  </mapmarker>
</AutoDrive>
```

**Wichtige Regeln:**

- IDs muessen lueckenlos von 1 bis N durchnummeriert sein
- `out` = ausgehende Verbindungen (kommaseparierte IDs)
- `incoming` = eingehende Verbindungen (kommaseparierte IDs)
- `flags` = immer 0 (1/2/4 sind FS22-Artefakte, werden beim Laden bereinigt)
- Der XML-Writer remappt IDs automatisch auf 1..N beim Speichern

---

← [Karte & Hintergrund](05-karte.md) | [Zurueck zur Uebersicht](index.md) | → [Typische Workflows](07-workflows.md)
