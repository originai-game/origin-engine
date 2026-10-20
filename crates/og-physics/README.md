# og-physics

Physics for the engine (`docs/design.md` §2.4, §2.7; phase 1 M6): rapier3d
and rapier2d (`enhanced-determinism`) as one scheduled system, `og.physics`,
over the ECS. Game code owns the components; the system steps them and
writes the dynamic bodies back.

```rust
let physics = og_physics::install(&mut world, og_physics::Physics::default())?; // after the systems that move kinematic bodies
physics.set_mesh(7, og_physics::MeshDesc { positions, indices })?;               // geometry for convex/trimesh colliders
world.spawn((Transform::at(Vec3::new(0.0, -0.5, 0.0)), Collider::cuboid(Vec3::new(40.0, 1.0, 40.0))))?; // static floor
world.spawn((Transform::at(Vec3::Y * 5.0), RigidBody::dynamic(), Collider::sphere(0.5)))?;
world.step()?;                                                                   // the ball falls; og.Velocity is added
let hit = physics.query(&world)?.raycast(&RaycastDesc { origin, direction: -Vec3::Y, ..Default::default() })?;
```

Features `physics3d` and `physics2d` (both default) choose the rapier
builds; `og.Physics.profile` (3 or 2) chooses the one a world steps. In 2D
bodies move in the xy plane and rotate about z; a transform keeps its z.

## API map

| Item | What it is |
|---|---|
| `install(&mut World, Physics) -> PhysicsSystem` | registers the types, inserts `og.Physics`, schedules `og.physics` at the end of the schedule and adds the `og.physics` state block |
| `PhysicsSystem` | `set_mesh(handle, MeshDesc)`, `remove_mesh(handle)`, `query(&World) -> PhysicsQuery` |
| `PhysicsQuery` | `raycast(&RaycastDesc)`, `shape_cast(&ShapeCastDesc)`, `overlap(&OverlapDesc)` over the colliders as the world holds them when it was built; `QueryFilter { mask, include_sensors, exclude }` |
| `raycast`, `shape_cast`, `overlap` | the same, on a view built for one call |
| `SYSTEM_NAME` | `og.physics`, the system's and the state block's name |

## Records (for `api.ogt`)

Field order, names and storage types are the layout; the IDL declares them
exactly so (an IDL record of the same name and layout is adopted). Codes
are `u32` constants in `src/schema.rs`.

