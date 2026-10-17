# M1 cross-target determinism

Produced by `tools/wasm-determinism/run.sh --report` at 7db717a on aarch64-apple-darwin.
glam is built with `scalar-math`; the math digest covers `og_core::math`
and glam Vec3A/Vec4/Quat/Mat4 arithmetic (1.4M results); the world is
`crates/og-ecs/tests/support/bouncing_boxes.rs`, seed 42, 3600 ticks.

| build | math digest | bouncing boxes world_hash |
|---|---|---|
| aarch64-apple-darwin debug | 6117185439544271997 | 9205731540494195112 |
| aarch64-apple-darwin release | 6117185439544271997 | 9205731540494195112 |
| wasm32 scalar (node v26.3.0) | 6117185439544271997 | 9205731540494195112 |
| wasm32 simd128 (node v26.3.0) | 6117185439544271997 | 9205731540494195112 |

All builds agree: math digest 6117185439544271997, world_hash 9205731540494195112.

x86_64 (SSE2) not run here: run the script on an x86_64 host (CI) to cover it.
