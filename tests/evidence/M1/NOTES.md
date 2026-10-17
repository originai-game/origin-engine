# M1 evidence

One line per file: what it is, how it was produced, at which revision.

- `cross-target-determinism.md` — CARGO_TARGET_DIR=target/agent-core tools/wasm-determinism/run.sh --report <file> (glam scalar-math, og.Time/og.Rng names; aarch64-apple-darwin debug+release, wasm32 scalar and simd128 under node v26.3.0) (rev 7db717a+dirty, 2026-10-17, sha256 d2786e929d9706e7)