| Record | Kind | Fields |
|---|---|---|
| `og.Transform` | component | `position: vec3`, `quaternion: quat`, `scale: vec3` (registered by schema, so the engine's own is adopted) |
| `og.RigidBody` | component | `kind` (0 dynamic, 1 static, 2 kinematic), `mass` (0: from density), `linear_damping`, `angular_damping`, `gravity_scale`, `lock` (bits: translation x 1, y 2, z 4; rotation x 8, y 16, z 32), `ccd` |
| `og.Collider` | component | `shape` (0 box, 1 sphere, 2 capsule, 3 cylinder, 4 convex, 5 trimesh), `size`, `radius`, `height`, `mesh`, `offset`, `rotation`, `friction`, `restitution`, `density`, `sensor`, `layers`, `mask` |
| `og.Velocity` | component | `linear`, `angular`; written for dynamic bodies, read for kinematic ones, set it to launch or stop a body |
| `og.Physics` | resource | `profile`, `gravity`, `substeps` (1–16), `solver_iterations` (1–32), `rest_speed` |
| `og.CollisionStarted` | event | `a`, `b` (entity ids, a < b), `point`, `normal` (from a to b) |
| `og.CollisionEnded` | event | `a`, `b` |
| `og.SensorEntered`, `og.SensorExited` | event | `sensor`, `other` |
| `og.Contact` | event | `a`, `b`, `sensor` (bit 0: a is a sensor, bit 1: b); every touching pair, each tick |

A collider without a `RigidBody` is static at its transform. Events sent in
tick N are read in tick N+1.

## Determinism and the state block

The rapier world persists between ticks, so contacts and solver warm starts
carry over. It is world state: the `og.physics` state block
(`og_ecs::StateBlock`) saves it with bincode, together with what each
entity's components were when physics last touched them. `snapshot`,
`restore` and `world_hash` therefore cover the whole simulation, and a world
restored at any tick continues exactly as the uninterrupted run
(`tests/pile.rs`: 500 bodies, 3600 ticks, churn, a kinematic paddle, a
sensor).

Each tick, before stepping, the system brings the rapier world in line with
the components, in query order (archetype creation order, then row order):

- a body or static collider whose entity is gone leaves, in entity order;
- a new body enters at its transform and velocity;
- a changed `og.RigidBody`, `og.Collider` or `og.Transform.scale` rebuilds
  that body (it loses its contacts' warm starts);
- a transform or velocity that game code changed since physics wrote it
  moves the rapier body (a teleport); kinematic bodies follow their
  transform and velocity every tick;
- a mesh registered with new geometry rebuilds the colliders made from it;
  the same geometry again changes nothing.

A restore of a snapshot without the block, and a tick that failed, rebuild
the rapier world from the components. Resting is the rest rule, not
rapier's sleep: a dynamic body supported with at most 2 mm of overlap and
slower than `rest_speed` before and after the step keeps its pose and loses
its residual velocity.

Meshes are assets, not world state: register them again in a fresh engine
before or after restoring.


## The step on the lanes

The whole step (bringing the rapier world in line, rapier's step, the
contact and body readout, characters, platformers, soft bodies) runs inside
`og_lanes::install` at the `physics` site. A world of at least `BODY_GRAIN`
(200) bodies for every lane thread (800 on four lanes, 1,600 on eight) steps
on the lanes pool: rapier's `parallel` build splits its broad phase, narrow
phase and staged solver across the pool's threads, and the readout of the
contact pairs splits by 512 pairs a task (`og_lanes::map`). Below that, and
with lanes off, the step runs on the calling thread inside its one-thread
pool, where rapier finds one thread and takes its serial paths, so a small
world never pays for waking the pool. The pool is rayon natively and
wasm-bindgen-rayon's Workers in the lanes wasm (rapier is built with
`parallel` there too, `cfg(target_feature = "atomics")`); the
single-threaded wasm builds rapier without it.

Why the world is the same on any number of threads: rapier under
`enhanced-determinism` promises bitwise-identical results for any pool size
and between its `parallel` and sequential builds (its
`tests/thread_count_determinism.rs` and `tests/parallel_path_parity.rs`):
its solver lays constraints out for a fixed eight workers whatever the pool
has, its parallel passes write to precomputed disjoint slots or concatenate
in the sequential order, and its narrow phase applies pair transitions in a
fixed order. og-physics's own split, the contact readout, is a map whose
results come back in pair order and are folded serially (the deepest
penetration per body is a minimum). What checks it here:
`tests/pile.rs` (a pen of the pile's bodies large enough for the pool,
stepped with lanes on and off, the same world and snapshot),
`tools/wasm-determinism` (a 1,000-body pen on the host's four-lane release
and on the single-threaded wasm, built without `parallel`: one hash) and
`packages/engine/tests/lanes.test.ts` (a 1,000-box pile through the native
engine with lanes off and on four threads). The lanes wasm itself is checked
in the browser (the same hashes as the single-threaded module).

The grain was measured with `examples/lanes_costs.rs` on the M4 Pro
(8 performance and 4 efficiency cores, a shared machine):
`tests/evidence/P4/physics-lanes/`. Four lanes pay from about 750 bodies,
eight from about 1,500 (rapier's stages wait for their slowest worker, and
an eighth thread lands on an efficiency core or a busy one).
## Beyond rigid bodies (the E5 round)

Components over the same rapier world and state block (`src/schema_ext.rs`,
reached from TypeScript through `crates/og-api/api.ogt`'s physics module):

| Record | What the step does with it |
|---|---|
| `og.Joint` | fixed, revolute (limits, velocity or position motor), prismatic, spring, rope and ball joints between a body and another (`other`) or a world point; the second frame keeps the pose it was made in; `value` is written back |
| `og.CharacterController` | a kinematic capsule moved by `move` after the step: slopes to `max_slope`, steps to `step_height`, ground snapping, carried by the moving platform it stands on, jumps on the rising edge with coyote time, pushes dynamic bodies; writes `grounded`, `velocity`, `ground` |
| `og.PlatformerController` | 2D: run acceleration, jump height and time to apex, fall and jump-cut gravity, apex hang, coyote time, jump buffer, air jumps, wall slide and jump, one-way layers with drop-through, corner correction, moving platforms; sends `og.CharacterEvent` |
| `og.Vehicle` | 3D: rapier's raycast vehicle with a top-speed fade and reverse braking; 2D: wheel bodies on sprung, motorised wheel joints; `vehicle_wheels` for drawing |
| `og.Rope`, `og.Cloth` | Verlet strands and grids with pins, wind and gusts, pushed out of every solid collider; cloth with `friction` against solids, `substeps`, and self-collision by `thickness` (below); `rope_points`, `cloth_points` |
| `og.Fluid2D` | double density relaxation (Clavet et al.) on a grid in a fixed order, velocity capped at half the radius per substep; fluids of one `group` simulated together, layering by `density` and blending by `mixing` (below); `fluid_particles`, `fluid_mix` |
| `og.Fluid3D` | 3D profile: the same model over a spatial hash, against 3D colliders and height fields, in optional `bounds`, with an emitter (`emitRate` up to `maxCount`) and a sink box; groups layer and mix as in 2D (below); `fluid3d_particles`, `fluid3d_mix` |
| `og.WaterSurface`, `og.WaterLine` | a height field and a spring line splashed by crossing bodies (`og.Splash`), buoyancy and drag on dynamic bodies; `water_heights`, `water_line` |
| `og.Ragdoll` | capsule bones from a host-registered rig (`set_ragdoll_rig`; og-runtime makes one from an og-anim skeleton) on limited ball joints in 3D and on limited hinges about z in 2D (rapier2d bodies, the bones in the xy plane at their animated z: a planar skeleton), following targets kinematically at weight 0 (og-runtime's `og.ragdollTargets` system hands it the animation's pose each tick, between og.anim and og.physics), simulated above it, `blend` moving to `weight` over `blend_time` |

Everything simulates with + − × ÷ and square roots (og_core::math for the
rest), in entity order, inside the state block: `tests/beyond_determinism.rs`
runs a 3D and a 2D world with every feature 1200 ticks twice to the same
hash and snapshot, and a restore at tick 600 ends in the same world and the
same ropes, cloth, fluid, water, wheels and bones. Rigs are assets like
meshes. CPU costs
per feature: `cargo run --release -p og-physics --example soft_costs`
(`tests/evidence/E5/feature-costs.md`) and, for the phase 2 round's,
`--example anim_physics_costs` (`tests/evidence/P2/anim-physics/costs.md`),
and for the 3D fluid `--example fluid3d_costs`
(`tests/evidence/P2/sph/costs.md`).

### Phase 2: cloth self-collision, fluid groups, 2D ragdolls

- **Cloth self-collision** (`og.Cloth.thickness`, 0 off): each substep
  finds, from a grid of cell 1.5 × `thickness` sorted by cell and index,
  the pairs of points closer than 1.5 × `thickness` whose rest distance is
  at least `thickness` (nearer neighbours are the links' business), and
  every constraint pass pushes those closer than `thickness` apart, in pair
  order, by inverse mass (pins do not move); a point moves at most half the
  thickness a substep, so it cannot step through a layer. `substeps`
  (default 1) splits the tick, the damping spread over them; `friction`
  (default 0) takes part of the slide of a point resting on a solid, so a
  cloth lowered onto the floor folds instead of spreading. With
  `thickness` 0, `substeps` 1 and `friction` 0 the step is the E5 one.
  `tests/self_collision.rs`: a 20 × 20 sheet lowered onto the floor piles up
  in layers and no two of its points that are not near neighbours come
  closer than 0.99 × the thickness at any tick (without it, they meet and
  the sheet lies in one layer).
- **Fluid groups** (`og.Fluid2D.group`, `density`, `mixing`): fluids
  sharing a nonzero group (four at most, in entity order; their lanes) step
  as one particle set. Each particle weighs its fluid's `density` (by its
  mix); pressure comes from every neighbour (number density) against each
  particle's own rest density and stiffness; a pair's displacement and
  viscous impulse are shared in inverse proportion to the masses, so the
  lighter particle moves more and a heavier fluid sinks; unmixed particles
  of different fluids are held apart at the boundary by an extra near
  pressure that `mixing` takes away; touching particles even out their mix
  (`mixing` × 0.25 a tick at full overlap, conserving each fluid's amount)
  and their mass follows. `fluid_mix(entity)` reads a member's lane and its
  particles' mix (og-runtime draws a group as one metaball surface whose
  colour follows it). A fluid alone (group 0, or the only one of its group)
  steps exactly as before. `tests/fluid_groups.rs`: a heavy fluid poured on
  a light one sinks under it (397 of 400 heavy particles below the light
  fluid's median), with `mixing` 1 the two blend (each particle's share
  within ±0.005 of a half, the amounts kept to 1e-3).
- **2D ragdolls**: the 3D ragdoll's code (`src/ragdoll.rs`) built for both
  profiles, each with its joint: a hinge about z with ±`jointLimit` in 2D.
  `tests/ragdoll.rs`: a 2D figure follows its targets at their z, goes limp
  and is knocked along +x, its bones turn only about z, the knee stays
  within the limit (1.12 rad at most landing under the body's weight; 3.03
  without a limit), and it recovers.

### Phase 2: 3D particle fluids (`og.Fluid3D`, `src/fluid3d.rs`)

Double density relaxation (Clavet, Beaudoin and Poulin 2005) rather than
position-based fluids: one neighbour search and two passes a substep, where
PBF needs a solver loop of several iterations over the same pairs; it is
stable at 60 Hz with two substeps and is the 2D fluid's model, so groups,
layering, mixing, emitters and sinks mean the same in both.

- **Step**: gravity, then per substep a prediction, the neighbour pairs
  from a spatial hash of cells the size of `radius` (a counting sort by
  cell; each particle searches its own cell and 13 forward ones, so every
  pair is found once, in a fixed order), viscosity on approaching pairs,
  the double density relaxation (pressure against `restDensity` gives
  cohesion and surface tension, near pressure keeps particles apart), and
  the collisions: the 3D colliders near each particle (pushed out, their
  normal velocity taken, the slide slowed), height fields (the surface
  from the triangle under the point, so nothing tunnels the terrain) and
  `bounds` around the entity.
- **Groups**: up to four fluids of one nonzero `group` step as one particle
  set, weighted by `density` (a heavier fluid sinks under a lighter one) and
  evening out their mix by `mixing`; the boundary between unmixed fluids is
  held at a tenth of 2D's strength (at 2D's it traps a light layer under a
  heavy one).
- **Emitters and sinks**: `emitRate` particles a second appear in a cube of
  `emitRadius` around `emitOffset` with `emitVelocity`, up to `maxCount`;
  a `sinkSize` box at `sinkOffset` removes those that enter it.
- **State and readouts**: everything is in the og.physics state block
  (snapshots, restore and the world hash cover it; the cross-target check
  pours two into a terrain bowl). `fluid3d_particles(entity)` and
  `fluid3d_mix(entity)`; og-runtime hands the particles to og-render's
  screen-space surface (`Scene::fluids_3d`).
- **Cost** (`--example fluid3d_costs`, M4 Pro, ms of CPU a tick; wasm32
  within 10 % of native except 4,000 at one substep, 18 % faster):
  1,000 particles 0.39 at one substep and 0.60 at two; 2,000: 0.86 and
  1.8; 4,000: 2.6 and 4.2; 8,000: 4.6 and 9.8. Particles piled deep (2,000
  in a terrain bowl) cost about twice those on a floor, having more
  neighbours. Keep a scene to a few thousand particles at two substeps.
- `tests/fluid3d.rs`: a block poured into a pool settles inside its bounds,
  a zero-gravity drop holds together and rounds, a viscous fluid spreads
  slower than water, fluid poured into a terrain bowl and onto a box stays
  on them, an emitter fills to `maxCount` and a sink drains it, a heavy
  fluid layers under a light one and two with `mixing` 1 blend keeping
  their amounts, a restore replays exactly, and the 2D profile leaves it
  alone.

## Tests

`cargo test -p og-physics`: `bodies.rs` (write-back, events, sensors,
layers, kinematic, locks, errors), `edits.rs` (component changes reaching
the persistent world, rewinding by restore), `queries.rs`, `profile_2d.rs`,
`pile.rs` (determinism and restore, set `OG_PHYSICS_PILE_REPORT` to write
its JSON report; lanes on and off on a pen large enough for the pool), and the E5 round's `joints.rs`, `controllers.rs`,
`vehicles.rs`, `soft.rs`, `ragdoll.rs`, `beyond_determinism.rs`, and phase 2's `fluid3d.rs`. The dev profile builds og-physics and rapier at
`opt-level = 3`; the pile takes about 15 s. `examples/lanes_costs.rs`
times the step with lanes off and on (`OG_LANES=off|4|8`, release).

## Status 2026-10-18 (what phase 1 still owes)

- **Reaching TypeScript**: every record above is in the IDL's physics
  module (`crates/og-api/api.ogt`, the E5 records since b8cdc04), and
  og-runtime links og-physics for both hosts. The readouts (`ropePoints`,
  `clothPoints`, `fluidParticles`, `waterHeight`, `vehicleWheels`,
  `ragdollPose`) and the rig from an og-anim skeleton
  (`ragdollRigFromSkeleton`, `setRagdollRig`) are `og.physics` functions.
- **Character controller**: delivered in the E5 round (above), with the 2D
  platformer controller, joints, vehicles, soft bodies, fluids, water and
  ragdolls.
- **wasm size**, measured (`tests/evidence/M6/physics-wasm-size.json`): the
  crate adds 694 KB brotli for the 3D profile, 541 KB for 2D and 1.05 MB
  for both to a module that already links og-core and og-ecs, which keeps
  og-host-web (1.10 MB at 49ed935) inside the 4 MB core budget.
- Evidence of the crate: `tests/evidence/M6/physics-*` (the pile's hashes
  agree in debug and release, and the restore at tick 1800 ends in the
  uninterrupted run's world).
