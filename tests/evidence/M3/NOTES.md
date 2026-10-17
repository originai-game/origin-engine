# M3 evidence

One line per file: what it is, how it was produced, at which revision.

- `showcase-metal.png` — og-render M3 showcase scene (gltf models, CSM, shadowed spot + point light, bloom, auto exposure, MSDF world text, sprites, TAA) on Apple M4 Pro Metal: OG_GFX_CAPTURE=1 cargo test --release -p og-tests --test gfx (the committed reference) (rev 924ca0d+dirty, 2026-10-17, sha256 74b159885d0a66f7)
- `showcase-vulkan-cpu.png` — the same showcase on Mesa lavapipe (llvmpipe LLVM 15, Mesa 25.0.7) in the E2B origin-runner-probe sandbox: bun crates/og-tests/scripts/capture-vulkan-cpu.ts; RMSE 0.0049 vs the Metal frame (rev 924ca0d+dirty, 2026-10-17, sha256 40f886d3cdce3101)
- `perf-baselines.json` — OG_PERF_RECORD=1 cargo test --release -p og-tests --test perf -- --ignored perf --test-threads=1 (best of 7x100 frames, M4 Pro Metal), merged from crates/og-tests/perf/metal-apple-m4-pro/*.json; lavapipe timings from og-gfx --frames 10 in E2B (rev 924ca0d+dirty, 2026-10-17, sha256 a48c31e08cfec6ac)
- `feature-costs.md` — og-gfx --costs --frames 100 (M4 Pro Metal, best of 2 sweeps x 5 runs), an interleaved 9-round A/B against the M0 og-gfx (ffd0e3d), and RMSE of crates/og-tests/references/metal vs vulkan-cpu (rev 924ca0d+dirty, 2026-10-17, sha256 d1c08a7716a9237c)
