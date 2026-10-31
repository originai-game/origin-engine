---
name: og-physics
description: Physics on Origin Engine — rigid bodies and colliders over rapier (3D and 2D), kinematic lifts, collision events with their closing speed and impulse, raycasts, shape casts and overlaps, forces, impulses and torques, and the simulated matter beyond rigid bodies: joints and motors, the 3D character controller and the 2D platformer controller (slopes, steps, moving platforms, pushing bodies), raycast vehicles, ropes and cloth, 2D and 3D particle fluids, and ragdolls blended from the animation. Use when anything in a game falls, collides, is pushed, walks on level geometry, drives, swings, flows or goes limp, or when a query must find what is hit.
---

Physics on Origin Engine is components over one deterministic simulation:
a body is `og.RigidBody` and `og.Collider` on an entity, a character is a
controller component, and the queries answer from the tick's start. How
fast, how high and how heavy for each genre is og-feel §12.

- §1 Bodies, contacts, forces and queries: kinematic bodies, collision
  events, raycasts, shape casts and overlaps, forces and impulses.
- §2 Beyond rigid bodies: joints, the character and platformer
  controllers, vehicles, ropes and cloth, 2D and 3D fluids, ragdolls.

## 1. Bodies, contacts, forces and queries

`og.RigidBody` and `og.Collider` on an entity make it a body in the
profile's simulation (rapier3d in 3D, rapier2d in 2D); contacts and sensors
arrive as events, and `og.physics` answers queries. `og.Velocity` is
optional: the step adds it to a dynamic body that has none, and a kinematic
body (`BODY_KINEMATIC`) moves by it each tick, its Transform written back:
a lift is a kinematic body with a Velocity whose system only turns the
velocity round at the ends, and a PlatformerController standing on it
rides along. A kinematic body without one stays where its Transform puts
it (move that by hand), and writing a moving one's Transform teleports it
there. `world.set(e, og.Velocity, ...)` refuses on
an entity without one (a body in `start`, before its first step):
`world.insert` it there, which adds and sets it.

`og.CollisionStarted` is read on the tick after the physics step, when
the bodies' `og.Velocity` already holds the solver's answer (they may
already be parting), so never weigh a hit from og.Velocity. The event
carries the hit itself, taken in the step that made the contact. `a` is
the lower id, and `normal` points from `a` to `b`. `point` is the deepest
contact point.
`relativeVelocity` (m/s) is `a`'s velocity relative to `b` at `point` as
that step began: each body's linear velocity plus its spin crossed with
the point's offset from its centre of mass. A static collider counts as
still, and a kinematic body moves at its velocity. Its part along
`normal` is the closing speed:
`Math.max(0, rv.x * n.x + rv.y * n.y + rv.z * n.z)`, which is positive
while they approach. `impulse` (N·s, at least 0) is the normal impulse the
solver gave the pair in that step, summed over substeps and contact points:
`b` took `impulse` along `normal` and `a` the same against it, so heavier
bodies hitting give more. Scale feedback on the closing speed (how fast)
or the impulse (how hard, mass included), as
`examples/rumpus-isle/src/critters.ts` does for its bumps.

```ts
const hit = og.physics.raycast(world, { origin: eye, direction: { x: 0, y: 0, z: -1 }, maxDistance: 30, filter: { mask: 0b10, exclude: player } });
if (hit.ok && hit.value) { /* hit.value.entity, distance, point, normal */ }
const under = og.physics.shapeCast(world, { shape: { shape: SHAPE_SPHERE, radius: 0.4 }, position: feet, direction: { x: 0, y: -1, z: 0 }, maxDistance: 2 });
const near = og.physics.overlap(world, { shape: { shape: SHAPE_BOX, size: { x: 4, y: 2, z: 4 } }, position: door });   // entities, by id
```

Push a dynamic body with `og.physics.applyForce(world, e, force, point?)`
(newtons over the next step: call it every tick for thrust, wind or a
car's downforce), `applyImpulse(world, e, impulse, point?)` (newton-seconds
at once: a kick, a hit, a jump; its velocity changes by impulse / mass),
`applyTorque` and `applyTorqueImpulse` (about each world axis; 2D uses z).
A point off the centre of mass spins it too. They apply at the og.physics
step after the systems, in the order given; a hit stop drops forces and
keeps impulses for after it; those not yet applied are in snapshots.
`og.RigidBody.centerOfMass` moves the centre of mass (a roly-poly toy, a
car that keeps its wheels down).

## 2. Beyond rigid bodies: joints, controllers, vehicles, ropes, cloth, fluids, ragdolls

