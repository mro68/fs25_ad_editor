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

# Entfernte Bypass-Use-Case-Doku darf nicht mehr auftauchen.
check_no_match 'generate_bypass\(state\)|state\.ui\.bypass' 'src/app/use_cases/API.md' 'Bypass laeuft ueber Route-Tool-Flow, nicht ueber eigenen Use-Case'

echo "✓ API-Doku-Sync-Check erfolgreich"
