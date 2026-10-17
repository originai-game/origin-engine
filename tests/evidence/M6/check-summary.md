# scripts/check.sh at 49ed935

`CARGO_TARGET_DIR=/Volumes/app/origin-engine/target/agent-qa OG_EVIDENCE_DIR=<target>/evidence-head bash scripts/check.sh` on a `git archive 49ed935` export (the shared working tree carried other agents' uncommitted edits), Apple M4 Pro, macOS, rustc 1.97.0, bun 1.4.2, node v26.3.0, 2026-10-18. Every step passes. The contents of `check-summary.txt`:

```
PASS fmt (1s)
PASS clippy (42s)
PASS test (157s)
PASS wasm-build (61s)
PASS wasm-opt (6s)
PASS size-gate (6s)
PASS drift (20s)
PASS bun-test (92s)
PASS determinism (61s)
```

size-gate: og_host_web.wasm 3,870,508 bytes after wasm-opt, **1,099,937 brotli** against the 4,194,304 budget (M2: 898,246; +22 % from M6 render, audio, probe and the runtime's asset loaders). Physics and UI are not linked into og-host-web yet (see physics-wasm-size.json).

drift: "og api check: the generated files match crates/og-api/api.ogt.". determinism: math digest 6117185439544271997 and bouncing-boxes hash 9205731540494195112 on all four builds.
