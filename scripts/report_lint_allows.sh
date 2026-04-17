#!/usr/bin/env bash
# Report-only Governance-Script: inventarisiert Rust-Lint-Ausnahmen ueber allow(...)-Attribute.
#
# Erfasst:
# - #[allow(...)]
# - #![allow(...)]
# - #[cfg_attr(..., allow(...))]
#
# Ausgabe: Tabelle auf STDOUT (Datei, Zeile, Attribut)
# Verhalten: report-only, Exit 0 bei 0 Treffern

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== Governance-Report: Rust allow(...) Ausnahmen ==="
echo "Quelle: versionierte *.rs Dateien, ohne target/.git"
echo "Hinweis: report-only, kein Fail-Gate."

tmp_report="$(mktemp)"
trap 'rm -f "$tmp_report"' EXIT

rg -n --no-heading 'allow\(' --glob '!target/**' --glob '!.git/**' --glob '**/*.rs' \
  | grep -E '^[^:]+:[0-9]+:[[:space:]]*#\[' \
  | sort -t: -k1,1 -k2,2n > "$tmp_report"

match_count="$(wc -l < "$tmp_report" | tr -d ' ')"
if [ "$match_count" = "0" ]; then
  echo "Keine allow(...)-Attribute gefunden."
  echo "Ergebnis: 0 Treffer"
  exit 0
fi

echo "Treffer: ${match_count} Attribut-Zeile(n)"
printf "%-4s | %-90s | %-6s | %s\n" "Nr." "Datei" "Zeile" "Attribut"
printf -- "-----+--------------------------------------------------------------------------------------------+--------+-----------------------------------------------\n"

index=1
while IFS= read -r row; do
  file_path="${row%%:*}"
  rest="${row#*:}"
  line_no="${rest%%:*}"
  attribute="${rest#*:}"
  printf "%-4d | %-90s | %-6s | %s\n" "$index" "$file_path" "$line_no" "${attribute# }"
  index=$((index + 1))
done < "$tmp_report"

echo "Ergebnis: ${match_count} Treffer"
