# M6 Core evidence (lines for NOTES.md)

- `core-cross-target-determinism.md` — CARGO_TARGET_DIR=target/agent-core tools/wasm-determinism/run.sh --report tests/evidence/M6/core-cross-target-determinism.md, after FieldType bool/mat4/enum/str, entity names and snapshot format 2; aarch64-apple-darwin debug+release, wasm32 scalar and simd128 under node v26.3.0; math digest 6117185439544271997 and world_hash 9205731540494195112, both unchanged from M1 (rev f23f1ce+dirty, 2026-10-17, sha256 ae03a3cd3822fc6a)
- `core-input-edges-transcript.json` — cargo test -p og-input --test edges (copied from $CARGO_TARGET_TMPDIR/input-edges-transcript.json): the seven-tick script and what a game system saw each tick (rev f23f1ce+dirty, 2026-10-17, sha256 13c87c0a76886eb6)
- `core-break-and-restore.md` — each new Core test run against one deliberate defect, then the source restored; all 21 defects caught (rev f23f1ce+dirty, 2026-10-17, sha256 32dd6e4ea2d4dc79)
