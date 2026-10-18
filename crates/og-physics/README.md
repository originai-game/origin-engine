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

## Beyond rigid bodies (the E5 round)

Components over the same rapier world and state block (`src/schema_ext.rs`,
reached from TypeScript through `crates/og-api/api.ogt`'s physics module):

| Record | What the step does with it |
|---|---|
| `og.Joint` | fixed, revolute (limits, velocity or position motor), prismatic, spring, rope and ball joints between a body and another (`other`) or a world point; the second frame keeps the pose it was made in; `value` is written back |
| `og.CharacterController` | a kinematic capsule moved by `move` after the step: slopes to `max_slope`, steps to `step_height`, ground snapping, carried by the moving platform it stands on, jumps on the rising edge with coyote time, pushes dynamic bodies; writes `grounded`, `velocity`, `ground` |
| `og.PlatformerController` | 2D: run acceleration, jump height and time to apex, fall and jump-cut gravity, apex hang, coyote time, jump buffer, air jumps, wall slide and jump, one-way layers with drop-through, corner correction, moving platforms; sends `og.CharacterEvent` |
| `og.Vehicle` | 3D: rapier's raycast vehicle with a top-speed fade and reverse braking; 2D: wheel bodies on sprung, motorised wheel joints; `vehicle_wheels` for drawing |
| `og.Rope`, `og.Cloth` | Verlet strands and grids with pins, wind and gusts, pushed out of every solid collider; `rope_points`, `cloth_points` |
| `og.Fluid2D` | double density relaxation (Clavet et al.) on a grid in a fixed order, velocity capped at half the radius per substep; `fluid_particles` |
| `og.WaterSurface`, `og.WaterLine` | a height field and a spring line splashed by crossing bodies (`og.Splash`), buoyancy and drag on dynamic bodies; `water_heights`, `water_line` |
| `og.Ragdoll` | capsule bones with limited ball joints from a host-registered rig (`set_ragdoll_rig`; og-runtime makes one from an og-anim skeleton), following targets kinematically at weight 0 (og-runtime's `og.ragdollTargets` system hands it the animation's pose each tick, between og.anim and og.physics), simulated above it, `blend` moving to `weight` over `blend_time` |

Everything simulates with + − × ÷ and square roots (og_core::math for the
rest), in entity order, inside the state block: `tests/beyond_determinism.rs`
runs a 3D and a 2D world with every feature 1200 ticks twice to the same
hash and snapshot, and a restore at tick 600 ends in the same world and the
same ropes, cloth, fluid, water, wheels and bones. Rigs are assets like
meshes. Not attempted: 3D SPH (the 3D water is the height field); cloth
self-collision; ragdolls in 2D. CPU costs per feature:
`cargo run --release -p og-physics --example soft_costs`
(`tests/evidence/E5/feature-costs.md`).

## Tests

`cargo test -p og-physics`: `bodies.rs` (write-back, events, sensors,
layers, kinematic, locks, errors), `edits.rs` (component changes reaching
the persistent world, rewinding by restore), `queries.rs`, `profile_2d.rs`,
`pile.rs` (determinism and restore; set `OG_PHYSICS_PILE_REPORT` to write
its JSON report), and the E5 round's `joints.rs`, `controllers.rs`,
`vehicles.rs`, `soft.rs`, `ragdoll.rs`, `beyond_determinism.rs`. The dev profile builds og-physics and rapier at
`opt-level = 3`; the pile takes about 15 s.

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
