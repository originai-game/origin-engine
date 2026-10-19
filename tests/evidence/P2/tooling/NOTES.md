# Phase 2 tooling round (owed rows 22, 34, 35)

Apple M4 Pro, macOS, Chrome for Testing 150 (headless, through `packages/engine/tests/web/cdp.ts`), 2026-10-19.

- `web-gfx-webgpu.json`: og-tests' 48 gfx scenes in Chrome on WebGPU, each against its tier's reference, else metal's (47 compared, msaa does not apply); `cd packages/engine && bun tests/web/reference-gfx.ts`, at 3d7ad18 (PASS, 31 s).
- `web-gfx-webgl2.json`: the same on WebGL2 (47 compared, taa does not apply; msaa, showcase, overlay and water-3d against `crates/og-tests/references/webgl2/`); `bun tests/web/reference-gfx.ts --webgl`, at 3d7ad18 (PASS, 52 s).
- `m4-m5-tooling.md`: the M4 glTF edit to reload run through `og dev` and the M5 timed 8-frame sweep: commands, numbers, load, and the defect the first run found.
- `m4-gltf-reload.json`: the eight reloads of that run (og dev's rebuild, swap and total; the driver's write-to-line time) and the red pixels counted in each capture; at 3d7ad18.
- `m4-gltf-reload-before-after.png`: the dropship before the edit and after reload 1 (crop of the 1280×720 captures); at 3d7ad18.
- `m5-sweep-8-frames.json`: the five timed sweeps; at 3d7ad18.
- `m5-sweep-8-frames.png`: the eight frames of the first sweep (Spin.speed -3 to 4), quarter size.
- `wasm-build-relink.md`: the gate's wasm-build before and after the publish variant got its own target dir (d43d62f).
- `break-and-restore.md`: each new check against the break it catches.
