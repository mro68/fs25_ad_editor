#!/usr/bin/env bash
# CI-Check: Architektur-Guardrails fuer Schichtentrennung.
#
# Regeln (Importrichtungen):
#   UI       → App → Core (niemals crate::core direkt)
#   UI       darf nicht crate::xml oder crate::render importieren
#   Render   → Shared (niemals crate::app, crate::ui oder Core via Root-Re-Export)
#   Core     darf nicht UI/Render/App importieren
#   Shared   darf keine Core-Typen direkt oder ueber Crate-Root-Re-Exports importieren
#   XML      darf nicht App/UI/Render importieren
#   use_cases duerfen nicht tools-interne Submodule importieren
#   make check-layers   (wenn in Makefile eingebunden)

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VIOLATIONS=0
CORE_ROOT_EXPORTS='AutoDriveMeta|BackgroundMap|Camera2D|Connection|ConnectionDirection|ConnectionPriority|MapMarker|MapNode|NodeFlag|RoadMap|SpatialIndex|SpatialMatch|WorldBounds'
ROOT_CORE_PATTERN="crate::(${CORE_ROOT_EXPORTS})(::|[^[:alnum:]_]|$)"

echo "=== Architektur-Check: Layer-Grenzen ==="

find_forbidden_use_imports() {
    local scope_dir="$1"
    local module_segment="$2"

    # Prueft komplette Rust-use-Statements inklusive mehrzeiliger crate::{...}-Gruppierungen.
    # Top-Level-Eintraege werden bewusst separat ausgewertet, damit verschachtelte
    # Submodule wie shared::options::render keine False Positives ausloesen.
    while IFS= read -r -d '' file; do
        perl -0ne '
            use strict;
            use warnings;

            our $module;

            BEGIN {
                $module = shift @ARGV;
            }

            sub grouped_item_matches {
                my ($item, $module) = @_;
                $item =~ s/^\s+|\s+$//g;
                return 0 if $item eq q{};

                return $item =~ /^\Q$module\E(?:$|::|\s+as\b)/ ? 1 : 0;
            }

            sub imports_forbidden_top_level_module {
                my ($statement, $module) = @_;

                return 1
                    if $statement =~ /\buse\s+crate::\Q$module\E(?:\s+as\b|::|\s*;)/;

                return 0 unless $statement =~ /\buse\s+crate::\{(.*)\}\s*;/;

                my $grouped_items = $1;
                my $depth = 0;
                my $current_item = q{};

                for my $char (split //, $grouped_items) {
                    if ($char eq q{{}) {
                        $depth++;
                    } elsif ($char eq q{}}) {
                        $depth--;
                    }

                    if ($char eq q{,} && $depth == 0) {
                        return 1 if grouped_item_matches($current_item, $module);
                        $current_item = q{};
                        next;
                    }

                    $current_item .= $char;
                }

                return grouped_item_matches($current_item, $module);
            }

            my $file = $ARGV;
            my $content = $_;

            while ($content =~ /(^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?use\b[\s\S]*?;)/mg) {
                my $statement = $1;
                my $prefix = substr($content, 0, $-[1]);
                my $start_line = 1 + ($prefix =~ tr/\n/\n/);
                my $normalized = $statement;

                $normalized =~ s{//[^\n]*}{}g;
                $normalized =~ s{/\*.*?\*/}{}gs;
                $normalized =~ s/\s+/ /gs;
                $normalized =~ s/^\s+|\s+$//g;

                if (imports_forbidden_top_level_module($normalized, $module)) {
                    print "$file:$start_line:$normalized\n";
                }
            }
        ' "$module_segment" "$file"
    done < <(rg --files -0 --glob '*.rs' "$scope_dir" 2>/dev/null || true)
}

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

# Regel 2a: App darf nicht auf UI zugreifen
# Es werden nur explizite Rust-Importe geprueft, damit Kommentare und Dokumentation
# keine False Positives erzeugen.
APP_UI_VIOLATIONS=$(find_forbidden_use_imports src/app ui)
if [ -n "$APP_UI_VIOLATIONS" ]; then
    echo "FEHLER: App importiert aus UI (Application darf Presentation nicht kennen):"
    echo "$APP_UI_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 2b: App darf nicht auf Render zugreifen
# Gleiches Vorgehen wie bei UI: nur explizite use-Importe statt Freitext-Matches.
APP_RENDER_VIOLATIONS=$(find_forbidden_use_imports src/app render)
if [ -n "$APP_RENDER_VIOLATIONS" ]; then
    echo "FEHLER: App importiert aus Render (Application darf Rendering nicht kennen):"
    echo "$APP_RENDER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 3: Render darf nicht auf UI/App/Core zugreifen
# Der Check prueft explizit auch Root-Re-Exports wie `crate::RoadMap`, damit
# Layer-Verletzungen nicht mehr durch das Crate-Root maskiert werden.
RENDER_VIOLATIONS=$(grep -rnE "crate::ui|crate::app|crate::core|${ROOT_CORE_PATTERN}" src/render/ --include='*.rs' 2>/dev/null || true)
if [ -n "$RENDER_VIOLATIONS" ]; then
    echo "FEHLER: Render importiert aus UI/App/Core oder ueber Root-Re-Exports aus Core:"
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

# Regel 9: shared darf keine Core-Typen importieren — weder direkt noch ueber Root-Re-Exports
SHARED_CORE_VIOLATIONS=$(grep -rnE "crate::core|${ROOT_CORE_PATTERN}" src/shared/ --include='*.rs' 2>/dev/null || true)
if [ -n "$SHARED_CORE_VIOLATIONS" ]; then
    echo "FEHLER: shared importiert aus Core oder ueber Root-Re-Exports aus Core:"
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
