# M1 cross-target determinism

Produced by `tools/wasm-determinism/run.sh --report` at dba5ceb3 on aarch64-apple-darwin.
glam is built with `scalar-math`; the math digest covers `og_core::math`
and glam Vec3A/Vec4/Quat/Mat4 arithmetic (1.4M results); the world is
`crates/og-ecs/tests/support/bouncing_boxes.rs`, seed 42, 3600 ticks.
The host release runs with lanes off and on (crates/og-lanes: the
1,000-body pen's rapier step on a four-thread pool, the beyond scenario
below og-physics's grain; the walk's few skeletons stay below og-anim's
grain, so animation runs serially either way); the wasm builds are
single-threaded, with rapier built without its parallel feature.

| build | math digest | bouncing boxes world_hash | physics world hashes | anim world hash and retarget |
|---|---|---|---|---|
| aarch64-apple-darwin debug | 6117185439544271997 | 15551159556835976588 | 18126409214070009362 | 14507017669422143574 |
| aarch64-apple-darwin release, lanes off | 6117185439544271997 | 15551159556835976588 | 18126409214070009362 | 14507017669422143574 |
| aarch64-apple-darwin release, lanes on (4 threads, 483 parallel calls) | 6117185439544271997 | 15551159556835976588 | 18126409214070009362 | 14507017669422143574 |
| wasm32 scalar (node v26.3.0) | 6117185439544271997 | 15551159556835976588 | 18126409214070009362 | 14507017669422143574 |
| wasm32 simd128 (node v26.3.0) | 6117185439544271997 | 15551159556835976588 | 18126409214070009362 | 14507017669422143574 |

All builds agree: math digest 6117185439544271997, world_hash 15551159556835976588, physics 18126409214070009362, anim 14507017669422143574.

x86_64 (SSE2) not run here: run the script on an x86_64 host (CI) to cover it.
