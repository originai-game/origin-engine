# debt3 / physics-vehicle

Branch `debt/physics-vehicle` from `origin/main` f87c89d8. Upstream TODOs
cleared from `crates/og-physics/src/raycast_vehicle.rs`, the og-physics
port of rapier 0.36.1's `DynamicRayCastVehicleController`. No og.Vehicle
member changed (`api.ogt` untouched), so there is no drift to regenerate.

## Commits

| commit | what |
|---|---|
| 7f958533 | One pass per wheel, with the side damping named. Pure refactor, bit-identical hashes. |
| d0050cf1 | The friction denominators use the effective inverse mass, with the rails test. |
| e22ebac6 | A dynamic ground body takes the opposite of each wheel impulse, with the raft test. |
| (docs) | README vehicle row, the phase-1 row, these notes. |

## What each TODO became

- **"collapse all those loops into one"**: `update_vehicle` now does one
  pass per wheel (place, cast, spring); `update_suspension` became
  `Wheel::update_suspension`. `update_friction`'s four loops became a
  ground count, one sizing pass per wheel (axle, side impulse, forward
  impulse, friction circle) and one apply pass. Sizing still finishes for
  every wheel before anything is applied, because the impulses are sized
  against the same velocities, as upstream does. `resolve_single_bilateral`
  and `resolve_single_unilateral` became one `side_impulse(chassis,
  Option<ground>, …)`. The arithmetic stays in upstream's order.
- **"move this into proper structure"** (`contact_damping = 0.2`): this is
  now the `SIDE_FRICTION_DAMPING` const (0.2, documented). Bullet's
  `fwdFactor` 0.5 became `FORWARD_GRIP_SHARE`. Neither value changed.
- **"take the effective inv mass into account"** (×3): `inv_mass_along(body,
  n)` = n · (effective_inv_mass ∘ n), as rapier's solver uses it. When the
  effective inverse mass is isotropic it returns `local_mprops.inv_mass`
  exactly, so the rounding of |n|² cannot move a free chassis's hash.
  Additional mass is already part of `local_mprops`, so it adds nothing
  new. A fixed or kinematic body's effective inverse mass is 0.
- **"apply friction impulse on the ground"**: when a wheel's ground
  collider has a dynamic parent body, that body receives −(forward + side)
  impulse at the contact point. It also receives −suspension impulse,
  without which a raft would not carry the car's weight and momentum would
  not be conserved. Fixed bodies, kinematic bodies and parentless colliders
  receive nothing, as before.

## New tests (crates/og-physics/tests/vehicles.rs), before → after

- `a_car_on_rails_with_its_front_wheels_turned_scrubs_them`: the chassis
  has x translation and every rotation locked, steer 0.05, full throttle,
  50 s.
  - Old denominator: 34.90 m/s, slip 0. The derivation for 1/m with an
    18.2 v N drag gives 34.93.
  - New: 29.15 m/s, slip 0.998. The derivation for the friction-circle
    drag gives 29.21. The test asserts it within 0.3 of the derived value.
  - The old code fails it (34.90 against 29.21).
- `a_car_on_a_raft_drives_it_backwards_and_weighs_on_it`:
  - A 1,000 kg car on a 4,000 kg raft on a frictionless road, 2 s of
    throttle. The car reaches 8.857 m/s. The raft was at 0 on the old code
    and is at −2.214 m/s on the new (−v/4). Momentum along z went from
    8,857 to −0.004 kg m/s.
  - The same raft afloat (gravityScale 0, rotation locked), measured over
    1 s. The pair's p_y falls by 264 on the old code and by 9,810.0 on the
    new (m g t = 9,810).
  - The old code fails it (the raft's z velocity is 0).
- All 5 existing vehicle tests pass unchanged, with no numeric expectation
  moved. `cargo test -p og-physics` passes in full, and fmt and clippy
  `-D warnings` are clean.

## Hashes (world_hash, a scratch harness run at each commit, then deleted)

| scenario | base f87c89d8 | 7f958533 | d0050cf1 | e22ebac6 |
|---|---|---|---|---|
| car on a parentless road (drive, steer, brake; 4 checkpoints) | 133d…, 7f67…, 1c09…, 2ccb… | same | same | same |
| full lock, roll 0.1 | 8635cc64c345c527 | same | same | same |
| 2D car | 5d8b2c2b89a9b9fe | same | same | same |
| beyond_determinism 3D cast (4 × 300 ticks) | 4d8c…, 69f2…, dd7b…, a6e8… | same | same | same |
| car on a static og.RigidBody road (fixed body) | …, 18df0b130193db33 | same | **…, 1e0b8b8b74185192** | same as d0050cf1 |
| car on a dynamic raft | 2ce3…, ffdf…, 2e88…, 26e8… | same | same | **022f…, 3c8f…, f69f…, bfec…** |

Why these moved:

- **Fixed-body road at d0050cf1.** Only the braking checkpoint moves. The
  old rolling-friction denominator added the fixed ground body's
  `local_mprops.inv_mass`, a nonzero value from its collider's density, as
  if the road could recoil. Its effective inverse mass is 0. The term only
  matters when the brake limits the impulse (max_impulse > 0), which is why
  the other checkpoints keep their hash.
- **Raft at e22ebac6.** The new reaction impulses move the raft.

None of the existing vehicle tests compare a hash. `beyond_determinism`
compares runs against each other, and its cast's hash did not move. Rule
from the brief: every chassis that is free (no locked axes) on parentless
or dynamic ground keeps its hash at item 2.

## Owed to the Mac lane

No frame was re-recorded here, and none should move. Meridian Loop
(`examples/meridian-loop`) is the only example with an og.Vehicle. Its
cars are free chassis (mass 1000, no lock) on parentless track colliders:
the trimesh road and the floor box have no og.RigidBody. That is the
"parentless road" row above, bit-identical at every commit. The
coordinator checks these frames, which were not re-recorded:
`examples/meridian-loop/references/metal/{race,slide,jump,ghost,countdown,
pub-straight,pub-countdown}.png` (the driving scenes) and their webgl2
twins. The title, board, results, pause and settings frames do not drive.

## The 2D vehicle (vehicle2.rs)

There is no gap. Its wheels are rapier bodies on wheel joints, so their
contacts go through rapier's solver, which applies the reaction to a
dynamic ground itself. Scratch measurement on a 16 kg raft on ice: car plus
wheels (6 kg) at 5.743 m/s, raft at −2.153 m/s, 34.46 against 34.45 kg m/s.
Unchanged.
