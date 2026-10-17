# M1 cross-target determinism (Core workstream, 2026-10-17)

A scratch crate over og-core and og-ecs, run on native arm64 (debug and release),
wasm32 scalar and wasm32 +simd128 under node. All four agree:

- og_core::math digest over 1.4M results: 2668451330784045241
- bouncing boxes, seed 42, 3600 ticks, world_hash: 12995895349266450384

x86_64 not run. The committed `tools/wasm-determinism/` check (Core follow-up)
is to make this repeatable in CI.
