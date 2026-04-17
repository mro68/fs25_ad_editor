#!/usr/bin/env bash
# Report-only Guardrail: listet dateien ueber einer Zeilen-Schwelle.
#
# Default-Schwelle: 800 Zeilen
# Optionaler Parameter 1: individuelle Schwelle
#
# Wichtig: Dieses Script ist bewusst report-only und beendet immer mit Exit-Code 0,
# sofern keine Laufzeitfehler im Script selbst auftreten.

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

DEFAULT_THRESHOLD=800
LINE_THRESHOLD="${1:-$DEFAULT_THRESHOLD}"

if ! [[ "$LINE_THRESHOLD" =~ ^[0-9]+$ ]]; then
  echo "[WARN] Ungueltige Schwelle '$LINE_THRESHOLD' erkannt; verwende $DEFAULT_THRESHOLD."
  LINE_THRESHOLD="$DEFAULT_THRESHOLD"
fi

echo "=== Guardrail-Report: Dateien ueber ${LINE_THRESHOLD} Zeilen ==="
echo "Hinweis: report-only, kein Fail-Gate."

collect_files() {
  if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git ls-files -z
    return
  fi

  echo "[WARN] Git-Dateiliste nicht verfuegbar; nutze find-Fallback." >&2
  find . \
    -type f \
    -not -path './.git/*' \
    -not -path './target/*' \
    -not -path './binaries/*' \
    -print0
}

TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

MATCH_COUNT=0
while IFS= read -r -d '' file_path; do
  [ -f "$file_path" ] || continue

  line_count="$(wc -l < "$file_path")"
  if [ "$line_count" -gt "$LINE_THRESHOLD" ]; then
    printf "%09d\t%s\n" "$line_count" "$file_path" >> "$TMP_REPORT"
    MATCH_COUNT=$((MATCH_COUNT + 1))
  fi
done < <(collect_files)

if [ "$MATCH_COUNT" -eq 0 ]; then
  echo "Keine Dateien ueber ${LINE_THRESHOLD} Zeilen gefunden."
  echo "Ergebnis: 0 Treffer"
  exit 0
fi

echo "Treffer: ${MATCH_COUNT} Datei(en)"
printf "%-8s | %s\n" "Zeilen" "Datei"
printf -- "---------+---------------------------------------------------------------\n"

sort -r "$TMP_REPORT" | while IFS=$'\t' read -r raw_count file_path; do
  printf "%8d | %s\n" "$((10#$raw_count))" "$file_path"
done

echo "Ergebnis: ${MATCH_COUNT} Treffer"
