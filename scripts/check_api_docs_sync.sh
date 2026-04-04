#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

API_DOC_FILES=()
while IFS= read -r -d '' file; do
  API_DOC_FILES+=("$file")
done < <(find src crates -type f -name API.md -print0)

APP_API="crates/fs25_auto_drive_engine/src/app/API.md"
APP_TOOLS_API="crates/fs25_auto_drive_engine/src/app/tools/API.md"
APP_USE_CASES_API="crates/fs25_auto_drive_engine/src/app/use_cases/API.md"
UI_API="crates/fs25_auto_drive_frontend_egui/src/ui/API.md"

search_regex() {
  local pattern="$1"
  shift

  if command -v rg >/dev/null 2>&1; then
    rg -n --no-heading "$pattern" "$@"
  else
    grep -n -E -- "$pattern" "$@"
  fi
}

search_literal() {
  local needle="$1"
  shift

  if command -v rg >/dev/null 2>&1; then
    rg -n --no-heading -F "$needle" "$@"
  else
    grep -n -F -- "$needle" "$@"
  fi
}

check_no_match() {
  local pattern="$1"
  local hint="$2"
  shift 2

  local tmp_file
  local search_status
  tmp_file="$(mktemp)"

  set +e
  search_regex "$pattern" "$@" >"$tmp_file"
  search_status=$?
  set -e

  if [ "$search_status" -eq 0 ]; then
    echo "[FAIL] Veraltete API-Dokumentation gefunden: $hint"
    cat "$tmp_file"
    rm -f "$tmp_file"
    exit 1
  fi

  if [ "$search_status" -ne 1 ]; then
    rm -f "$tmp_file"
    echo "[FAIL] API-Doku-Suche fehlgeschlagen: $hint"
    exit 1
  fi

  rm -f "$tmp_file"
}

check_has_literal() {
  local needle="$1"
  local hint="$2"
  shift 2

  local tmp_file
  local search_status
  tmp_file="$(mktemp)"

  set +e
  search_literal "$needle" "$@" >"$tmp_file"
  search_status=$?
  set -e

  if [ "$search_status" -eq 1 ]; then
    echo "[FAIL] Erwartete Doku-Aussage fehlt: $hint"
    rm -f "$tmp_file"
    exit 1
  fi

  if [ "$search_status" -ne 0 ]; then
    rm -f "$tmp_file"
    echo "[FAIL] API-Doku-Suche fehlgeschlagen: $hint"
    exit 1
  fi

  rm -f "$tmp_file"
}

echo "=== API-Doku-Sync-Check ==="

if [ "${#API_DOC_FILES[@]}" -eq 0 ]; then
  echo "[FAIL] Keine API.md-Dateien unter src/ oder crates/ gefunden"
  exit 1
fi

# Nach IndexSet-Migration darf in API.md kein Arc<HashSet<u64>> mehr stehen.
check_no_match 'Arc<HashSet<u64>>|&HashSet<u64>' 'IndexSet statt HashSet in API.md verwenden' "${API_DOC_FILES[@]}"

# Alte RouteTool-Dokuvertraege aus frueheren Versionen.
check_no_match 'render_context_menu\(response\)' 'RouteTool nutzt TangentMenuData statt render_context_menu(response)' "$APP_TOOLS_API"

# RouteTool-Doku muss den gesplitteten Basisvertrag plus Capabilities beschreiben.
check_has_literal 'RouteToolCore' 'RouteTool-Doku beschreibt den festen Kernvertrag' "$APP_TOOLS_API"
check_has_literal 'RouteToolPanelBridge' 'RouteTool-Doku beschreibt die Panel-Bruecke' "$APP_TOOLS_API"
check_has_literal 'RouteToolHostSync' 'RouteTool-Doku beschreibt den Host-Sync-Vertrag' "$APP_TOOLS_API"
check_has_literal 'RouteToolDrag' 'RouteTool-Doku dokumentiert Drag als Capability' "$APP_TOOLS_API"
check_has_literal 'RouteToolChainInput' 'RouteTool-Doku dokumentiert Chain-Input als Capability' "$APP_TOOLS_API"

