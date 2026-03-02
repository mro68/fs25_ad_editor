# Extras: Streckenteilung, Duplikate, Optionen

← [Karte & Hintergrund](05-karte.md) | [Zurück zur Übersicht](index.md)

## Streckenteilung (Distanzen-Neuverteilung)

Mit der Streckenteilung lassen sich Wegpunkt-Abstände gleichmäßig neu verteilen. Das ist nützlich, wenn AutoDrive zu grobe Abstände für bestimmte Manöver hat.

### Zugang

In der **Toolbar** oder über **Bearbeiten → Strecke aufteilen**

### Verwendung

1. Einen zusammenhängenden Pfad selektieren (Start- und Endpunkt auswählen)
2. **Zielabstand** in der Optionsleiste eingeben (z. B. 5 Meter)
3. "Strecke aufteilen" ausführen

### Ergebnis

Die Nodes entlang des Pfades werden so neu platziert, dass ihre Abstände dem Zielwert entsprechen. Vorhandene Verbindungsrichtungen und Prioritäten werden soweit möglich beibehalten.

> **Hinweis:** Die Operation ist via Undo rückgängig zu machen.

---

## Duplikat-Bereinigung

AutoDrive-Courses aus verschiedenen Quellen können doppelte Nodes (selbe oder sehr ähnliche Koordinaten) enthalten. Diese können zu Routing-Problemen führen.

### Zugang

**Bearbeiten → Duplikate bereinigen**

### Vorgehensweise

1. Ein **Toleranz-Radius** wird eingestellt (Standard: 0,5 m)
2. Nodes innerhalb des Radius gelten als Duplikate
3. Von jedem Duplikat-Cluster wird ein Node behalten, alle anderen gelöscht
4. Verbindungen werden auf den verbleibenden Node umgeleitet

### Ergebnis-Anzeige

Die Statusleiste zeigt an, wie viele Duplikate entfernt wurden.

> **Hinweis:** Die Operation ist via Undo rückgängig zu machen.

---

## Optionen

Über **Datei → Einstellungen** oder die **Optionen-Toolbar** erreichbar.

### Anzeige-Optionen

| Option | Beschreibung |
|--------|--------------|
| **Node-Größe** | Größe der Wegpunkt-Kreise im Viewport |
| **Verbindungs-Breite** | Linienbreite der Verbindungen |
| **Marker anzeigen** | Map-Marker ein-/ausblenden |
| **Node-IDs anzeigen** | Numerische IDs über Nodes einblenden |
| **Grid anzeigen** | Hilfsgitter im Viewport |

### Render-Qualität

| Qualität | Beschreibung |
|----------|--------------|
| **Low** | Einfache Kreise, keine Anti-Aliasing (maximale Performance) |
| **Medium** | Geglättete Kreise, Standard |
| **High** | Volles Anti-Aliasing, Spline-Kurven geglättet |

Einstellung über **Ansicht → Render-Qualität** oder Dropdown in der Toolbar.

### Snap-Optionen

| Option | Standardwert | Beschreibung |
|--------|-------------|--------------|
| **Snap-Radius** | 5 m | Wie nah ein Klick an einen Node muss, um zu snappen |
| **Grid-Snap** | Aus | An Grid-Schnittpunkte snappen |
| **Grid-Größe** | 10 m | Rasterweite bei aktiviertem Grid-Snap |

### Kurven-Optionen

| Option | Standardwert | Beschreibung |
|--------|-------------|--------------|
| **Segmentanzahl** | 16 | Anzahl der Liniensegmente pro Kurve (höher = glatter) |
| **Kurventyp** | Cubic | Standard-Bézier-Grad (Linear / Quadratic / Cubic) |

### Persistenz

Alle Optionen werden in der **`fs25_auto_drive_editor.toml`** im Anwendungs-Konfigurationsverzeichnis gespeichert und beim nächsten Start automatisch geladen.

---

## Farbcodierung Referenz

| Farbe | Element | Bedeutung |
|-------|---------|-----------|
| **Grün** | Verbindungspfeil | Regular (Einrichtung) |
| **Blau** | Verbindungspfeil | Dual (bidirektional) |
| **Orange** | Verbindungspfeil | Reverse |
| **Gelb** | Verbindung | SubPriority (dünner) |
| **Weiß** | Node-Kreis | Normal (nicht selektiert) |
| **Gelb** | Node-Kreis | Selektiert |
| **Hellblau** | Node-Kreis | Hover |
| **Rot** | Map-Marker | Benanntes Ziel |
| **Cyan** | Vorschau | Tool-Vorschau (noch nicht bestätigt) |

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
- IDs müssen lückenlos von 1 bis N durchnummeriert sein
- `out` = ausgehende Verbindungen (kommaseparierte IDs)
- `incoming` = eingehende Verbindungen (kommaseparierte IDs)
- `flags` = immer 0 (1/2/4 sind FS22-Artefakte, werden beim Laden bereinigt)
- Der XML-Writer remappt IDs automatisch auf 1..N beim Speichern

---

← [Karte & Hintergrund](05-karte.md) | [Zurück zur Übersicht](index.md) | → [Typische Workflows](07-workflows.md)
