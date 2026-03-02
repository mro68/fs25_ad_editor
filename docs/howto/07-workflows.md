# Typische Workflows

← [Extras & Optionen](06-extras.md) | [Zurück zur Übersicht](index.md)

## Workflow 1: Neuen Kurs von Grund auf erstellen

1. **Leere Datei erzeugen** — `Datei → Neu` (oder `Ctrl+N`)
2. **Hintergrundkarte laden** (optional) — `Datei → Hintergrundkarte öffnen`
3. **Heightmap laden** (optional) — `Datei → Heightmap laden`
4. **Nodes setzen** — Linksklick-Tool (`1`) aktivieren, Nodes auf der Karte platzieren
5. **Verbindungen setzen** — Connect-Tool (`2`) aktivieren, Nodes paarweise klicken
6. **Speichern** — `Ctrl+S`

> **Tipp:** Mit dem Route-Modus (Taste `R`) können Nodes und Verbindungen in einem Zug gesetzt werden.

---

## Workflow 2: Bestehenden Kurs bearbeiten

1. **Öffnen** — `Datei → Öffnen` oder `Ctrl+O`
2. **Bereich zoomen** — Scrollrad oder `F` nach Selektion
3. **Nodes verschieben** — Selektieren, dann Drag
4. **Falsche Verbindungen löschen** — 2 Nodes selektieren, `X`
5. **Neue Verbindungen setzen** — 2 Nodes selektieren, `C`
6. **Speichern** — `Ctrl+S`

---

## Workflow 3: Zwei Kurse zusammenführen (Merge)

1. **Erste Datei öffnen** — `Ctrl+O`
2. **Zweite Datei importieren** — `Datei → Importieren / Zusammenführen`
3. **Verbindungspunkte suchen** — Betroffene Randbereiche heranzoomen
4. **Snap-Punkte verbinden** — Connect-Tool (`2`), Nodes paarweise anklicken
5. **Duplikate bereinigen** — `Bearbeiten → Duplikate bereinigen` mit Toleranz 1–2 m
6. **Speichern**

---

## Workflow 4: Abschnitt löschen

1. **Bereich selektieren** — Shift+Drag (Rechteck) oder Alt+Drag (Lasso)
2. **Selektion prüfen** — Properties-Panel zeigt Anzahl der selektierten Nodes
3. **Löschen** — `Delete`-Taste
4. **Verbindungen prüfen** — angrenzende Nodes nach losen Enden kontrollieren
5. **Speichern**

---

## Workflow 5: Kurvenabschnitt hinzufügen

1. **Curve-Tool aktivieren** — Taste `4`
2. **Kubischer Bézier-Modus** — im Tool-Panel `Cubic` auswählen
3. **Startpunkt klicken** — auf bestehenden Node oder freie Position
4. **Endpunkt klicken** — zweiter Klick setzt das Ende
5. **Kontrollpunkte anpassen** — Drag der blauen Kontrollpunkt-Handles
6. **Bestätigen** — Enter oder abschließender Klick

> **Tipp:** Den Start/End-Punkt auf einen bestehenden Node setzen sorgt für automatisches Snapping.

---

## Workflow 6: Route gleichmäßig unterteilen

In engen Kurven oder für AutoDrive-Strecken mit feiner Granularität:

1. **Abschnitt selektieren** — Pfad-Selektion mit Shift+Klick oder Segment-Doppelklick
2. **Streckenteilung öffnen** — `Bearbeiten → Strecke aufteilen`
3. **Zielabstand eingeben** (z. B. 3 – 5 m)
4. **Ausführen** — Nodes werden gleichmäßig neu verteilt
5. **Undo** bei Bedarf — `Ctrl+Z`

---

## Workflow 7: Kurs exportieren und in FS25 testen

1. **Speichern** — `Ctrl+S` → Datei wird als `AutoDrive_config.xml` gespeichert
2. **Datei in FS25-Savegame kopieren** — in `Savegame/vehicles/AutoDrive/`
3. **FS25 starten** und AutoDrive laden
4. **Testen** — einen Auftrag mit der neuen Route planen

> **Wichtig:** AutoDrive benötigt lückenlose IDs (1, 2, 3 ... N). Der Editor remappt IDs automatisch beim Speichern — manuelle Korrekturen sind nicht notwendig.

---

← [Extras & Optionen](06-extras.md) | [Zurück zur Übersicht](index.md)
