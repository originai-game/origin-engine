# Benchmarks

The measurement pages and scripts from the spike (`docs/spike-results.md`).

- `compare.html` — the reference scenes in three.js 0.180 (instanced, `naive=1`, `sphere=1`, `moving=1`).
- `spike-index.html` + `spike-worker.js` — the engine side of the comparison. They loaded the
  spike's wasm (`WebSpike`); they are re-pointed at `og-host-web` once it ships (M2), until then
  they do not run.
- `serve.ts` — a static server with COOP/COEP and a `POST /result` sink (`bun serve.ts`).
- `run-chrome.sh <url> <seconds>` — headless Chrome for Testing with vsync off; pages POST their numbers.
- `views.ts` — the TypeScript typed-view system benchmark (`bun views.ts 10000`).
- `e2b-run.ts` — runs a Linux binary headless in the platform's E2B `probe` sandbox (billable).

Models for `compare.html` are the ones in `crates/og-tests/models/`; copy or symlink them to `tools/bench/models/`.