# Index-basierte RouteTool-Doku wurde durch stabile RouteToolId-Contracts ersetzt.
check_no_match 'SelectRouteToolRequested \{ index: usize \}|RouteToolWithAnchorsRequested \{ index: usize \}|SelectRouteTool \{ index: usize \}|RouteToolWithAnchors \{ index: usize \}|last_basic_command_index|last_smooth_curve_index|last_section_tool_index' 'App-API dokumentiert RouteToolId und RouteToolSelectionMemory statt alter Index-Felder' "$APP_API"

# Floating-Menues und Icons sind gruppen-/id-basiert statt slot-basiert.
check_no_match 'FloatingMenuKind::Basics|FloatingMenuKind::SectionTools|route_tool_icon\(idx: usize\)' 'Floating-Menues und Route-Icons dokumentieren den gruppen-/id-basierten Vertrag' "$UI_API" "$APP_API" docs/ARCHITECTURE_PLAN.md

# RouteOffsetTool fuehrt keinen ungenutzten Preview-Cache mehr.
check_no_match 'cached_preview' 'RouteOffsetTool-Doku enthaelt keinen toten Preview-Cache' "$APP_TOOLS_API"

# Entfernte Bypass-Use-Case-Doku darf nicht mehr auftauchen.
check_no_match 'generate_bypass\(state\)|state\.ui\.bypass' 'Bypass laeuft ueber Route-Tool-Flow, nicht ueber eigenen Use-Case' "$APP_USE_CASES_API"

# docs/How-To-Use.md bleibt bewusst ein Redirect-Stub fuer alte Links.
check_has_literal 'Diese Datei ist nur noch ein Einstiegspunkt fuer alte Links auf die fruehere Sammel-Anleitung.' 'docs/How-To-Use.md bleibt ein Redirect-Stub fuer alte Links' docs/How-To-Use.md
check_has_literal 'Die gepflegte, aktuelle User-Dokumentation liegt unter [docs/howto/index.md](howto/index.md).' 'docs/How-To-Use.md verweist auf die aktuelle How-To-Sammlung' docs/How-To-Use.md
check_has_literal 'Die fruehere Ein-Datei-Anleitung wird nicht mehr inhaltlich gepflegt.' 'docs/How-To-Use.md darf nicht wieder als gepflegte Vollanleitung missverstanden werden' docs/How-To-Use.md

# UI-API muss den Dissolve-Dialogzustand und den bestaetigten Flow dokumentieren.
check_has_literal 'UiState::confirm_dissolve_group_id' 'src/ui/API.md dokumentiert den Group-Dissolve-Zustand' "$UI_API"
check_has_literal 'show_confirm_dissolve_dialog' 'src/ui/API.md dokumentiert den Dissolve-Dialog' "$UI_API"
check_has_literal 'AppIntent::DissolveGroupConfirmed { segment_id }' 'src/ui/API.md dokumentiert den bestaetigten Dissolve-Intent' "$UI_API"

# Der Tool-Encapsulation-Report bleibt ein historisches Audit und keine Vertragsquelle.
check_has_literal '**Status:** Historisches Audit-Dokument' 'docs/TOOL_ENCAPSULATION_REPORT.md bleibt als historisches Audit markiert' docs/TOOL_ENCAPSULATION_REPORT.md
check_has_literal 'Er ist **nicht** die kanonische Beschreibung des aktuellen Tool-Katalogs oder der aktuell gueltigen Tool-Vertraege.' 'docs/TOOL_ENCAPSULATION_REPORT.md darf nicht als aktuelle Vertragsquelle missverstanden werden' docs/TOOL_ENCAPSULATION_REPORT.md
check_has_literal 'aktuellen API-Dokumente und Modul-Docstrings' 'docs/TOOL_ENCAPSULATION_REPORT.md verweist auf die aktuellen Vertragsquellen' docs/TOOL_ENCAPSULATION_REPORT.md

echo "✓ API-Doku-Sync-Check erfolgreich"