Inside a system the queries see the colliders as they were at the start of
the tick (`og.physics.queries` records them before the game's systems), so
the answer does not depend on system order; `QueryFilter.includeSensors`
adds sensors. The step itself runs on the lanes (og-build §9) with the same result
as one thread. Beyond rigid bodies, all are components over the same simulation,
deterministic and part of snapshots (`og api show physics`):

- `og.Joint` — fixed, hinge (`JOINT_REVOLUTE`, limits, a velocity or
  position motor), slider, spring, rope, ball; to another body (`other`) or
  a world point (`otherAnchor`); `value` reads the angle or slide.
- `og.CharacterController` (3D, or top-down with `gravityScale` 0) — set
  `move` every tick and `jump` to jump; it climbs `maxSlope` and
  `stepHeight`, snaps to the ground, rides moving platforms, slides along
  walls, pushes dynamic bodies; read `grounded`, `velocity`, `ground`. Its
  og.Collider's `layers` and `mask` decide what blocks it, as for bodies
  (a ghost through a gate: leave the gate's layer out of its `mask`); the
  same for the PlatformerController.
- `og.PlatformerController` (2D) — `moveX`, `jump`, `down` in; jump height
  and time to apex, coyote time, jump buffer, wall jumps, one-way layers,
  corner correction as tunables (og-feel §12 has values by genre);
  `impulse` (m/s, taken once on the next step) knocks it back or launches
  it;
  `og.CharacterEvent` reports jumps and landings for dust and sounds.
  Both controllers push the dynamic bodies they move into (`pushBodies`,
  on by default) with at most `pushStrength` (1,000 N): the push is mass
  times the speed asked of the body, capped, so mass tells. Give a crate
  its `og.RigidBody.mass`: 2 kg is shoved at the run speed, 100 kg creeps,
  a tonne on the ground holds (friction beats the push). Lower
  `pushStrength` for a weak hero; `pushBodies: false` for a hero that
  never shoves (a crate then blocks like a wall).
  `templates/archetypes/runner`'s courier is one.
- `og.Vehicle` — a raycast car (3D, `throttle`, `steer`, `brake`; wheels
  from `og.physics.vehicleWheels`) or a car on wheel joints (2D). Its
  defaults drive a 1,000 kg road car: set the chassis's
  `og.RigidBody.mass` (og-feel §12 has the numbers for a racer, a truck
  and a kart). Its wheel rays meet only what its chassis's og.Collider
  `layers` and `mask` meet, so a ghost car on its own layer drives through
  the player's. `rollInfluence` 0 (the default) never tips it over in a
  bend; `og.RigidBody.centerOfMass` lowers its weight. It writes `speed`,
  `contacts` and `slip` (the most any tyre slides, 0 to 1); each wheel's
  `slip` and `sideSpeed` come with its pose, for smoke, skid marks and a
  squeal.
- `og.Rope` and `og.Cloth` — Verlet, drawn by the engine when `visible`.
  A cloth that folds or piles up (a dropped sheet, a cape on the ground)
  wants `thickness` about its `spacing` so it never passes through
  itself, `substeps` 2–4 and `friction` near 1 so it lies where it lands
  (self-collision costs about 1 ms a tick per 1,000 points).
- `og.Fluid2D` — a particle fluid drawn as metaballs on a sprite layer.
  Give two or more fluids the same `group` and they interact: by
  `density` the heavier sinks under the lighter (oil on water), by
  `mixing` 0 they stay in layers and by 1 they blend, colour and weight
  evening out where they touch; a group draws as one surface. Up to
  sixteen fluids draw a frame (four to a group).
- `og.Fluid3D` — a 3D particle fluid (3D profile) drawn as one
  screen-space surface that refracts and tints what lies behind it,
  reflects the sky and takes the scene's lights. Give it `bounds` (a
  container around the entity) or colliders and terrain to hold it; an
  emitter (`emitRate`, `emitOffset`, `emitVelocity`, up to `maxCount`) pours
  and a sink box (`sinkSize`, `sinkOffset`) drains. Groups work as in 2D:
  `density` 3 under 1 gives honey under water, `mixing` 1 blends their
  colours. Look: `color`, `opacity` per metre, `reflectivity`,
  `refraction`, `smoothing` (0 shows the beads); fluids drawn in one frame
  share one surface and the first fluid's look. Cost is the particles:
  about 0.9 ms of CPU a tick for 2,000 at one substep and 1.8 ms at the
  default two (native and wasm alike), 4 ms for 4,000 and 10 ms for 8,000
  at two, roughly double that when they pile deep, so budget a scene to a
  few thousand; the surface costs about 1.1 ms on the GPU whatever the count,
  growing with the pixels it covers. `og.physics.fluid3dParticles` reads
  the positions (the fx-showcase `fluids3d` station).
- `og.Ragdoll` — give the entity a rig
  (`og.physics.setRagdollRig(world, e, must(og.physics.ragdollRigFromSkeleton(world, anim.skeleton)))`),
  then set `weight` 1 to go limp. At weight 0 its bodies follow the
  animation's pose every tick, so the fall starts from the pose and its
  velocity; the model is drawn blended from the animation to the bones over
  `blendTime`. In a 2D world (`og.Physics.profile` 2) the same works for a
  side-on skinned model whose bones lie in the xy plane: the bones are
  rapier2d bodies on hinges about z (the fx-showcase `ragdoll2d` station).

Readouts (`ropePoints`, `clothPoints`, `fluidParticles`,
`fluid3dParticles`, `waterHeight`, `vehicleWheels`, `ragdollPose`) show
the last step. `templates/fx-showcase` has a station for each.
