# Typische Workflows

← [Extras & Optionen](06-extras.md) | [Zurueck zur Uebersicht](index.md)

## Workflow 1: Kurs oeffnen und Arbeitsumgebung vorbereiten

1. **Datei oeffnen** mit **`Ctrl+O`**.
2. Automatische Erkennung fuer Heightmap, Uebersichtskarte oder passendes Map-ZIP pruefen.
3. Falls noetig zusaetzlich eine Hintergrundkarte ueber **Ansicht -> Hintergrund laden...** laden.
4. Im linken Panel Richtung und Strassenart fuer neue Verbindungen voreinstellen.

> **Tipp:** Analyse-Tools zeigen schon jetzt sichtbar an, ob noch Farmland-Daten oder eine Hintergrundkarte fehlen.

---

## Workflow 2: Passendes Route-Tool schnell finden

1. Fuer Basis-Geometrie **`G`** druecken.
2. Fuer Abschnitts-Tools **`B`** druecken.
3. Fuer Analyse-Tools **`A`** druecken.
4. Alternativ die Command Palette mit **`K`** oder **`Ctrl+K`** oeffnen und nach dem Toolnamen suchen.
5. Deaktivierte Eintraege lesen statt suchen: der Disabled-Hinweis sagt direkt, was noch fehlt.

> **Beispiele:**
> **Geordnete Node-Kette selektieren** fuer Ausweichstrecke oder Strecke versetzen.
> **Farmland-Daten zuerst laden** fuer Feld erkennen oder Feldweg erkennen.
> **Hintergrundkarte zuerst laden** fuer Farb-Pfad erkennen.

---

## Workflow 3: Neue Strecke mit Grundbefehlen zeichnen

1. Ein Basis-Tool ueber **`G`**, die Seitenleiste oder **Route-Tools** aktivieren.
2. Start- und Endpunkt setzen.
3. Je nach Tool Kontrollpunkte, Tangenten oder Zwischenpunkte ergaenzen.
4. Richtung, Strassenart und Segmentierung im Route-Tool-Panel einstellen.
5. Mit **`Enter`** bestaetigen.

**Empfehlung nach Anwendungsfall:**

- **Gerade Strecke** fuer lineare Abschnitte.
- **Bezier Grad 2** fuer einfache Kurven.
- **Bezier Grad 3** fuer praezise Formkontrolle mit Start-/End-Tangente.
- **Spline** fuer Verlaeufe durch viele Punkte.
- **Geglaettete Kurve** fuer weich an bestehende Winkel angeschlossene Verbindungen.

---

## Workflow 4: Ausweichstrecke oder Versatz auf eine bestehende Kette anwenden

1. Eine geordnete Kette selektieren, zum Beispiel per **Shift+Klick**, Rechteck-Lasso oder Doppelklick.
2. Mit **`B`** die Gruppe **Bearbeiten** oeffnen.
3. **Ausweichstrecke** oder **Strecke versetzen** waehlen.
4. Vorschau und Parameter pruefen.
5. Mit **`Enter`** bestaetigen.

> **Tipp:** Wenn das Tool im Menue sichtbar, aber deaktiviert ist, bildet die aktuelle Selektion keine gueltige geordnete Kette.

---

## Workflow 5: Feldgrenze oder Feldweg aus Farmland-Daten erzeugen

1. Eine Uebersichtskarte mit Farmland-Daten laden oder ueber **Datei -> Uebersichtskarte generieren** erzeugen.
2. Mit **`A`** die Analyse-Gruppe oeffnen.
3. Fuer geschlossene Feldringe **Feld erkennen** waehlen.
4. Fuer eine Mittellinie zwischen zwei Feldseiten **Feldweg erkennen** waehlen.
5. Die Vorschau pruefen und mit **`Enter`** uebernehmen.

> **Tipp:** **Feld erkennen** erzeugt spaeter wieder oeffenbare Tool-Gruppen. **Feldweg erkennen** erzeugt ein normales Ergebnis ohne spaeteren Tool-Edit.

---

## Workflow 6: Farb-Pfad aus einer Hintergrundkarte ableiten

1. Eine passende Hintergrundkarte laden.
2. **Farb-Pfad erkennen** ueber **`A`** oder die Command Palette aktivieren.
3. Mit **`Alt+Drag`** eine oder mehrere Lasso-Regionen fuer Farbproben zeichnen.
4. Im Tool-Panel Berechnung, Toleranz und Anschlussmodus einstellen.
5. Netzstatistik pruefen und mit **`Enter`** bestaetigen.

> **Tipp:** Das Tool bleibt im Katalog sichtbar, auch wenn noch keine Hintergrundkarte geladen ist. So sehen Sie sofort, warum es gerade nicht aktivierbar ist.

---

## Workflow 7: Eine Gruppe spaeter erneut oeffnen

1. Die Gruppe per Doppelklick auf einen Gruppen-Node selektieren.
2. Rechtsklick und **Gruppe bearbeiten** waehlen.
3. Im Gruppen-Bearbeitungsfenster pruefen, ob **Tool bearbeiten** verfuegbar ist.
4. Falls der Button sichtbar ist, das urspruengliche Tool erneut oeffnen und Parameter anpassen.
5. Falls der Button fehlt, das Ergebnis manuell bearbeiten oder das Tool neu ausfuehren.

**Der Button fehlt absichtlich bei:**

- manuell erzeugten Gruppen
- Ergebnissen von **Feldweg erkennen**
- Ergebnissen von **Farb-Pfad erkennen**

---

## Workflow 8: Strecke gleichmaessig neu verteilen und exportieren

1. Eine zusammenhaengende Kette selektieren.
2. Im Eigenschaften-Bereich **▶ Einteilung aendern** waehlen oder das Kontextmenue **Streckenteilung** nutzen.
3. Abstand oder Node-Anzahl anpassen.
4. Vorschau bestaetigen und mit **`Enter`** uebernehmen.
5. Mit **`Ctrl+S`** speichern und die Datei in FS25 testen.

> **Wichtig:** Der Editor remappt IDs beim Speichern automatisch auf eine lueckenlose Reihenfolge. Manuelle XML-Korrekturen sind dafuer nicht noetig.

---

← [Extras & Optionen](06-extras.md) | [Zurueck zur Uebersicht](index.md)
