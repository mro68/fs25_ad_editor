#!/usr/bin/env bash
# CI-Check: Architektur-Guardrails fuer die Workspace-Crates.
#
# Regeln (Crate-/Layer-Richtungen):
#   fs25_auto_drive_host_bridge             → fs25_auto_drive_engine
#   fs25_auto_drive_frontend_egui              → fs25_auto_drive_engine + fs25_auto_drive_host_bridge + fs25_auto_drive_render_wgpu
#   fs25_auto_drive_frontend_flutter_bridge    → fs25_auto_drive_host_bridge
#   fs25_auto_drive_render_wgpu                → fs25_auto_drive_engine::shared
#   fs25_auto_drive_engine kennt keine Frontend-Crates
#   render bleibt innerhalb der egui-Frontend-Crate ein Host-Adapter
#   ui bleibt innerhalb der egui-Frontend-Crate auf app/shared beschraenkt
#   use_cases duerfen nicht tools-interne Submodule importieren

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VIOLATIONS=0
ENGINE_DIR="crates/fs25_auto_drive_engine/src"
HOST_BRIDGE_DIR="crates/fs25_auto_drive_host_bridge/src"
EGUI_DIR="crates/fs25_auto_drive_frontend_egui/src"
BRIDGE_DIR="crates/fs25_auto_drive_frontend_flutter_bridge/src"
RENDER_WGPU_DIR="crates/fs25_auto_drive_render_wgpu/src"

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

# Regel 1: Engine-App darf keine Frontend- oder Render-Host-Module kennen
APP_FRONTEND_VIOLATIONS=$(grep -rnE 'crate::ui|crate::render|crate::editor_app|fs25_auto_drive_frontend_egui|fs25_auto_drive_frontend_flutter_bridge|fs25_auto_drive_host_bridge|fs25_auto_drive_render_wgpu' "$ENGINE_DIR/app" --include='*.rs' 2>/dev/null || true)
if [ -n "$APP_FRONTEND_VIOLATIONS" ]; then
    echo "FEHLER: Engine-App importiert aus Frontend-Modulen oder Frontend-Crates:"
    echo "$APP_FRONTEND_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 2: Engine-Core darf nicht nach oben, in Frontends oder in den Render-Host greifen
CORE_UPPER_VIOLATIONS=$(grep -rnE 'crate::app|crate::ui|crate::render|crate::editor_app|fs25_auto_drive_frontend_egui|fs25_auto_drive_frontend_flutter_bridge|fs25_auto_drive_host_bridge|fs25_auto_drive_render_wgpu' "$ENGINE_DIR/core" --include='*.rs' 2>/dev/null || true)
if [ -n "$CORE_UPPER_VIOLATIONS" ]; then
    echo "FEHLER: Engine-Core importiert aus hoeheren Layern oder Frontend-Crates:"
    echo "$CORE_UPPER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 3: Engine-XML darf nicht nach oben, in Frontends oder in den Render-Host greifen
XML_UPPER_VIOLATIONS=$(grep -rnE 'crate::app|crate::ui|crate::render|crate::editor_app|fs25_auto_drive_frontend_egui|fs25_auto_drive_frontend_flutter_bridge|fs25_auto_drive_host_bridge|fs25_auto_drive_render_wgpu' "$ENGINE_DIR/xml" --include='*.rs' 2>/dev/null || true)
if [ -n "$XML_UPPER_VIOLATIONS" ]; then
    echo "FEHLER: Engine-XML importiert aus hoeheren Layern oder Frontend-Crates:"
    echo "$XML_UPPER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 4: Engine-Shared darf keine Core-Typen direkt importieren
SHARED_CORE_VIOLATIONS=$(grep -rn 'crate::core' "$ENGINE_DIR/shared" --include='*.rs' 2>/dev/null || true)
if [ -n "$SHARED_CORE_VIOLATIONS" ]; then
    echo "FEHLER: Engine-Shared importiert aus Core:"
    echo "$SHARED_CORE_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 5: Engine-use_cases duerfen nicht aus tools-internen Submodulen importieren
USE_CASES_TOOLS_VIOLATIONS=$(grep -rn 'crate::app::tools::' "$ENGINE_DIR/app/use_cases" --include='*.rs' 2>/dev/null \
    | grep -v 'crate::app::tools::ToolResult\|apply_tool_result' || true)
if [ -n "$USE_CASES_TOOLS_VIOLATIONS" ]; then
    echo "FEHLER: Engine-use_cases importieren aus tools-internen Submodulen:"
    echo "$USE_CASES_TOOLS_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 6: Egui-Render darf nicht auf UI/App/Core/XML oder Root-Fassade zugreifen
RENDER_VIOLATIONS=$(grep -rnE 'crate::ui|crate::editor_app|crate::app|crate::core|crate::xml|fs25_auto_drive_editor::' "$EGUI_DIR/render" --include='*.rs' 2>/dev/null || true)
if [ -n "$RENDER_VIOLATIONS" ]; then
    echo "FEHLER: Egui-Render importiert aus verbotenen Layern oder der Root-Fassade:"
    echo "$RENDER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 7: Egui-UI darf nicht direkt auf XML, Render oder Core zugreifen
