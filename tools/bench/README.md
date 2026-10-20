# Benchmarks

The measurement pages and scripts from the spike (`docs/spike-results.md`).

- `threejs-compare.ts` — the design §7 comparison of the engine itself with three.js 0.180 in headless Chrome for Testing:
  the §7 reference scene (`crates/og-tests/reference-scene` against `three-reference.js`) and a 20,000-sphere per-object
  field (`spheres/` against `three-spheres.js`), engine on WebGPU and WebGL2 (`engine.page.ts`), three on WebGL2
  (and WebGPU for the instanced spheres); `--shots` for screenshots. Results: `tests/evidence/P4/web-host-tools/threejs-comparison.md`.
- `compare.html` — the reference scenes in three.js 0.180 (instanced, `naive=1`, `sphere=1`, `moving=1`).
- `spike-index.html` + `spike-worker.js` — the engine side of the comparison. They loaded the
  spike's wasm (`WebSpike`); they are re-pointed at `og-host-web` once it ships (M2), until then
  they do not run.
- `serve.ts` — a static server with COOP/COEP and a `POST /result` sink (`bun serve.ts`).
- `run-chrome.sh <url> <seconds>` — headless Chrome for Testing with vsync off; pages POST their numbers.
- `views.ts` — the TypeScript typed-view system benchmark (`bun views.ts 10000`).
- `e2b-run.ts` — runs a Linux binary headless in the platform's E2B `probe` sandbox (billable).

Models for `compare.html` are the ones in `crates/og-tests/models/`; copy or symlink them to `tools/bench/models/`.
