# E6 evidence

| File | What |
|---|---|
| `nav-costs.json` | og-nav: grid A* and JPS, flow fields, bake and build of a navmesh, 200 agents a tick (`cargo run --release -p og-nav --example nav_costs`) |
| `ai-costs.json` | og-ai: 200 guards on a behaviour tree with sight against a wall, on og.nav (`cargo run --release -p og-ai --example ai_costs`) |
| `audio-costs.json` | og-audio: 32 voices through every bus insert and per-voice filter, ms of mixing per second of audio (`cargo run --release -p og-audio --example mix_costs`) |
| `i18n-costs.json` | og-ui: an Arabic label shaped and reordered, a Latin one passed through (`cargo run --release -p og-ui --example shape_costs`) |
| `wasm-size.json` | the publish wasm's brotli size and each E6 crate's share of its code |
| `determinism.json` | replays and audio captures that ran twice and matched |
| `audio/presets.json` | every sound preset rendered twice, SHA-256 (`scripts/sound-presets.sh`) |
| `audio/e6-mix.json`, `audio/waveforms.png` | the mixer's busy scene (PCM hash of two runs) and the waveforms of it and of topdown's `og run --audio` |
| `i18n/locale-sheet.png` | ui-kit's main menu, pause and confirm screens in en, zh-CN, ja, ko and ar (from `crates/og-ui/references/metal/kit-locale-*`) |
| `polish/<template>/` | `og polish` on the six templates: `report.md`, its frames, `polish.json` |
| `break-and-restore.md` | every new test broken once in the code it covers, and what failed |