UI_OTHER_VIOLATIONS=$(grep -rnE 'crate::xml|crate::render|crate::core|fs25_auto_drive_editor::' "$EGUI_DIR/ui" --include='*.rs' 2>/dev/null || true)
if [ -n "$UI_OTHER_VIOLATIONS" ]; then
    echo "FEHLER: Egui-UI importiert direkt aus XML/Render/Core oder der Root-Fassade:"
    echo "$UI_OTHER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 8: Egui-UI darf keine vollstaendige mutable AppState-Referenz annehmen
UI_MUT_APPSTATE_VIOLATIONS=$(grep -rn '&mut[[:space:]]\+AppState' "$EGUI_DIR/ui" --include='*.rs' 2>/dev/null || true)
if [ -n "$UI_MUT_APPSTATE_VIOLATIONS" ]; then
    echo "FEHLER: Egui-UI nutzt &mut AppState (Intent/Command-Boundary verletzt):"
    echo "$UI_MUT_APPSTATE_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 9: Egui-UI darf keine direkten state.* Feldzuweisungen durchfuehren
UI_STATE_ASSIGN_VIOLATIONS=$(grep -rn '\bstate\.[A-Za-z0-9_\.]*[[:space:]]*=' "$EGUI_DIR/ui" --include='*.rs' 2>/dev/null | grep -v '// layer-ok' || true)
if [ -n "$UI_STATE_ASSIGN_VIOLATIONS" ]; then
    echo "FEHLER: Egui-UI enthaelt direkte state.* Zuweisungen (statt Intent/Command):"
    echo "$UI_STATE_ASSIGN_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 10: Flutter-Bridge darf keine Frontend-Crates, UI-Toolkits oder direkte Engine-Imports kennen
BRIDGE_FRONTEND_VIOLATIONS=$(grep -rnE 'fs25_auto_drive_frontend_egui|fs25_auto_drive_engine::|crate::ui|crate::render|crate::editor_app|egui::|eframe::' "$BRIDGE_DIR" --include='*.rs' 2>/dev/null || true)
if [ -n "$BRIDGE_FRONTEND_VIOLATIONS" ]; then
    echo "FEHLER: Flutter-Bridge importiert Frontend-Module, direkte Engine-Symbolpfade oder UI-Toolkits:"
    echo "$BRIDGE_FRONTEND_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 11: Host-Bridge-Core darf keine Frontend-Crates, Render-Core oder UI-Toolkits kennen
HOST_BRIDGE_FRONTEND_VIOLATIONS=$(grep -rnE 'fs25_auto_drive_frontend_egui|fs25_auto_drive_frontend_flutter_bridge|fs25_auto_drive_render_wgpu|crate::ui|crate::render|crate::editor_app|egui::|eframe::|egui_wgpu::|wgpu::' "$HOST_BRIDGE_DIR" --include='*.rs' 2>/dev/null || true)
if [ -n "$HOST_BRIDGE_FRONTEND_VIOLATIONS" ]; then
    echo "FEHLER: Host-Bridge-Core importiert Host-/Render-spezifische Layer oder Toolkits:"
    echo "$HOST_BRIDGE_FRONTEND_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 12: Render-wgpu-Core darf keine App/Core/XML- oder Host-UI-Abhaengigkeiten haben
RENDER_WGPU_LAYER_VIOLATIONS=$(grep -rnE 'fs25_auto_drive_engine::app|fs25_auto_drive_engine::core|fs25_auto_drive_engine::xml|fs25_auto_drive_frontend_egui|fs25_auto_drive_frontend_flutter_bridge|fs25_auto_drive_host_bridge|fs25_auto_drive_editor::|egui::|eframe::|egui_wgpu::|crate::app|crate::core|crate::xml' "$RENDER_WGPU_DIR" --include='*.rs' 2>/dev/null || true)
if [ -n "$RENDER_WGPU_LAYER_VIOLATIONS" ]; then
    echo "FEHLER: Render-wgpu-Core importiert verbotene Layer oder Host-Toolkits:"
    echo "$RENDER_WGPU_LAYER_VIOLATIONS"
    VIOLATIONS=$((VIOLATIONS + 1))
fi

# Regel 13: Frontend-Crates duerfen nicht gegenseitig oder ueber Root importieren
EGUI_ROOT_VIOLATIONS=$(grep -rn 'fs25_auto_drive_editor::' "$EGUI_DIR" --include='*.rs' 2>/dev/null || true)
if [ -n "$EGUI_ROOT_VIOLATIONS" ]; then
    echo "FEHLER: Egui-Frontend importiert ueber die Root-Fassade statt direkt ueber lokale Re-Exports:"
    echo "$EGUI_ROOT_VIOLATIONS"
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
