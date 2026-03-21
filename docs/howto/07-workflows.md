# Typische Workflows

← [Extras & Optionen](06-extras.md) | [Zurueck zur Uebersicht](index.md)

## Workflow 1: Neuen Kurs von Grund auf erstellen

1. **Leere Datei erzeugen** — `Datei → Neu` (oder `Ctrl+N`)
2. **Hintergrundkarte laden** (optional) — `Datei → Hintergrundkarte oeffnen`
3. **Heightmap laden** (optional) — `Datei → Heightmap laden`
4. **Nodes setzen** — **Select-Tool** aktivieren (T → Select), Nodes auf der Karte platzieren
5. **Verbindungen setzen** — **Connect-Tool** aktivieren (T → Connect), Nodes paarweise klicken
6. **Speichern** — `Ctrl+S`

> **Tipp:** Mit den **Route-Tools** (T → Gerade / Kurve / Spline) koennen Nodes und Verbindungen in einem Zug gesetzt werden.

---

## Workflow 2: Bestehenden Kurs bearbeiten

1. **Oeffnen** — `Datei → Oeffnen` oder `Ctrl+O`
2. **Bereich zoomen** — Scrollrad oder `F` nach Selektion
3. **Nodes verschieben** — Selektieren, dann Drag
4. **Falsche Verbindungen loeschen** — 2 Nodes selektieren, `X`
5. **Neue Verbindungen setzen** — 2 Nodes selektieren, `C`
6. **Speichern** — `Ctrl+S`

---

## Workflow 3: Zwei Kurse zusammenfuehren (Merge)

1. **Erste Datei oeffnen** — `Ctrl+O`
2. **Zweite Datei importieren** — `Datei → Importieren / Zusammenfuehren`
3. **Verbindungspunkte suchen** — Betroffene Randbereiche heranzoomen
4. **Snap-Punkte verbinden** — Connect-Tool (T → Connect), Nodes paarweise anklicken
5. **Duplikate bereinigen** — `Bearbeiten → Duplikate bereinigen` mit Toleranz 1–2 m
6. **Speichern**

---

## Workflow 4: Abschnitt loeschen

1. **Bereich selektieren** — Shift+Drag (Rechteck) oder Alt+Drag (Lasso)
2. **Selektion pruefen** — Properties-Panel zeigt Anzahl der selektierten Nodes
3. **Loeschen** — `Delete`-Taste
4. **Verbindungen pruefen** — angrenzende Nodes nach losen Enden kontrollieren
5. **Speichern**

---

## Workflow 5: Kurvenabschnitt hinzufuegen

1. **Curve-Tool aktivieren** — T → Kurve
2. **Kubischer Bézier-Modus** — im Tool-Panel `Cubic` auswaehlen
3. **Startpunkt klicken** — auf bestehenden Node oder freie Position
4. **Endpunkt klicken** — zweiter Klick setzt das Ende
5. **Kontrollpunkte anpassen** — Drag der blauen Kontrollpunkt-Handles
6. **Bestaetigen** — Enter oder abschliessender Klick

> **Tipp:** Den Start/End-Punkt auf einen bestehenden Node setzen sorgt fuer automatisches Snapping.

---

## Workflow 6: Route gleichmaessig unterteilen

In engen Kurven oder fuer AutoDrive-Strecken mit feiner Granularitaet:

1. **Abschnitt selektieren** — Pfad-Selektion mit Shift+Klick oder Gruppen-Doppelklick
2. **Streckenteilung oeffnen** — `Bearbeiten → Strecke aufteilen`
3. **Zielabstand eingeben** (z. B. 3 – 5 m)
4. **Ausfuehren** — Nodes werden gleichmaessig neu verteilt
5. **Undo** bei Bedarf — `Ctrl+Z`

---

## Workflow 7: Kurs exportieren und in FS25 testen

1. **Speichern** — `Ctrl+S` → Datei wird als `AutoDrive_config.xml` gespeichert
2. **Datei in FS25-Savegame kopieren** — in `Savegame/vehicles/AutoDrive/`
3. **FS25 starten** und AutoDrive laden
4. **Testen** — einen Auftrag mit der neuen Route planen

> **Wichtig:** AutoDrive benoetigt lueckenlose IDs (1, 2, 3 ... N). Der Editor remappt IDs automatisch beim Speichern — manuelle Korrekturen sind nicht notwendig.

---

← [Extras & Optionen](06-extras.md) | [Zurueck zur Uebersicht](index.md)
