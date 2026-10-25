# r4/physics-contact: the hit on og.CollisionStarted (Rumpus Isle FINDINGS #9)

Branch `r4/physics-contact`, cut from `1bb341e5` (origin/main, 0.4.0).

## What landed

| Commit | What |
|---|---|
| `80027637` | `og.CollisionStarted` gains `impulse` and `relativeVelocity` in og-physics (both profiles: `sim.rs` is compiled once per profile), in `api.ogt` (and a new unit `newtonSecond`; `point` is now declared `unit m`), with the generated outputs regenerated from the IDL (`og_idl::generate`/`write`, the same calls `og api gen` makes; `og api check` passes). The CHANGELOG `Unreleased` line is in this commit. Tests: og-physics `tests/contact_hits.rs` and og-runtime `tests/contact_hit.rs`. |
| `b988c7b6` | og-game skill §13 now describes the two fields and no longer advises caching velocities. Rumpus Isle's cache workaround is removed (`Critter.vx`/`vz` and Drive's writes; `Bumps` reads `relativeVelocity` along the flat normal), and FINDINGS #9's row is updated. |
| (this commit) | These notes. |

## The fields, their units and orientation

- `impulse` (f32, N·s, ≥ 0; IDL `unit newtonSecond`, Rust `unit = "N*s"`).
  This is the normal impulse rapier's solver applied to the pair in the
  tick's step: `ContactPair::total_impulse_magnitude()`, which sums
  `ContactData::impulse` over the solver manifolds' points. It is summed
  over substeps when `og.Physics.substeps` > 1. `b` takes `+impulse ·
  normal`, and `a` takes `-impulse · normal`.
- `relativeVelocity` (Vector3, m/s; z = 0 in 2D). This is `a`'s velocity
  relative to `b` at `point`: `v_a(point) − v_b(point)`, where each side
  is `linear + angular × (point − centre of mass)`. The velocities are
  read as the substep that made the contact began, before the solver
  answered it. The centre of mass is taken after the step, so the lever
  arm matches the reported point. Its part along `normal` (a→b) is the
  closing speed, positive while they approach. A static collider (no
  rapier body) is still. A velocity-based kinematic body counts at its
  og.Velocity. A position-based kinematic body (a character or platformer
  controller) counts at the velocity carrying it to its target, which is
  what rapier interpolates inside the step.
- With several substeps, a pair touching after a substep that is not the
  last carries its impulse and its first relative velocity forward, so a
  contact that starts mid-tick reports the whole tick's impulse and the
  velocity it met with. A pair that stops touching in between starts
  over.
- Sensor overlaps send no CollisionStarted, so they have no hit.
  `og.Contact` and `CollisionEnded` are unchanged.

## Tests and results

`crates/og-physics/tests/contact_hits.rs` (4 tests, each run in the 3D
and 2D profiles). On `1bb341e5` the fields do not exist, so the file
does not compile.
- Light into heavy: a 1 kg ball at 6 m/s into a 10 kg ball, weightless and
  frictionless. Each ball's momentum change equals the impulse along the
  normal, in opposite directions (within 2 %). The impulse is 6·10/11 (the
  reduced mass, restitution 0). `relativeVelocity` is (6,0,0) with the
  light ball as `a`, and (−6,0,0) with the normal flipped when the heavy
  ball is `a` (spawned first), so the closing speed is 6 either way. A
  `substeps 4` run checks the carry-over. With the carry disabled
  (mutation), that case fails with impulse 0 and velocity ≈ 0.
- Spin: a 2 m rod spinning at 10 rad/s with no linear velocity sweeps into
  a static slab. `relativeVelocity` equals ω × r at the event's point
  within 5 %, the closing speed is > 5 m/s, and the impulse is > 0.
- Static and kinematic: a 2 kg box dropped onto a static floor. Its
  `relativeVelocity` equals minus its falling velocity as the step began.
  The impulse equals its momentum change plus m·g·dt for that tick (11.20
  against 10.88 + 0.33). In a second case, a weightless 1 kg box is struck
  by a kinematic platform rising at 3 m/s: the relative velocity is
  (0,3,0) and the impulse equals the box's new velocity times its mass.
- Snapshot: a spinning rod and a ball. A snapshot taken five ticks in,
  before the hit, is restored and replayed, and the event comes back equal
  field for field on the same tick.

`crates/og-runtime/tests/contact_hit.rs` (1 test, profiles "3d" and
"2d"). A game's system reads `world.events("og.CollisionStarted")`
through the host ABI. The light-into-heavy hit reads
`relativeVelocity.x` 6, `impulse` 5.4545. A save taken before the hit and
restored with `world.restore` reads the same event on the same tick. On
the old code, `impulse` is missing from the JSON, so the test fails.

