# Architektur-Ueberblick

## Datenfluss

```mermaid
flowchart LR
  subgraph UI[ui]
    MENU[Menu]
    STATUS[StatusBar]
    INPUT[Mouse/Keyboard]
  end

  subgraph APP[app]
    INTENT[AppIntent]
    CMD[AppCommand]
    CTRL[AppController]
    STATE[AppState]
    UC[UseCases]
    SCENE[RenderScene Builder]
  end

  subgraph CORE[core]
    ROAD[RoadMap]
    INDEX[Spatial Index - kiddo]
  end

  subgraph XML[xml]
    IO[Parser/Writer]
  end

  subgraph GFX[render]
    RENDERER[Renderer]
  end

  MENU --> INTENT
  INPUT --> INTENT
  INTENT --> CTRL
  CTRL --> CMD
  CMD --> UC
  UC --> STATE

  UC -->|mutate/query| ROAD
  ROAD -->|nodes/edges| CTRL
  ROAD --> INDEX

  CTRL -->|build scene| SCENE
  SCENE --> RENDERER

  IO --> ROAD
  ROAD --> IO

  STATUS -. read-only .-> STATE
```

## Grundsaetze

- Die UI sendet nur `AppIntent`, mutiert aber keine Core-Daten direkt.
- Die App mappt Intents auf Commands und fuehrt Mutationen zentral ueber Use-Cases aus.
- Der Core enthaelt Datenmodell, XML-IO und Spatial-Queries.
- Das Rendering bekommt nur vorbereitete Render-Daten.
