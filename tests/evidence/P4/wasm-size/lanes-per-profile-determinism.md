# M1 cross-target determinism

Produced by `tools/wasm-determinism/run.sh --report` at 9d220790 on aarch64-apple-darwin.
glam is built with `scalar-math`; the math digest covers `og_core::math`
and glam Vec3A/Vec4/Quat/Mat4 arithmetic (1.4M results); the world is
`crates/og-ecs/tests/support/bouncing_boxes.rs`, seed 42, 3600 ticks.
The host release runs with lanes off and on (crates/og-lanes: the
1,000-body pen's rapier step on a four-thread pool, the beyond scenario
below og-physics's grain; the walk's few skeletons stay below og-anim's
grain, so animation runs serially either way); the scalar and simd128
wasm builds are single-threaded, with rapier built without its parallel
feature; the lanes builds (one per physics profile, as the 3D and 2D
lanes modules are built) have it and run on one lane under node, each
with its own profile's physics hash only (- for the other).

| build | math digest | bouncing boxes world_hash | physics 3D world hashes | physics 2D world hash | anim world hash and retarget |
|---|---|---|---|---|---|
| aarch64-apple-darwin debug | 6117185439544271997 | 15551159556835976588 | 12669269624230193360 | 12448362945291953010 | 6030136770684287795 |
| aarch64-apple-darwin release, lanes off | 6117185439544271997 | 15551159556835976588 | 12669269624230193360 | 12448362945291953010 | 6030136770684287795 |
| aarch64-apple-darwin release, lanes on (4 threads, 1660 parallel calls) | 6117185439544271997 | 15551159556835976588 | 12669269624230193360 | 12448362945291953010 | 6030136770684287795 |
| wasm32 scalar (node v26.3.0) | 6117185439544271997 | 15551159556835976588 | 12669269624230193360 | 12448362945291953010 | 6030136770684287795 |
| wasm32 simd128 (node v26.3.0) | 6117185439544271997 | 15551159556835976588 | 12669269624230193360 | 12448362945291953010 | 6030136770684287795 |
| wasm32 lanes 3d (atomics, one lane; node v26.3.0) | 6117185439544271997 | 15551159556835976588 | 12669269624230193360 | - | 6030136770684287795 |
| wasm32 lanes 2d (atomics, one lane; node v26.3.0) | 6117185439544271997 | 15551159556835976588 | - | 12448362945291953010 | 6030136770684287795 |

All builds agree: math digest 6117185439544271997, world_hash 15551159556835976588, physics 3D 12669269624230193360, physics 2D 12448362945291953010, anim 6030136770684287795.

x86_64 (SSE2) not run here: run the script on an x86_64 host (CI) to cover it.
