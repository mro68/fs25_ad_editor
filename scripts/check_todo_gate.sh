#!/usr/bin/env bash
set -euo pipefail

# Blockiert todo!() in Domain-/Host-Bridge-Produktionscode (nur non-test via --lib).
cargo clippy --no-deps \
  -p fs25_auto_drive_engine \
  -p fs25_auto_drive_host_bridge \
  -p fs25_auto_drive_host_bridge_ffi \
  --lib \
  -- \
  -D clippy::todo
