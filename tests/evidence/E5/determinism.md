# E5 determinism records

## Physics beyond rigid bodies (og-physics `tests/beyond_determinism.rs`)

A 3D world (a character walking onto a moving platform, a motorised hinge, a
chain of ball joints, a raycast car, a Verlet rope and a cloth, height-field
water with crates dropped in, a ragdoll that goes limp and recovers) and a 2D
world (a platformer running, jumping through a one-way platform and off a
wall, a wheel-joint car, a particle fluid, a water line). Each runs 1200 ticks
twice. At tick 600 a snapshot is restored into a fresh world built with
another seed, and that world runs on to tick 1200.

| Profile | tick 0 | tick 600 | tick 1200 | tick 1200 after the restore | snapshot |
|---|---|---|---|---|---|
| 3D | `bd8fe3534792ea5d` | `31ba620e34d21f17` | `d8e85e787700916a` | `d8e85e787700916a` | 43,473 B |
| 2D | `6df8e6c577c260ba` | `7f91e43ca0e00f74` | `229340c9164d3fca` | `229340c9164d3fca` | 20,510 B |

The hashes are bit-identical on aarch64 macOS (M4 Pro, native) and on
x86_64 Linux (the test cross-built with `cargo zigbuild --release --target
x86_64-unknown-linux-gnu.2.31` and run in the og-lavapipe container). The
test also checks that ropes, cloth, fluid particles, water heights, wheel
poses and ragdoll bones read back equal after the restore, and that every
feature moved.

## Through TypeScript (packages/engine `tests/e5.test.ts`)

A 3D world with every new component (joint, character, car, rope, cloth,
water and a crate splashing into it) and a 2D world (platformer with a held
jump, fluid, water line, a falling rock) each run 150 ticks, snapshot, run
150 more, restore, and run 150 again. The test asserts the hash after the
restore equals the snapshot's hash, the end hash equals the uninterrupted
run's, and two fresh engines end on the same hash. `engine.hash()` is not
pinned there, because the TypeScript fixtures differ from the Rust ones.

## Rendering

The new og-tests scenes and fx-showcase stations are deterministic frames:
the vulkan-cpu references recorded in the container were reproduced by a
second run at RMSE 0 (`fx-showcase-vulkan-cpu.txt`; og-tests: 11 of 11
atmosphere tests). The day-night clock is a function of the tick
(`start + tick * dt * speed`, asserted inside and outside systems), so the
sky and the light do not depend on wall time. Weather particles run in the
renderer's seeded particle simulation under a fixed system id that no entity
has.
