# Typische Workflows

← [Extras & Optionen](06-extras.md) | [Zurueck zur Uebersicht](index.md)

## Workflow 1: Kurs oeffnen und Arbeitsumgebung vorbereiten

1. **Datei oeffnen** mit **`Ctrl+O`**.
2. Im Dialog **Nach dem Laden erkannt** pruefen, ob Heightmap und Hintergrundbild automatisch gesetzt wurden.
3. Wenn ein passender Map-Mod-ZIP angeboten wird, **Uebersichtskarte generieren** waehlen. Falls kein Treffer passt, im selben Dialog **ZIP-Datei auswaehlen** verwenden, dann Layer einstellen und die erzeugte Karte speichern, sobald die Layer-Auswahl verfuegbar sein soll.
4. Nach dem Speichern als `overview.png` ist das Layer-Menue sofort verfuegbar. Beim naechsten Oeffnen stellt der Editor dasselbe gespeicherte Layer-Bundle bevorzugt mit Ihren Default-Layern wieder her. Ohne `overview_terrain.png` bleibt automatisch nur der Legacy-Fallback ueber `overview.png` oder `overview.jpg` aktiv.
4. Falls noetig zusaetzlich eine Hintergrundkarte ueber **Ansicht -> Hintergrund laden...** laden.
5. Im linken Panel Richtung und Strassenart fuer neue Verbindungen voreinstellen.

> **Tipp:** Analyse-Tools zeigen schon jetzt sichtbar an, ob noch Farmland-Daten oder eine Hintergrundkarte fehlen.
> **Tipp:** Wenn nach der Uebersichtskarten-Generierung eine Warnung in der Statusleiste erscheint, betrifft das haeufig nur das Merken der Layer-Voreinstellungen. Die erzeugte Karte kann trotzdem bereits geladen und nutzbar sein.

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

> **Tipp:** Wenn Sie eine neu generierte Uebersichtskarte spaeter wiederverwenden wollen, bestaetigen Sie danach den Dialog zum Speichern als `overview.png`. Der Editor speichert dann auch die einzelnen Layer-Dateien und `overview.json`, sodass das Layer-Menue sofort und auch beim naechsten XML-Load wieder verfuegbar ist. **Feld erkennen** erzeugt spaeter wieder oeffenbare Tool-Gruppen. **Feldweg erkennen** erzeugt ein normales Ergebnis ohne spaeteren Tool-Edit.

---

## Workflow 6: Farb-Pfad aus einer Hintergrundkarte ableiten

Der Ablauf ist Single-Step — sampeln, **Berechnen**, im Editor live justieren, **Uebernehmen**:

1. Eine passende Hintergrundkarte laden.
2. **Farb-Pfad erkennen** ueber **`A`** oder die Command Palette aktivieren.
3. **Sampling:** Mit Klick oder **`Alt+Drag`** Farbproben sammeln. Toleranz und Rauschfilter im Tool-Panel einstellen.
4. **Berechnen** klicken (oder **`Enter`** druecken), um das erkannte Netz zu erzeugen.
5. **Editor-Phase:** Geometrie- und Matching-Slider wirken live auf die Vorschau, Kreuzungspunkte koennen per Drag verschoben werden. **Knotenabstand**, **Radius Kreuzung** und Anschlussmodus dort einstellen.
6. Mit **Uebernehmen** das Netz als finale Nodes und Verbindungen einfuegen.

> **Tipp:** **Reset** verwirft Sampling und Editor-Stand und beginnt von vorn. **Berechnen** ist nur in der Sampling-Phase sichtbar, **Uebernehmen** nur im Editor. Das Tool bleibt im Katalog sichtbar, auch wenn noch keine Hintergrundkarte geladen ist; so sehen Sie sofort, warum es gerade nicht aktivierbar ist.

> **Hinweis:** **Radius Kreuzung** beeinflusst nur die Kreuzungsbegradigung. Die finalen Streckenlaengen und der Abstand der erzeugten Nodes bleiben an **Knotenabstand** gekoppelt.

> **Zukunftsnotiz:** Die erkannten Mittellinien bleiben intern erhalten und sind die Grundlage fuer eine geplante Funktion **Zweispurige Strassen**, die aus einer Mittellinie zwei parallele Fahrspuren ableiten wird.

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

## Workflow 9: Selektion oder Verschieben rueckgaengig machen

1. Nodes per Klick, Rechteck, Lasso oder Doppelklick selektieren.
2. Falls noetig die Selektion per Drag verschieben.
3. Mit **`Ctrl+Z`** die letzte Selektions- oder Verschiebeaktion rueckgaengig machen.
4. Mit **`Ctrl+Y`** oder **`Shift+Ctrl+Z`** den Schritt wiederherstellen.

> **Tipp:** Ein kompletter Drag zaehlt als ein einzelner Undo-Schritt. Rechteck- und Lasso-Selektion lassen sich genauso rueckgaengig machen wie Klick-Selektionen.

---

← [Extras & Optionen](06-extras.md) | [Zurueck zur Uebersicht](index.md)