Other runs: `cargo test -p og-physics` passes in full (25 test binaries,
including pile, lanes, determinism, saves and time_scale).
`cargo test -p og-api`, which includes the IDL-vs-Rust layout check in
`tests/modules.rs`, passes. `cargo test -p og-idl` passes, and so do
og-runtime's `mid_tick_save`, `restored_event` and `no_renderer`.
`cargo clippy -D warnings` passes for og-physics and og-runtime's tests,
and `cargo fmt --check` passes. In packages/knowledge, `validate` and
`bun test` pass (48 tests). `tsc` on examples/rumpus-isle passes. The
existing test that checks the old Rumpus Isle numbers is FINDINGS #9's
repro, run on the native host with a dev addon of this branch
(`og-host-node`, put at `packages/engine/native/` for the run and removed
after) and a temporary `console.log` in `Bumps`:

```
OG_RENDERER=none og run examples/rumpus-isle --headless --commands "0.2:seat:wasd;0.3:seat:arrows;0.4:match;0.5:go;0.8:key_down:KeyA;0.8:key_down:ArrowRight;1.6:key_up:KeyA;1.6:key_up:ArrowRight;2.0:quit"
bump closing 9.23 impulse 6.30
bump closing 2.30 impulse 2.24
tick 120 hash 57d1a1c4fce1fd39 entities 122
```

The head-on closing speed is 9.23 m/s, the same as the cache workaround
gave (FINDINGS #9). The impulse, 6.30 N·s, fits two 1 kg critters at
restitution 0.35: 0.5 · 1.35 · 9.23 ≈ 6.23. Run against the published
0.4.0 addon, the same script fails on `rv.x` (no such field). Any run of
a game checkout that predates the engine that ships this needs the new
addon.

## Decisions

- **No ContactForceEvent.** Rapier sends it only above a per-collider
  force threshold, and only through an event handler on `step_with_events`.
  The pair's solver data (`ContactData::impulse`, written back by the
  solver every substep) is already on the contact graph that the readout
  walks, so the impulse comes from there, deterministic and in edge order.
- **The impulse is the normal part only, summed over points.** Rapier
  stores friction as warm-start data in the tangent basis, and its sign
  conventions differ between 2D and 3D. A game weighing a hit wants the
  normal part. One scalar plus the event's normal fixes both sides'
  vectors.
- **The velocity is taken before the solver, not after.** It is the
  number the finding asked for (how fast they met). The impulse already
  holds the solver's answer.
- **No save or schema version bump.** og-ecs snapshots store every type's
  layout hash and field table. The new fields change
  `og.CollisionStarted`'s layout hash, so a save taken before this is
  refused by a plain restore, naming the type, and maps field by field
  with `migrate` (new fields read 0). The og.physics state block (the
  bincode of the rapier `Sim`) does not change: nothing new is kept
  between ticks. The motions and the carry-over are rebuilt every step.
  og-physics schemas never set `version` (`RigidBody.centerOfMass` and
  others were added without one), and the IDL check builds its schemas at
  version 1. `UiKey`/og-ui state 9 and `TerrainMeta v4` were bumps of
  hand-encoded formats, which this is not.
- **Determinism.** No clock is read. The motions are recorded in rapier's
  body order and indexed by handle. The carry-over is a `BTreeMap` keyed
  by collider handles. The readout stays in the lanes' edge ranges,
  joined in edge order as before. The snapshot test and the runtime test
  replay the same values.
- **Rumpus Isle keeps `Bumps` before `Drive`** in its system list. The
  order no longer matters for the closing speed, and moving it could move
  its frames for no gain.

## Left out

- Friction (tangent) impulse and a per-point list. Neither was asked
  for, and see above.
- The hit on `og.Contact` (sent every tick for every touching pair) and on
  sensor events. Only the start of a contact is a hit. A game that wants
  the force of a resting contact can ask for it later. That would be
  another field on Contact, at a per-tick cost for every pair.
- A test for a position-based kinematic body (a controller) hitting a
  dynamic body. The code takes the controller's velocity from rapier's
  interpolation (`RigidBodyPosition::interpolate_velocity`, as rapier does
  inside the step), but no test covers that case.

## What the coordinator must re-check

- **World hashes change by design** for every world that registers
  og-physics: the type table carries `og.CollisionStarted`'s new layout
  hash. Rumpus Isle's hashes also change because `Critter` lost two fields.
  The simulation itself is unchanged: the step only reads more. No pinned
  hash literal was found in the tests, and every og-physics determinism
  test passes. A cross-host hash comparison (`packages/engine/tests`,
  web-reference) needs the wasm module rebuilt with this change before it
  compares.
- **Metal reference frames: none expected to move.** The step's
  integration is untouched. The one game-side change is Rumpus Isle's
  `Bumps` closing speed, now `relativeVelocity`'s flat projection instead
  of the cached Drive velocity. In the head-on repro it reads 9.23 m/s as
  before. A spin term about y is perpendicular to the normal for these
  spheres, but the last bits of the shove could differ, so re-check
  `examples/rumpus-isle/references/metal/*` (the bump ring's frames first)
  in the functional lane. Nothing else reads the new fields yet.
- **Cost:** each substep now records every body's velocity (one pass over
  rapier's bodies), and the readout computes the hit for every touching
  solid pair. `tests/pile.rs` passes. The perf lane's baselines are for the
  wave's end.
- The functional lane (fmt, clippy, every test, drift,
  bun-functional): `og api check` passed here, but the full lane was not
  run, per AGENTS.md.
