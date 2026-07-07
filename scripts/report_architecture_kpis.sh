#!/usr/bin/env bash
# Architektur-KPI-Report fuer das Workspace (ohne FFI-Crate).
#
# Liefert einfache, robuste Kennzahlen fuer Wartbarkeit:
# - Anzahl pub use pro Crate-Wurzel
# - Zeilenumfang zentraler Integrationsdateien
# - Anzahl direkte crate::core Re-Exports im app/mod.rs
# - Anzahl Host-Bridge DTO-Top-Level-Re-Exports
#
# Ausgabe:
# - stdout (menschenlesbar)
# - docs/ARCHITECTURE_KPI_REPORT.md (Markdown-Report)

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

REPORT_PATH="docs/ARCHITECTURE_KPI_REPORT.md"

count_pattern() {
    local file="$1"
    local pattern="$2"
    grep -E -c "$pattern" "$file" 2>/dev/null || echo "0"
}

line_count() {
    local file="$1"
    wc -l < "$file" 2>/dev/null || echo "0"
}

HOST_BRIDGE_LIB="crates/fs25_auto_drive_host_bridge/src/lib.rs"
ENGINE_APP_MOD="crates/fs25_auto_drive_engine/src/app/mod.rs"
EDITOR_APP_MOD="crates/fs25_auto_drive_frontend_egui/src/editor_app/mod.rs"
DTO_MOD="crates/fs25_auto_drive_host_bridge/src/dto/mod.rs"

HOST_BRIDGE_PUB_USE_COUNT="$(count_pattern "$HOST_BRIDGE_LIB" '^[[:space:]]*pub use ')"
ENGINE_APP_PUB_USE_COUNT="$(count_pattern "$ENGINE_APP_MOD" '^[[:space:]]*pub use ')"
DTO_PUB_USE_COUNT="$(count_pattern "$DTO_MOD" '^[[:space:]]*pub use ')"

ENGINE_APP_CORE_REEXPORTS="$(count_pattern "$ENGINE_APP_MOD" '^[[:space:]]*pub use crate::core')"

EDITOR_APP_LINES="$(line_count "$EDITOR_APP_MOD")"
HOST_BRIDGE_LINES="$(line_count "$HOST_BRIDGE_LIB")"

cat > "$REPORT_PATH" <<EOF
# Architektur-KPI-Report

Stand: $(date -u +"%Y-%m-%d %H:%M:%SZ")  
Scope: Workspace ohne \`crates/fs25_auto_drive_host_bridge_ffi\`

## 1) API-Surface

- \`crates/fs25_auto_drive_host_bridge/src/lib.rs\`: **$HOST_BRIDGE_PUB_USE_COUNT** \`pub use\`-Zeilen
- \`crates/fs25_auto_drive_engine/src/app/mod.rs\`: **$ENGINE_APP_PUB_USE_COUNT** \`pub use\`-Zeilen
- \`crates/fs25_auto_drive_host_bridge/src/dto/mod.rs\`: **$DTO_PUB_USE_COUNT** \`pub use\`-Zeilen

## 2) Re-Export-Kopplung

- Direkte Core-Re-Exports in \`app/mod.rs\`: **$ENGINE_APP_CORE_REEXPORTS**

## 3) Integrations-Komplexitaet (Dateigroesse)

- \`editor_app/mod.rs\`: **$EDITOR_APP_LINES** Zeilen
- \`host_bridge/lib.rs\`: **$HOST_BRIDGE_LINES** Zeilen

## 4) Interpretation (kurz)

- Sinkende \`pub use\`-Zahlen und kleinere Integrationsdateien deuten auf bessere Entkopplung hin.
- Ein stabiler oder sinkender Wert bei Core-Re-Exports reduziert unbeabsichtigte API-Ausweitung.
- Dieser Report ist bewusst leichtgewichtig und CI-freundlich.

EOF

echo "=== Architektur-KPIs ==="
echo "Host-Bridge pub use: $HOST_BRIDGE_PUB_USE_COUNT"
echo "Engine app/mod.rs pub use: $ENGINE_APP_PUB_USE_COUNT"
echo "DTO mod.rs pub use: $DTO_PUB_USE_COUNT"
echo "Engine app/mod.rs core re-exports: $ENGINE_APP_CORE_REEXPORTS"
echo "editor_app/mod.rs lines: $EDITOR_APP_LINES"
echo "host_bridge/lib.rs lines: $HOST_BRIDGE_LINES"
echo "Report geschrieben: $REPORT_PATH"
