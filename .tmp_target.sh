#!/bin/bash

# 1. Nur den Namen des aktuellen Ordners extrahieren (z.B. "mein-projekt")
PROJECT_NAME=$(basename "$PWD")
TMP_TARGET="/tmp/${PROJECT_NAME}_target"

# Sicherheits-Check: Sind wir im richtigen Projekt-Verzeichnis?
if [ ! -f "Cargo.toml" ] && [ ! -f "pubspec.yaml" ]; then
    echo "❌ Fehler: Hier ist kein Rust- oder Flutter-Projekt!"
    exit 1
fi

echo "🔧 Verlinke target für Projekt: $PROJECT_NAME"

# 2. Altes target-Verzeichnis oder alten Link entfernen
if [ -L "target" ]; then
    rm "target"
elif [ -d "target" ]; then
    echo "🗑️  Lösche lokales target-Verzeichnis..."
    rm -rf "target"
fi

# 3. Struktur in /tmp vorbereiten
# -p erstellt auch den Projekt-Unterordner, falls /tmp sauber ist
mkdir -p "$TMP_TARGET"

# 4. Den Symlink erstellen
ln -s "$TMP_TARGET" "target"

echo "🚀 Fertig! './target' -> '$TMP_TARGET'"