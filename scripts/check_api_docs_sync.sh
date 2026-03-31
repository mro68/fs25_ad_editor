#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

check_no_match() {
  local pattern="$1"
  local scope="$2"
  local hint="$3"

  if rg -n --no-heading "$pattern" $scope >/tmp/api_doc_sync_matches.txt; then
    echo "[FAIL] Veraltete API-Dokumentation gefunden: $hint"
    cat /tmp/api_doc_sync_matches.txt
    rm -f /tmp/api_doc_sync_matches.txt
    exit 1
  fi
  rm -f /tmp/api_doc_sync_matches.txt
}

echo "=== API-Doku-Sync-Check ==="

# Nach IndexSet-Migration darf in API.md kein Arc<HashSet<u64>> mehr stehen.
check_no_match 'Arc<HashSet<u64>>|&HashSet<u64>' 'src/**/API.md src/app/API.md src/ui/API.md' 'IndexSet statt HashSet in API.md verwenden'

# Alte RouteTool-Dokuvertraege aus frueheren Versionen.
check_no_match 'render_context_menu\(response\)' 'src/app/tools/API.md' 'RouteTool nutzt TangentMenuData statt render_context_menu(response)'

# RouteTool-Doku muss den direkten Default-Methoden-Vertrag beschreiben.
check_no_match 'RouteToolDrag|RouteToolTangent|RouteToolRegistry|RouteToolChainInput' 'src/app/tools/API.md' 'RouteTool-Doku nutzt direkte Hooks statt alter Capability-Traits'

# Index-basierte RouteTool-Doku wurde durch stabile RouteToolId-Contracts ersetzt.
check_no_match 'SelectRouteToolRequested \{ index: usize \}|RouteToolWithAnchorsRequested \{ index: usize \}|SelectRouteTool \{ index: usize \}|RouteToolWithAnchors \{ index: usize \}|last_basic_command_index|last_smooth_curve_index|last_section_tool_index' 'src/app/API.md' 'App-API dokumentiert RouteToolId und RouteToolSelectionMemory statt alter Index-Felder'

# Floating-Menues und Icons sind gruppen-/id-basiert statt slot-basiert.
check_no_match 'FloatingMenuKind::Basics|FloatingMenuKind::SectionTools|route_tool_icon\(idx: usize\)' 'src/ui/API.md src/app/API.md docs/ARCHITECTURE_PLAN.md' 'Floating-Menues und Route-Icons dokumentieren den gruppen-/id-basierten Vertrag'

# RouteOffsetTool fuehrt keinen ungenutzten Preview-Cache mehr.
check_no_match 'cached_preview' 'src/app/tools/API.md' 'RouteOffsetTool-Doku enthaelt keinen toten Preview-Cache'

# Entfernte Bypass-Use-Case-Doku darf nicht mehr auftauchen.
check_no_match 'generate_bypass\(state\)|state\.ui\.bypass' 'src/app/use_cases/API.md' 'Bypass laeuft ueber Route-Tool-Flow, nicht ueber eigenen Use-Case'

echo "✓ API-Doku-Sync-Check erfolgreich"
