#!/usr/bin/env bash
# CI-Check: Architektur-Guardrails fuer Schichtentrennung.
#
# Regeln (Importrichtungen):
#   UI       → App → Core (niemals crate::core direkt)
#   UI       darf nicht crate::xml oder crate::render importieren
#   Render   → Shared (niemals crate::app oder crate::ui)
#   Core     darf nicht UI/Render/App importieren
#   Shared   darf nicht crate::core direkt importieren (nur Crate-Root-Re-Exports)
#   XML      darf nicht App/UI/Render importieren
#   use_cases duerfen nicht tools-interne Submodule importieren
#   make check-layers   (wenn in Makefile eingebunden)

set -euo pipefail

VIOLATIONS=0

echo "=== Architektur-Check: Layer-Grenzen ==="

# Regel 1: UI darf nicht direkt auf Core zugreifen
UI_CORE_VIOLATIONS=$(grep -rn 'crate::core' src/ui/ --include='*.rs' 2>/dev/null || true)
if [ -n "$UI_CORE_VIOLATIONS" ]; then
    echo "FEHLER: UI importiert direkt aus core (muss ueber app re-exports gehen):"
    echo "$UI_CORE_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 2: Core darf nicht auf UI/Render/App zugreifen
CORE_UI_VIOLATIONS=$(grep -rn 'crate::ui\|crate::render\|crate::app' src/core/ --include='*.rs' 2>/dev/null || true)
if [ -n "$CORE_UI_VIOLATIONS" ]; then
    echo "FEHLER: Core importiert aus UI/Render/App:"
    echo "$CORE_UI_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 3: Render darf nicht auf UI/App zugreifen
RENDER_VIOLATIONS=$(grep -rn 'crate::ui\|crate::app' src/render/ --include='*.rs' 2>/dev/null || true)
if [ -n "$RENDER_VIOLATIONS" ]; then
    echo "FEHLER: Render importiert aus UI/App:"
    echo "$RENDER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 4: UI darf nicht auf XML/Render zugreifen
UI_OTHER_VIOLATIONS=$(grep -rn 'crate::xml\|crate::render' src/ui/ --include='*.rs' 2>/dev/null || true)
if [ -n "$UI_OTHER_VIOLATIONS" ]; then
    echo "FEHLER: UI importiert direkt aus XML/Render:"
    echo "$UI_OTHER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 5: UI darf keine vollstaendige mutable AppState-Referenz annehmen
UI_MUT_APPSTATE_VIOLATIONS=$(grep -rn '&mut[[:space:]]\+AppState' src/ui/ --include='*.rs' 2>/dev/null || true)
if [ -n "$UI_MUT_APPSTATE_VIOLATIONS" ]; then
    echo "FEHLER: UI nutzt &mut AppState (Intent/Command-Boundary verletzt):"
    echo "$UI_MUT_APPSTATE_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 6: UI darf keine direkten state.* Feldzuweisungen durchfuehren
UI_STATE_ASSIGN_VIOLATIONS=$(grep -rn '\bstate\.[A-Za-z0-9_\.]*[[:space:]]*=' src/ui/ --include='*.rs' 2>/dev/null | grep -v '// layer-ok' || true)
if [ -n "$UI_STATE_ASSIGN_VIOLATIONS" ]; then
    echo "FEHLER: UI enthaelt direkte state.* Zuweisungen (statt Intent/Command):"
    echo "$UI_STATE_ASSIGN_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 7: XML darf nicht auf App/UI/Render zugreifen
XML_UPPER_VIOLATIONS=$(grep -rn 'crate::app\|crate::ui\|crate::render' src/xml/ --include='*.rs' 2>/dev/null || true)
if [ -n "$XML_UPPER_VIOLATIONS" ]; then
    echo "FEHLER: XML importiert aus App/UI/Render (Schichtenverletzung nach oben):"
    echo "$XML_UPPER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 8: use_cases duerfen nicht aus tool-internen Submodulen importieren
# (Ausnahme: crate::app::tools direkt fuer ToolResult/oeffentliche Typen ist OK;
#  verboten ist der Import tool-interner Geometrie/Logic wie tools::spline::geometry)
USE_CASES_TOOLS_VIOLATIONS=$(grep -rn 'crate::app::tools::' src/app/use_cases/ --include='*.rs' 2>/dev/null \
    | grep -v 'crate::app::tools::ToolResult\|apply_tool_result' || true)
if [ -n "$USE_CASES_TOOLS_VIOLATIONS" ]; then
    echo "FEHLER: use_cases importiert aus tools-internen Submodulen (gemeinsame Logik muss in shared liegen):"
    echo "$USE_CASES_TOOLS_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 9: shared darf nicht direkt aus crate::core importieren
# (Core-Typen muessen ueber den Crate-Root eingebunden werden, z.B. crate::Camera2D)
SHARED_CORE_VIOLATIONS=$(grep -rn 'crate::core' src/shared/ --include='*.rs' 2>/dev/null || true)
if [ -n "$SHARED_CORE_VIOLATIONS" ]; then
    echo "FEHLER: shared importiert direkt aus crate::core (Crate-Root-Re-Exports verwenden):"
    echo "$SHARED_CORE_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

if [ "$VIOLATIONS" -eq 0 ]; then
    echo "✓ Alle Layer-Grenzen eingehalten."
    exit 0
else
    echo ""
    echo "✗ $VIOLATIONS Verletzung(en) gefunden."
    exit 1
fi
