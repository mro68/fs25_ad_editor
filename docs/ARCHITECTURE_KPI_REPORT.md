# Architektur-KPI-Report

Stand: 2026-07-06 21:04:30Z  
Scope: Workspace ohne `crates/fs25_auto_drive_host_bridge_ffi`

## 1) API-Surface

- `crates/fs25_auto_drive_host_bridge/src/lib.rs`: **4** `pub use`-Zeilen
- `crates/fs25_auto_drive_engine/src/app/mod.rs`: **13** `pub use`-Zeilen
- `crates/fs25_auto_drive_host_bridge/src/dto/mod.rs`: **12** `pub use`-Zeilen

## 2) Re-Export-Kopplung

- Direkte Core-Re-Exports in `app/mod.rs`: **3**

## 3) Integrations-Komplexitaet (Dateigroesse)

- `editor_app/mod.rs`: **240** Zeilen
- `host_bridge/lib.rs`: **65** Zeilen

## 4) Interpretation (kurz)

- Sinkende `pub use`-Zahlen und kleinere Integrationsdateien deuten auf bessere Entkopplung hin.
- Ein stabiler oder sinkender Wert bei Core-Re-Exports reduziert unbeabsichtigte API-Ausweitung.
- Dieser Report ist bewusst leichtgewichtig und CI-freundlich.

