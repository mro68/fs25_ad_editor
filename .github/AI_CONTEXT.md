# Kontext fuer AI-Tools

Dieses Projekt nutzt mehrere Dokumentations-Ebenen fuer AI-Assistenten:

1. **`.github/copilot-instructions.md`** → GitHub Copilot in VS Code
2. **`.windsurf/rules/*.md`** → Windsurf/Codeium und andere Tools
3. **`docs/*.md`** → Projektdokumentation

Alle enthalten konsistente Informationen zu:
- Architektur (egui + wgpu)
- App-Fluss (`AppIntent` -> `AppCommand` -> Use-Cases)
- Code-Konventionen (DE/EN Mix)
- AutoDrive XML-Format (Structure of Arrays)
- Performance-Ziele (100k Nodes)

## Source of Truth (Architektur)

- Primaer: `docs/ARCHITECTURE_PLAN.md`
- Ergaenzend: `src/app/API.md`, `src/ui/API.md`, `src/render/API.md`
