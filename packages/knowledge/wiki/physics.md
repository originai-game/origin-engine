<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `physics` module

Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.

The profile is the project's (`profile "2d"` or `"3d"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.

The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.

- **TypeScript**: functions are the namespace `og.physics`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module physics {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 3543</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`BODY_DYNAMIC`](#const-BODY_DYNAMIC) | constant | RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick. |
| [`BODY_STATIC`](#const-BODY_STATIC) | constant | RigidBody.kind: never moves; collides with dynamic bodies. |
| [`BODY_KINEMATIC`](#const-BODY_KINEMATIC) | constant | RigidBody.kind: moved by game code through its Transform; its Velocity, if any, is how fast it pushes. |
| [`LOCK_TRANSLATION_X`](#const-LOCK_TRANSLATION_X) | constant | RigidBody.lock bit: no translation along x. |
| [`LOCK_TRANSLATION_Y`](#const-LOCK_TRANSLATION_Y) | constant | RigidBody.lock bit: no translation along y. |
| [`LOCK_TRANSLATION_Z`](#const-LOCK_TRANSLATION_Z) | constant | RigidBody.lock bit: no translation along z. |
| [`LOCK_ROTATION_X`](#const-LOCK_ROTATION_X) | constant | RigidBody.lock bit: no rotation about x. |
| [`LOCK_ROTATION_Y`](#const-LOCK_ROTATION_Y) | constant | RigidBody.lock bit: no rotation about y. |
| [`LOCK_ROTATION_Z`](#const-LOCK_ROTATION_Z) | constant | RigidBody.lock bit: no rotation about z (the 2D profile's only rotation). |
| [`SHAPE_BOX`](#const-SHAPE_BOX) | constant | Collider.shape: a box of `size` (both profiles). |
| [`SHAPE_SPHERE`](#const-SHAPE_SPHERE) | constant | Collider.shape: a sphere of `radius` (a circle in 2D). |
| [`SHAPE_CAPSULE`](#const-SHAPE_CAPSULE) | constant | Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles). |
| [`SHAPE_CYLINDER`](#const-SHAPE_CYLINDER) | constant | Collider.shape: a cylinder along y of `radius` and `height` (3D only). |
| [`SHAPE_CONVEX`](#const-SHAPE_CONVEX) | constant | Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D). |
| [`SHAPE_TRIMESH`](#const-SHAPE_TRIMESH) | constant | Collider.shape: the triangles of `mesh` (3D only; for static and kinematic bodies). |
| [`SHAPE_HEIGHTFIELD`](#const-SHAPE_HEIGHTFIELD) | constant | Collider.shape: the height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry). |
| [`PROFILE_2D`](#const-PROFILE_2D) | constant | Physics.profile: rapier2d, bodies in the xy plane rotating about z. |
| [`PROFILE_3D`](#const-PROFILE_3D) | constant | Physics.profile: rapier3d. |
| [`og.RigidBody`](#record-RigidBody) | component | A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one. |
| [`og.Collider`](#record-Collider) | component | The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale. |
| [`og.Velocity`](#record-Velocity) | component | Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing), read for kinematic ones; set it to launch or stop a body. |
| [`og.Physics`](#record-Physics) | resource | The physics settings of the world. The profile is the project's; the same components serve both profiles. |
| [`og.CollisionStarted`](#record-CollisionStarted) | event | Two solid colliders started touching during the tick. |
| [`og.CollisionEnded`](#record-CollisionEnded) | event | Two solid colliders stopped touching, or one of them went away. |
| [`og.SensorEntered`](#record-SensorEntered) | event | A collider started overlapping a sensor. |
| [`og.SensorExited`](#record-SensorExited) | event | A collider stopped overlapping a sensor, or one of them went away. |
| [`og.Contact`](#record-Contact) | event | One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now. |
| [`JOINT_FIXED`](#const-JOINT_FIXED) | constant | `Joint.kind`: the two bodies move as one. |
| [`JOINT_REVOLUTE`](#const-JOINT_REVOLUTE) | constant | `Joint.kind`: a hinge about `axis` (2D: about z), with optional limits and a motor. |
| [`JOINT_PRISMATIC`](#const-JOINT_PRISMATIC) | constant | `Joint.kind`: a slider along `axis`, with optional limits and a motor. |
| [`JOINT_SPRING`](#const-JOINT_SPRING) | constant | `Joint.kind`: a damped spring pulling the anchors to `restLength` apart. |
| [`JOINT_ROPE`](#const-JOINT_ROPE) | constant | `Joint.kind`: a rope: the anchors stay at most `length` apart, free inside it. |
| [`JOINT_BALL`](#const-JOINT_BALL) | constant | `Joint.kind`: a ball and socket (3D; the 2D profile treats it as a hinge). |
| [`MOTOR_NONE`](#const-MOTOR_NONE) | constant | `Joint.motor`: no motor. |
| [`MOTOR_VELOCITY`](#const-MOTOR_VELOCITY) | constant | `Joint.motor`: drive towards `motorTarget` speed (rad/s or m/s). |
| [`MOTOR_POSITION`](#const-MOTOR_POSITION) | constant | `Joint.motor`: spring towards `motorTarget` angle or position (a servo). |
| [`og.Joint`](#record-Joint) | component | A joint between two bodies (or a body and a point of the world). The component may sit on one of the bodies (`body` 0) or on an entity of its own, so one body can have several joints. Anchors are in each body's frame (rotated and moved with it, not scaled); without `other` the second anchor is a world point. A joint whose bodies are not both `og.RigidBody` entities does nothing until they are. |
| [`og.CharacterController`](#record-CharacterController) | component | A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the ground, carried by the moving platform it stands on and falling by gravity. The entity needs an og.Transform and an og.Collider; the step makes it a kinematic body and pushes dynamic bodies it walks into. |
| [`og.PlatformerController`](#record-PlatformerController) | component | A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height (releasing jump while rising cuts the jump), apex hang, air jumps, wall slide and wall jump, one-way platforms (colliders on `oneWayLayers` are solid from above only; hold `down` to drop through), moving platforms (it rides what it stands on) and corner correction (a jump that clips a ceiling corner by less than `cornerCorrection` slides past it). The entity needs an og.Transform and an og.Collider (a box or capsule); it becomes a kinematic body. |
| [`CHARACTER_JUMPED`](#const-CHARACTER_JUMPED) | constant | `CharacterEvent.kind`: a jump from the ground (or in coyote time). |
| [`CHARACTER_LANDED`](#const-CHARACTER_LANDED) | constant | `CharacterEvent.kind`: touched the ground after being in the air. |
| [`CHARACTER_WALL_JUMPED`](#const-CHARACTER_WALL_JUMPED) | constant | `CharacterEvent.kind`: a jump off a wall. |
| [`CHARACTER_AIR_JUMPED`](#const-CHARACTER_AIR_JUMPED) | constant | `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps). |
| [`CHARACTER_LEFT_GROUND`](#const-CHARACTER_LEFT_GROUND) | constant | `CharacterEvent.kind`: left the ground without jumping (ran off an edge). |
| [`CHARACTER_BUMPED`](#const-CHARACTER_BUMPED) | constant | `CharacterEvent.kind`: hit a ceiling while rising. |
| [`og.CharacterEvent`](#record-CharacterEvent) | event | Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. Read the next tick. |
| [`DRIVE_REAR`](#const-DRIVE_REAR) | constant | `Vehicle.drive`: the rear wheels drive. |
| [`DRIVE_FRONT`](#const-DRIVE_FRONT) | constant | `Vehicle.drive`: the front wheels drive. |
| [`DRIVE_ALL`](#const-DRIVE_ALL) | constant | `Vehicle.drive`: every wheel drives. |
| [`og.Vehicle`](#record-Vehicle) | component | A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). 3D: a raycast car with four wheels (suspension, grip, steering on the front axle); its forward is the chassis's local +z (three.js Object3D.lookAt aims +z). 2D: two wheel bodies on sprung wheel joints (front at +x), driven by a motor. The wheels are not entities: og.physics.vehicleWheels gives their poses for drawing. |
| [`og.Rope`](#record-Rope) | component | A Verlet rope: `segments` links from the entity's position (plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind, colliding with every solid collider at `radius`. It moves nothing (one-way: a rope joint ties bodies together, og.Joint JOINT_ROPE). Its points are world state (the og.physics state block); og.physics.ropePoints reads them and the renderer draws the rope as a strip of `width` and `color`. |
| [`PIN_TOP`](#const-PIN_TOP) | constant | `Cloth.pins` bit: the whole top row is pinned. |
| [`PIN_TOP_CORNERS`](#const-PIN_TOP_CORNERS) | constant | `Cloth.pins` bit: the top-left and top-right corners are pinned. |
| [`PIN_LEFT`](#const-PIN_LEFT) | constant | `Cloth.pins` bit: the whole left column is pinned (a flag on a pole). |
| [`PIN_TOP_RINGS`](#const-PIN_TOP_RINGS) | constant | `Cloth.pins` bit: every fourth point of the top row (curtain rings). |
| [`og.Cloth`](#record-Cloth) | component | A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's position (its top-left corner, the grid laid out along the entity's local x and down -y), pinned by `pins` (pinned points follow the entity), with stretch and shear constraints, gravity, wind, collision against every solid collider and, with `thickness`, against itself. One-way like og.Rope; its points are world state. Drawn as a double-sided mesh with `material` (og.render.standardMaterial; 0: a plain cloth colour). |
| [`og.Fluid2D`](#record-Fluid2D) | component | A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure, viscosity) on a fixed grid in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider (pushed out of bodies; they push nothing back) and stay inside `bounds` around the entity when it is set. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: the particles blurred into a field the renderer thresholds into one surface of `color`, on sprite layer `layer`. |
| [`og.Fluid3D`](#record-Fluid3D) | component | A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure that gives surface tension, viscosity) over a spatial hash in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider, the terrain's height field included (pushed out; they push nothing back), and stay inside `bounds` around the entity when it is set. An emitter adds particles at `emitRate` up to `maxCount`; a sink box removes those that enter it. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as a screen-space surface: the particles' depth smoothed into one surface, refracting what is behind by its thickness and reflecting the sky, lit by the scene's lights. Cost: about 0.9 ms of CPU a tick for 2000 particles at 1 substep, 1.8 ms at 2 (native and wasm alike), growing with the particles; keep a scene's fluids to a few thousand particles. The surface costs about 1.1 ms of GPU (an Apple M4 Pro, the fluid covering a third of a 1280×720 frame) whatever the particle count, growing with the pixels it covers. |
| [`og.WaterSurface`](#record-WaterSurface) | component | Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies crossing it push waves (splashes; og.Splash events) and float by `buoyancy`. The heights are world state; an og.Water on the same entity draws them. |
| [`og.WaterLine`](#record-WaterLine) | component | 2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and passing waves to its neighbours by `spread`. Bodies (and characters) crossing the surface splash it (og.Splash events) and float by `buoyancy`. The springs are world state; an og.Water2D on the same entity draws it. |
| [`og.Splash`](#record-Splash) | event | Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. Read the next tick. |
| [`og.Ragdoll`](#record-Ragdoll) | component | A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy plane at their animated z, for a skeleton laid out in that plane), built from the skeleton's rest pose. `weight` is where the pose comes from: 0 the animation (the bodies follow it kinematically), 1 the physics (the bodies are dynamic, starting with the animation's velocity); the pose drawn blends between them over `blendTime`, both ways. The bodies are world state (the og.physics state block), not entities. |
| [`QueryFilter`](#type-QueryFilter) | desc struct | Which colliders a scene query considers. The zero value considers every solid collider on every layer. |
| [`RaycastDesc`](#type-RaycastDesc) | desc struct | A ray for og.physics.raycast. |
| [`RaycastHit`](#type-RaycastHit) | struct | The nearest collider a ray meets. |
| [`ShapeDesc`](#type-ShapeDesc) | desc struct | The shape of a shape cast or an overlap test: the shape fields of a Collider. The zero value is a 1 m box. |
| [`ShapeCastDesc`](#type-ShapeCastDesc) | desc struct | A shape swept along a direction for og.physics.shapeCast. |
| [`ShapeCastHit`](#type-ShapeCastHit) | struct | The first collider a swept shape meets. |
| [`OverlapDesc`](#type-OverlapDesc) | desc struct | A shape placed in the world for og.physics.overlap. |
| [`MeshDesc`](#type-MeshDesc) | desc struct | Geometry for convex and triangle-mesh colliders, in the mesh's own space. |
| [`og.physics.raycast`](#fn-raycast) | function | The nearest collider along a ray. Inside a system it sees the colliders as they were at the start of the tick (og.physics.queries records them before the game's systems); between ticks, as the world holds them. |
| [`og.physics.shapeCast`](#fn-shapeCast) | function | The first collider a shape meets when swept along a direction; the same view of the colliders as raycast. |
| [`og.physics.overlap`](#fn-overlap) | function | Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast. |
| [`og.physics.setMesh`](#fn-setMesh) | function | Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. Loaded glTF geometries are registered under their own handles already; meshes are assets, not world state. |
| [`WheelPose`](#type-WheelPose) | struct | A wheel of an og.Vehicle as the last step left it, for drawing. |
| [`RagdollBoneDesc`](#type-RagdollBoneDesc) | desc struct | One bone of a ragdoll rig: a capsule from the bone's position along `axis` for `length`. |
| [`RagdollRigDesc`](#type-RagdollRigDesc) | desc struct | The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton. |
| [`BonePose`](#type-BonePose) | struct | A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight). |
| [`og.physics.ropePoints`](#fn-ropePoints) | function | An og.Rope's points after the last step, from its entity to its end (empty before its first step). The engine draws visible ropes itself; this is for game logic and custom drawing. |
| [`og.physics.clothPoints`](#fn-clothPoints) | function | An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge). |
| [`og.physics.fluidParticles`](#fn-fluidParticles) | function | An og.Fluid2D's particle positions after the last step, in world space. |
| [`og.physics.fluid3dParticles`](#fn-fluid3dParticles) | function | An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared). |
| [`og.physics.waterHeight`](#fn-waterHeight) | function | The surface height of an og.WaterSurface (world y) or an og.WaterLine (world y at `x`; `z` ignored) at a point, as the last step left it: for boats that bob and spray that starts on the surface. |
| [`og.physics.vehicleWheels`](#fn-vehicleWheels) | function | An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right), for drawing wheel models. |
| [`og.physics.ragdollRigFromSkeleton`](#fn-ragdollRigFromSkeleton) | function | A ragdoll rig from a loaded skeleton (og.load.animation's `skeleton`): a bone per joint of its first skin, parents first, each from its joint's rest pose to its first child joint, every bone naming its joint and the rig naming the skeleton. Pass it to og.physics.setRagdollRig, edited or not. |
| [`og.physics.setRagdollRig`](#fn-setRagdollRig) | function | Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ragdoll.blend). While weight is 0 the bones follow the rig's rest pose at the entity. Rigs are assets, not world state: kept by the engine and given again to a reloaded world. |
| [`og.physics.ragdollPose`](#fn-ragdollPose) | function | An og.Ragdoll's bones after the last step, in rig order (empty before its first step). |

## Functions

<a id="fn-raycast"></a>

### `og.physics.raycast`

The nearest collider along a ray. Inside a system it sees the colliders as they were at the start of the tick (og.physics.queries records them before the game's systems); between ticks, as the world holds them.

```ts
og.physics.raycast(world: World, desc?: RaycastDesc): Result<RaycastHit | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RaycastDesc` | yes | The ray and the filter. |

**Returns** `Result<RaycastHit | null>`: The hit, or null when nothing is hit; refused for a zero direction.

Binding symbol `og_physics_raycast` · publish builds: kept

```ogt
fn raycast {
  doc "The nearest collider along a ray. Inside a system it sees the colliders as they were at the start of the tick (og.physics.queries records them before the game's systems); between ticks, as the world holds them."
  param world World "The world."
  param desc RaycastDesc "The ray and the filter."
  returns Result.Option.RaycastHit "The hit, or null when nothing is hit; refused for a zero direction."
}
```

<sub>`api.ogt` line 4832</sub>

<a id="fn-shapeCast"></a>

### `og.physics.shapeCast`

The first collider a shape meets when swept along a direction; the same view of the colliders as raycast.

```ts
og.physics.shapeCast(world: World, desc?: ShapeCastDesc): Result<ShapeCastHit | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ShapeCastDesc` | yes | The shape, its sweep and the filter. |

**Returns** `Result<ShapeCastHit | null>`: The hit, or null; refused for a zero direction or an unknown mesh.

Binding symbol `og_physics_shape_cast` · publish builds: kept

```ogt
fn shapeCast {
  doc "The first collider a shape meets when swept along a direction; the same view of the colliders as raycast."
  param world World "The world."
  param desc ShapeCastDesc "The shape, its sweep and the filter."
  returns Result.Option.ShapeCastHit "The hit, or null; refused for a zero direction or an unknown mesh."
}
```

<sub>`api.ogt` line 4839</sub>

<a id="fn-overlap"></a>

### `og.physics.overlap`

Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast.

```ts
og.physics.overlap(world: World, desc?: OverlapDesc): Result<readonly Entity[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `OverlapDesc` | yes | The shape, its placement and the filter. |

**Returns** `Result<readonly Entity[]>`.

Binding symbol `og_physics_overlap` · publish builds: kept

```ogt
fn overlap {
  doc "Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast."
  param world World "The world."
  param desc OverlapDesc "The shape, its placement and the filter."
  returns Result.List.Entity
}
```

<sub>`api.ogt` line 4846</sub>

<a id="fn-setMesh"></a>

### `og.physics.setMesh`

Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. Loaded glTF geometries are registered under their own handles already; meshes are assets, not world state.

```ts
og.physics.setMesh(world: World, mesh: Geometry, desc?: MeshDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Geometry` | no | The handle colliders name in Collider.mesh. |
| `desc` | `MeshDesc` | yes | Positions and triangles. |

**Returns** `Result<void>`: Refused for an empty, non-finite or out-of-range mesh.

Binding symbol `og_physics_set_mesh` · publish builds: kept

```ogt
fn setMesh {
  doc "Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. Loaded glTF geometries are registered under their own handles already; meshes are assets, not world state."
  param world World "The world."
  param mesh Geometry "The handle colliders name in Collider.mesh."
  param desc MeshDesc "Positions and triangles."
  returns Result.Void "Refused for an empty, non-finite or out-of-range mesh."
}
```

<sub>`api.ogt` line 4853</sub>

<a id="fn-ropePoints"></a>

### `og.physics.ropePoints`

An og.Rope's points after the last step, from its entity to its end (empty before its first step). The engine draws visible ropes itself; this is for game logic and custom drawing.

```ts
og.physics.ropePoints(world: World, entity: Entity): Result<readonly Vector3[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The rope's entity. |

**Returns** `Result<readonly Vector3[]>`: Refused for an entity without an og.Rope.

Binding symbol `og_physics_rope_points` · publish builds: kept

```ogt
fn ropePoints {
  doc "An og.Rope's points after the last step, from its entity to its end (empty before its first step). The engine draws visible ropes itself; this is for game logic and custom drawing."
  param world World "The world."
  param entity Entity "The rope's entity."
  returns Result.List.Vector3 "Refused for an entity without an og.Rope."
}
```

<sub>`api.ogt` line 4907</sub>

<a id="fn-clothPoints"></a>

### `og.physics.clothPoints`

An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge).

```ts
og.physics.clothPoints(world: World, entity: Entity): Result<readonly Vector3[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The cloth's entity. |

**Returns** `Result<readonly Vector3[]>`: Refused for an entity without an og.Cloth.

Binding symbol `og_physics_cloth_points` · publish builds: kept

```ogt
fn clothPoints {
  doc "An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge)."
  param world World "The world."
  param entity Entity "The cloth's entity."
  returns Result.List.Vector3 "Refused for an entity without an og.Cloth."
}
```

<sub>`api.ogt` line 4914</sub>

<a id="fn-fluidParticles"></a>

### `og.physics.fluidParticles`

An og.Fluid2D's particle positions after the last step, in world space.

```ts
og.physics.fluidParticles(world: World, entity: Entity): Result<readonly Vector2[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The fluid's entity. |

**Returns** `Result<readonly Vector2[]>`: Refused for an entity without an og.Fluid2D.

Binding symbol `og_physics_fluid_particles` · publish builds: kept

```ogt
fn fluidParticles {
  doc "An og.Fluid2D's particle positions after the last step, in world space."
  param world World "The world."
  param entity Entity "The fluid's entity."
  returns Result.List.Vector2 "Refused for an entity without an og.Fluid2D."
}
```

<sub>`api.ogt` line 4921</sub>

<a id="fn-fluid3dParticles"></a>

### `og.physics.fluid3dParticles`

An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared).

```ts
og.physics.fluid3dParticles(world: World, entity: Entity): Result<readonly Vector3[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The fluid's entity. |

**Returns** `Result<readonly Vector3[]>`: Refused for an entity without an og.Fluid3D.

Binding symbol `og_physics_fluid3d_particles` · publish builds: kept

```ogt
fn fluid3dParticles {
  doc "An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared)."
  param world World "The world."
  param entity Entity "The fluid's entity."
  returns Result.List.Vector3 "Refused for an entity without an og.Fluid3D."
}
```

<sub>`api.ogt` line 4928</sub>

<a id="fn-waterHeight"></a>

### `og.physics.waterHeight`

The surface height of an og.WaterSurface (world y) or an og.WaterLine (world y at `x`; `z` ignored) at a point, as the last step left it: for boats that bob and spray that starts on the surface.

```ts
og.physics.waterHeight(world: World, entity: Entity, x: number, z: number): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The water's entity. |
| `x` | `number` | no | World x, in metres. |
| `z` | `number` | no | World z, in metres (3D). |

**Returns** `Result<number>`: Refused for an entity without water; outside the water, its rest level.

Binding symbol `og_physics_water_height` · publish builds: kept

```ogt
fn waterHeight {
  doc "The surface height of an og.WaterSurface (world y) or an og.WaterLine (world y at `x`; `z` ignored) at a point, as the last step left it: for boats that bob and spray that starts on the surface."
  param world World "The world."
  param entity Entity "The water's entity."
  param x f32 "World x, in metres."
  param z f32 "World z, in metres (3D)."
  returns Result.f32 "Refused for an entity without water; outside the water, its rest level."
}
```

<sub>`api.ogt` line 4935</sub>

<a id="fn-vehicleWheels"></a>

### `og.physics.vehicleWheels`

An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right), for drawing wheel models.

```ts
og.physics.vehicleWheels(world: World, entity: Entity): Result<readonly WheelPose[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The vehicle's entity. |

**Returns** `Result<readonly WheelPose[]>`: Refused for an entity without a vehicle.

Binding symbol `og_physics_vehicle_wheels` · publish builds: kept

```ogt
fn vehicleWheels {
  doc "An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right), for drawing wheel models."
  param world World "The world."
  param entity Entity "The vehicle's entity."
  returns Result.List.WheelPose "Refused for an entity without a vehicle."
}
```

<sub>`api.ogt` line 4944</sub>

<a id="fn-ragdollRigFromSkeleton"></a>

### `og.physics.ragdollRigFromSkeleton`

A ragdoll rig from a loaded skeleton (og.load.animation's `skeleton`): a bone per joint of its first skin, parents first, each from its joint's rest pose to its first child joint, every bone naming its joint and the rig naming the skeleton. Pass it to og.physics.setRagdollRig, edited or not.

```ts
og.physics.ragdollRigFromSkeleton(world: World, skeleton: SkeletonAsset): Result<RagdollRigDesc>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `skeleton` | `SkeletonAsset` | no | The skeleton. |

**Returns** `Result<RagdollRigDesc>`: Refused for a skeleton that is not loaded or has no skin.

Binding symbol `og_physics_ragdoll_rig_from_skeleton` · publish builds: kept

```ogt
fn ragdollRigFromSkeleton {
  doc "A ragdoll rig from a loaded skeleton (og.load.animation's `skeleton`): a bone per joint of its first skin, parents first, each from its joint's rest pose to its first child joint, every bone naming its joint and the rig naming the skeleton. Pass it to og.physics.setRagdollRig, edited or not."
  param world World "The world."
  param skeleton SkeletonAsset "The skeleton."
  returns Result.RagdollRigDesc "Refused for a skeleton that is not loaded or has no skin."
}
```

<sub>`api.ogt` line 4951</sub>

<a id="fn-setRagdollRig"></a>

### `og.physics.setRagdollRig`

Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ragdoll.blend). While weight is 0 the bones follow the rig's rest pose at the entity. Rigs are assets, not world state: kept by the engine and given again to a reloaded world.

```ts
og.physics.setRagdollRig(world: World, entity: Entity, desc?: RagdollRigDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The ragdoll's entity. |
| `desc` | `RagdollRigDesc` | yes | The bones, parents first. |

**Returns** `Result<void>`: Refused for an empty rig or a parent that does not come before its child.

Binding symbol `og_physics_set_ragdoll_rig` · publish builds: kept

```ogt
fn setRagdollRig {
  doc "Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ragdoll.blend). While weight is 0 the bones follow the rig's rest pose at the entity. Rigs are assets, not world state: kept by the engine and given again to a reloaded world."
  param world World "The world."
  param entity Entity "The ragdoll's entity."
  param desc RagdollRigDesc "The bones, parents first."
  returns Result.Void "Refused for an empty rig or a parent that does not come before its child."
}
```

<sub>`api.ogt` line 4958</sub>

<a id="fn-ragdollPose"></a>

### `og.physics.ragdollPose`

An og.Ragdoll's bones after the last step, in rig order (empty before its first step).

```ts
og.physics.ragdollPose(world: World, entity: Entity): Result<readonly BonePose[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The ragdoll's entity. |

**Returns** `Result<readonly BonePose[]>`: Refused for an entity without a ragdoll.

Binding symbol `og_physics_ragdoll_pose` · publish builds: kept

```ogt
fn ragdollPose {
  doc "An og.Ragdoll's bones after the last step, in rig order (empty before its first step)."
  param world World "The world."
  param entity Entity "The ragdoll's entity."
  returns Result.List.BonePose "Refused for an entity without a ragdoll."
}
```

<sub>`api.ogt` line 4966</sub>

## Components, resources and events

<a id="record-RigidBody"></a>

### `og.RigidBody` (component)

A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one.

Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport).

Token `og.RigidBody`, registered under the schema name `og.RigidBody`; its value type is `RigidBody`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | 0 … 2 | — | label “Kind” | BODY_DYNAMIC (0, simulated and written back), BODY_STATIC (1, never moves) or BODY_KINEMATIC (2, moved by game code through its Transform). |
| `mass` | `number` | `0` | ≥ 0 | `kg` | tunable | Mass; 0 derives it from the collider's density and volume. |
| `linearDamping` | `number` | `0` | 0 … 100 | `perSecond` | tunable | How fast linear velocity decays (0: none). |
| `angularDamping` | `number` | `0` | 0 … 100 | `perSecond` | tunable | How fast angular velocity decays (0: none). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies Physics.gravity for this body: 0 floats, 1 is normal. |
| `lock` | `number` | `0` | 0 … 63 | — | — | Locked axes, a sum of the LOCK_* bits: 1, 2, 4 no translation along x, y, z; 8, 16, 32 no rotation about x, y, z (2D uses 1, 2 and 32). |
| `ccd` | `number` | `0` | 0 … 1 | — | — | 1 enables continuous collision detection, for fast small bodies that must not tunnel; 0 off. |

```ogt
component RigidBody {
  doc "A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one."
  doc "Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport)."
  field kind u32 "BODY_DYNAMIC (0, simulated and written back), BODY_STATIC (1, never moves) or BODY_KINEMATIC (2, moved by game code through its Transform)." {
    min 0
    max 2
    label "Kind"
  }
  field mass f32 "Mass; 0 derives it from the collider's density and volume." {
    min 0
    unit kg
    tunable true
  }
  field linearDamping f32 "How fast linear velocity decays (0: none)." {
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field angularDamping f32 "How fast angular velocity decays (0: none)." {
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field gravityScale f32 "Multiplies Physics.gravity for this body: 0 floats, 1 is normal." {
    default 1
    min -10
    max 10
    tunable true
  }
  field lock u32 "Locked axes, a sum of the LOCK_* bits: 1, 2, 4 no translation along x, y, z; 8, 16, 32 no rotation about x, y, z (2D uses 1, 2 and 32)." {
    min 0
    max 63
  }
  field ccd u32 "1 enables continuous collision detection, for fast small bodies that must not tunnel; 0 off." {
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 3568</sub>

<a id="record-Collider"></a>

### `og.Collider` (component)

The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.

Token `og.Collider`, registered under the schema name `og.Collider`; its value type is `Collider`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `number` | `0` | 0 … 6 | — | label “Shape” | One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (3D), 6 the height field `mesh` (3D: a terrain). |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | ≥ 0 | `m` | tunable | Full extents of a box, as three.js BoxGeometry(width, height, depth). |
| `radius` | `number` | `0.5` | ≥ 0 | `m` | tunable | Radius of a sphere, circle, capsule or cylinder. |
| `height` | `number` | `1` | ≥ 0 | `m` | tunable | A capsule's distance between its cap centres (three.js CapsuleGeometry length); a cylinder's height. |
| `mesh` | `Geometry` | `0` | — | — | — | The geometry whose vertices a convex or triangle-mesh shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh; for SHAPE_HEIGHTFIELD, a terrain's height field (og.terrain.load's TerrainInfo.geometry). |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Position of the shape relative to the entity. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Rotation of the shape relative to the entity. |
| `friction` | `number` | `0.5` | 0 … 2 | — | tunable | Coulomb friction (0 is ice, 1 rubber); a pair uses the mean of both colliders. |
| `restitution` | `number` | `0` | 0 … 1 | — | tunable | Bounciness (0 no bounce, 1 elastic); a pair uses the mean of both colliders. |
| `density` | `number` | `1` | ≥ 0 | `kgPerM3` | tunable | Density, used when RigidBody.mass is 0. |
| `sensor` | `number` | `0` | 0 … 1 | — | — | 1 makes this a sensor: it reports SensorEntered and SensorExited and pushes nothing; 0 a solid. |
| `layers` | `number` | `1` | — | — | — | The collision layers this collider is on, as bits. |
| `mask` | `number` | `4294967295` | — | — | — | The layers it collides with, as bits. Two colliders interact when each one's layers meet the other's mask. |

```ogt
component Collider {
  doc "The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale."
  field shape u32 "One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (3D), 6 the height field `mesh` (3D: a terrain)." {
    min 0
    max 6
    label "Shape"
  }
  field size Vector3 "Full extents of a box, as three.js BoxGeometry(width, height, depth)." {
    default 1 1 1
    min 0
    unit m
    tunable true
  }
  field radius f32 "Radius of a sphere, circle, capsule or cylinder." {
    default 0.5
    min 0
    unit m
    tunable true
  }
  field height f32 "A capsule's distance between its cap centres (three.js CapsuleGeometry length); a cylinder's height." {
    default 1
    min 0
    unit m
    tunable true
  }
  field mesh Geometry "The geometry whose vertices a convex or triangle-mesh shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh; for SHAPE_HEIGHTFIELD, a terrain's height field (og.terrain.load's TerrainInfo.geometry)."
  field offset Vector3 "Position of the shape relative to the entity." {
    unit m
  }
  field rotation Quaternion "Rotation of the shape relative to the entity."
  field friction f32 "Coulomb friction (0 is ice, 1 rubber); a pair uses the mean of both colliders." {
    default 0.5
    min 0
    max 2
    tunable true
  }
  field restitution f32 "Bounciness (0 no bounce, 1 elastic); a pair uses the mean of both colliders." {
    min 0
    max 1
    tunable true
  }
  field density f32 "Density, used when RigidBody.mass is 0." {
    default 1
    min 0
    unit kgPerM3
    tunable true
  }
  field sensor u32 "1 makes this a sensor: it reports SensorEntered and SensorExited and pushes nothing; 0 a solid." {
    min 0
    max 1
  }
  field layers u32 "The collision layers this collider is on, as bits." {
    default 1
  }
  field mask u32 "The layers it collides with, as bits. Two colliders interact when each one's layers meet the other's mask." {
    default 4294967295
  }
}
```

<sub>`api.ogt` line 3609</sub>

<a id="record-Velocity"></a>

### `og.Velocity` (component)

Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing), read for kinematic ones; set it to launch or stop a body.

Token `og.Velocity`, registered under the schema name `og.Velocity`; its value type is `Velocity`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `linear` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | The linear velocity (2D uses x and y). |
| `angular` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `radps` | tunable | The angular velocity about each axis (2D uses z). |

```ogt
component Velocity {
  doc "Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing), read for kinematic ones; set it to launch or stop a body."
  field linear Vector3 "The linear velocity (2D uses x and y)." {
    unit mps
    tunable true
  }
  field angular Vector3 "The angular velocity about each axis (2D uses z)." {
    unit radps
    tunable true
  }
}
```

<sub>`api.ogt` line 3668</sub>

<a id="record-Physics"></a>

### `og.Physics` (resource)

The physics settings of the world. The profile is the project's; the same components serve both profiles.

Token `og.Physics`, registered under the schema name `og.Physics`; its value type is `Physics`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `profile` | `number` | `3` | 2 … 3 | — | label “Profile” | PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt. Set the other one at run time to change profile: the next step builds that profile's simulation from the components (contacts, joints' warm starts, ropes, cloth, fluids and ragdoll bodies start again), so a 3D game can play a 2D scene and back. |
| `gravity` | `Vector3` | `{ x: 0, y: -9.81, z: 0 }` | — | `mps2` | tunable | The acceleration of every dynamic body, times its gravityScale; 2D uses x and y. |
| `substeps` | `number` | `1` | 1 … 16 | — | tunable | Physics steps per tick, each of dt / substeps: more is stiffer stacks and less tunnelling, at more cost. |
| `solverIterations` | `number` | `4` | 1 … 32 | — | tunable | Solver iterations per step. |
| `restSpeed` | `number` | `0.02` | 0 … 1 | `mps` | tunable | A dynamic body touching something, supported with at most 2 mm of overlap and slower than this before and after a step, keeps its pose and loses its residual velocity (the same number in m/s and rad/s); 0 disables it. |

```ogt
resource Physics {
  doc "The physics settings of the world. The profile is the project's; the same components serve both profiles."
  field profile u32 "PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt. Set the other one at run time to change profile: the next step builds that profile's simulation from the components (contacts, joints' warm starts, ropes, cloth, fluids and ragdoll bodies start again), so a 3D game can play a 2D scene and back." {
    default 3
    min 2
    max 3
    label "Profile"
  }
  field gravity Vector3 "The acceleration of every dynamic body, times its gravityScale; 2D uses x and y." {
    default 0 -9.81 0
    unit mps2
    tunable true
  }
  field substeps u32 "Physics steps per tick, each of dt / substeps: more is stiffer stacks and less tunnelling, at more cost." {
    default 1
    min 1
    max 16
    tunable true
  }
  field solverIterations u32 "Solver iterations per step." {
    default 4
    min 1
    max 32
    tunable true
  }
  field restSpeed f32 "A dynamic body touching something, supported with at most 2 mm of overlap and slower than this before and after a step, keeps its pose and loses its residual velocity (the same number in m/s and rad/s); 0 disables it." {
    default 0.02
    min 0
    max 1
    unit mps
    tunable true
  }
}
```

<sub>`api.ogt` line 3680</sub>

<a id="record-CollisionStarted"></a>

### `og.CollisionStarted` (event)

Two solid colliders started touching during the tick.

Token `og.CollisionStarted`, registered under the schema name `og.CollisionStarted`; its value type is `CollisionStarted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `a` | `Entity` | `0` | — | — | — | The entity with the lower id. |
| `b` | `Entity` | `0` | — | — | — | The entity with the higher id. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The deepest contact point in world space (z is 0 in 2D). |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The contact normal, from a towards b. |

```ogt
event CollisionStarted {
  doc "Two solid colliders started touching during the tick."
  field a Entity "The entity with the lower id."
  field b Entity "The entity with the higher id."
  field point Vector3 "The deepest contact point in world space (z is 0 in 2D)."
  field normal Vector3 "The contact normal, from a towards b."
}
```

<sub>`api.ogt` line 3714</sub>

<a id="record-CollisionEnded"></a>

### `og.CollisionEnded` (event)

Two solid colliders stopped touching, or one of them went away.

Token `og.CollisionEnded`, registered under the schema name `og.CollisionEnded`; its value type is `CollisionEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `a` | `Entity` | `0` | — | — | — | The entity with the lower id. |
| `b` | `Entity` | `0` | — | — | — | The entity with the higher id. |

```ogt
event CollisionEnded {
  doc "Two solid colliders stopped touching, or one of them went away."
  field a Entity "The entity with the lower id."
  field b Entity "The entity with the higher id."
}
```

<sub>`api.ogt` line 3722</sub>

<a id="record-SensorEntered"></a>

### `og.SensorEntered` (event)

A collider started overlapping a sensor.

Token `og.SensorEntered`, registered under the schema name `og.SensorEntered`; its value type is `SensorEntered`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sensor` | `Entity` | `0` | — | — | — | The entity whose collider is the sensor. |
| `other` | `Entity` | `0` | — | — | — | The entity that entered it. |

```ogt
event SensorEntered {
  doc "A collider started overlapping a sensor."
  field sensor Entity "The entity whose collider is the sensor."
  field other Entity "The entity that entered it."
}
```

<sub>`api.ogt` line 3728</sub>

<a id="record-SensorExited"></a>

### `og.SensorExited` (event)

A collider stopped overlapping a sensor, or one of them went away.

Token `og.SensorExited`, registered under the schema name `og.SensorExited`; its value type is `SensorExited`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sensor` | `Entity` | `0` | — | — | — | The entity whose collider is the sensor. |
| `other` | `Entity` | `0` | — | — | — | The entity that left it. |

```ogt
event SensorExited {
  doc "A collider stopped overlapping a sensor, or one of them went away."
  field sensor Entity "The entity whose collider is the sensor."
  field other Entity "The entity that left it."
}
```

<sub>`api.ogt` line 3734</sub>

<a id="record-Contact"></a>

### `og.Contact` (event)

One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now.

Token `og.Contact`, registered under the schema name `og.Contact`; its value type is `Contact`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `a` | `Entity` | `0` | — | — | — | The entity with the lower id. |
| `b` | `Entity` | `0` | — | — | — | The entity with the higher id. |
| `sensor` | `number` | `0` | — | — | — | 0 for two solids; an overlap with a sensor otherwise: 1 a is the sensor, 2 b is, 3 both are. |

```ogt
event Contact {
  doc "One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now."
  field a Entity "The entity with the lower id."
  field b Entity "The entity with the higher id."
  field sensor u32 "0 for two solids; an overlap with a sensor otherwise: 1 a is the sensor, 2 b is, 3 both are."
}
```

<sub>`api.ogt` line 3740</sub>

<a id="record-Joint"></a>

### `og.Joint` (component)

A joint between two bodies (or a body and a point of the world). The component may sit on one of the bodies (`body` 0) or on an entity of its own, so one body can have several joints. Anchors are in each body's frame (rotated and moved with it, not scaled); without `other` the second anchor is a world point. A joint whose bodies are not both `og.RigidBody` entities does nothing until they are.

Token `og.Joint`, registered under the schema name `og.Joint`; its value type is `Joint`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | 0 … 5 | — | label “Kind” | JOINT_FIXED 0, JOINT_REVOLUTE 1 (hinge), JOINT_PRISMATIC 2 (slider), JOINT_SPRING 3, JOINT_ROPE 4, JOINT_BALL 5. |
| `body` | `Entity` | `0` | — | — | — | The first body; 0 is this entity. |
| `other` | `Entity` | `0` | — | — | — | The second body; 0 joins the first one to the world at `otherAnchor`. |
| `anchor` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the joint sits on the first body, in its frame. |
| `otherAnchor` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it sits on the second body, in its frame (a world point without `other`). |
| `axis` | `Vector3` | `{ x: 1, y: 0, z: 0 }` | — | — | — | The hinge or slide axis in the first body's frame (a hinge in 2D turns about z). |
| `limits` | `boolean` | `false` | — | — | — | Whether `min`..`max` bound the hinge angle (rad) or the slide (m). |
| `min` | `number` | `0` | — | — | — | The lower limit: radians for a hinge, metres for a slider. |
| `max` | `number` | `0` | — | — | — | The upper limit. |
| `motor` | `number` | `0` | 0 … 2 | — | — | MOTOR_NONE 0, MOTOR_VELOCITY 1 (drive at motorTarget speed), MOTOR_POSITION 2 (spring to motorTarget). |
| `motorTarget` | `number` | `0` | — | — | tunable | The motor's target speed (rad/s, m/s) or angle or position (rad, m). |
| `motorStiffness` | `number` | `50` | ≥ 0 | — | tunable | A position motor's stiffness; a velocity motor's strength. |
| `motorDamping` | `number` | `5` | ≥ 0 | — | tunable | A position motor's damping. |
| `motorMaxForce` | `number` | `0` | ≥ 0 | — | tunable | The most force (or torque) the motor applies; 0 is unbounded. |
| `restLength` | `number` | `1` | ≥ 0 | `m` | tunable | A spring's rest length. |
| `stiffness` | `number` | `40` | ≥ 0 | — | tunable | A spring's stiffness (N/m). |
| `damping` | `number` | `2` | ≥ 0 | — | tunable | A spring's damping (N·s/m). |
| `length` | `number` | `2` | ≥ 0 | `m` | tunable | A rope's longest distance between the anchors. |
| `collide` | `boolean` | `false` | — | — | — | Whether the two bodies still collide with each other. |
| `value` | `number` | `0` | — | — | read only | Written: the hinge angle (rad) or the slide (m) after the step; the anchors' distance for springs and ropes. |

```ogt
component Joint {
  doc "A joint between two bodies (or a body and a point of the world). The component may sit on one of the bodies (`body` 0) or on an entity of its own, so one body can have several joints. Anchors are in each body's frame (rotated and moved with it, not scaled); without `other` the second anchor is a world point. A joint whose bodies are not both `og.RigidBody` entities does nothing until they are."
  field kind u32 "JOINT_FIXED 0, JOINT_REVOLUTE 1 (hinge), JOINT_PRISMATIC 2 (slider), JOINT_SPRING 3, JOINT_ROPE 4, JOINT_BALL 5." {
    min 0
    max 5
    label "Kind"
  }
  field body Entity "The first body; 0 is this entity."
  field other Entity "The second body; 0 joins the first one to the world at `otherAnchor`."
  field anchor Vector3 "Where the joint sits on the first body, in its frame." {
    unit m
  }
  field otherAnchor Vector3 "Where it sits on the second body, in its frame (a world point without `other`)." {
    unit m
  }
  field axis Vector3 "The hinge or slide axis in the first body's frame (a hinge in 2D turns about z)." {
    default 1 0 0
  }
  field limits bool "Whether `min`..`max` bound the hinge angle (rad) or the slide (m)."
  field min f32 "The lower limit: radians for a hinge, metres for a slider."
  field max f32 "The upper limit."
  field motor u32 "MOTOR_NONE 0, MOTOR_VELOCITY 1 (drive at motorTarget speed), MOTOR_POSITION 2 (spring to motorTarget)." {
    min 0
    max 2
  }
  field motorTarget f32 "The motor's target speed (rad/s, m/s) or angle or position (rad, m)." {
    tunable true
  }
  field motorStiffness f32 "A position motor's stiffness; a velocity motor's strength." {
    default 50
    min 0
    tunable true
  }
  field motorDamping f32 "A position motor's damping." {
    default 5
    min 0
    tunable true
  }
  field motorMaxForce f32 "The most force (or torque) the motor applies; 0 is unbounded." {
    min 0
    tunable true
  }
  field restLength f32 "A spring's rest length." {
    default 1
    min 0
    unit m
    tunable true
  }
  field stiffness f32 "A spring's stiffness (N/m)." {
    default 40
    min 0
    tunable true
  }
  field damping f32 "A spring's damping (N·s/m)." {
    default 2
    min 0
    tunable true
  }
  field length f32 "A rope's longest distance between the anchors." {
    default 2
    min 0
    unit m
    tunable true
  }
  field collide bool "Whether the two bodies still collide with each other."
  field value f32 "Written: the hinge angle (rad) or the slide (m) after the step; the anchors' distance for springs and ropes." {
    readonly true
  }
}
```

<sub>`api.ogt` line 3757</sub>

<a id="record-CharacterController"></a>

### `og.CharacterController` (component)

A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the ground, carried by the moving platform it stands on and falling by gravity. The entity needs an og.Transform and an og.Collider; the step makes it a kinematic body and pushes dynamic bodies it walks into.

Token `og.CharacterController`, registered under the schema name `og.CharacterController`; its value type is `CharacterController`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `move` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | — | The wanted velocity across the ground (m/s, world space; y is ignored unless gravityScale is 0). Set it every tick. |
| `jump` | `boolean` | `false` | — | — | — | Held to jump: a jump starts on the tick it becomes true while grounded (or in coyote time). |
| `jumpSpeed` | `number` | `6` | ≥ 0 | `mps` | tunable | The upward speed of a jump. |
| `gravityScale` | `number` | `1` | 0 … 10 | — | tunable | Multiplies og.Physics.gravity; 0 for top-down movement in any direction. |
| `maxSlope` | `number` | `45` | 0 … 89 | `deg` | tunable | The steepest slope it walks up. |
| `stepHeight` | `number` | `0.3` | ≥ 0 | `m` | tunable | The highest step it climbs without jumping (0: none). |
| `snapDistance` | `number` | `0.2` | ≥ 0 | `m` | tunable | How far below it the ground still holds it (stairs down, slopes): 0 lets it leave every edge. |
| `skin` | `number` | `0.02` | ≥ 0.001 | `m` | — | The gap kept between it and every surface. |
| `coyoteTime` | `number` | `0.1` | ≥ 0 | `s` | tunable | After leaving an edge, a jump still starts within this time. |
| `maxFallSpeed` | `number` | `40` | ≥ 0 | `mps` | tunable | The fastest fall. |
| `pushBodies` | `boolean` | `true` | — | — | — | Push dynamic bodies it walks into. |
| `grounded` | `boolean` | `false` | — | — | read only | Written: standing on walkable ground after the step. |
| `sliding` | `boolean` | `false` | — | — | read only | Written: sliding down a slope steeper than maxSlope. |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | read only | Written: the velocity it moved at this tick (the platform's carry included). |
| `groundNormal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | read only | Written: the ground's normal (zero in the air). |
| `coyote` | `number` | `0` | — | — | read only | Written: seconds left in which a jump may still start (coyote time). |
| `jumpWasHeld` | `boolean` | `false` | — | — | read only | Written: `jump` as the last step saw it (a jump starts on its rising edge). |
| `ground` | `Entity` | `0` | — | — | read only | Written: the entity it stands on (0 in the air); a moving one carries it. |

```ogt
component CharacterController {
  doc "A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, snapped to the ground, carried by the moving platform it stands on and falling by gravity. The entity needs an og.Transform and an og.Collider; the step makes it a kinematic body and pushes dynamic bodies it walks into."
  field move Vector3 "The wanted velocity across the ground (m/s, world space; y is ignored unless gravityScale is 0). Set it every tick." {
    unit mps
  }
  field jump bool "Held to jump: a jump starts on the tick it becomes true while grounded (or in coyote time)."
  field jumpSpeed f32 "The upward speed of a jump." {
    default 6
    min 0
    unit mps
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity; 0 for top-down movement in any direction." {
    default 1
    min 0
    max 10
    tunable true
  }
  field maxSlope f32 "The steepest slope it walks up." {
    default 45
    min 0
    max 89
    unit deg
    tunable true
  }
  field stepHeight f32 "The highest step it climbs without jumping (0: none)." {
    default 0.3
    min 0
    unit m
    tunable true
  }
  field snapDistance f32 "How far below it the ground still holds it (stairs down, slopes): 0 lets it leave every edge." {
    default 0.2
    min 0
    unit m
    tunable true
  }
  field skin f32 "The gap kept between it and every surface." {
    default 0.02
    min 0.001
    unit m
  }
  field coyoteTime f32 "After leaving an edge, a jump still starts within this time." {
    default 0.1
    min 0
    unit s
    tunable true
  }
  field maxFallSpeed f32 "The fastest fall." {
    default 40
    min 0
    unit mps
    tunable true
  }
  field pushBodies bool "Push dynamic bodies it walks into." {
    default true
  }
  field grounded bool "Written: standing on walkable ground after the step." {
    readonly true
  }
  field sliding bool "Written: sliding down a slope steeper than maxSlope." {
    readonly true
  }
  field velocity Vector3 "Written: the velocity it moved at this tick (the platform's carry included)." {
    unit mps
    readonly true
  }
  field groundNormal Vector3 "Written: the ground's normal (zero in the air)." {
    readonly true
  }
  field coyote f32 "Written: seconds left in which a jump may still start (coyote time)." {
    readonly true
  }
  field jumpWasHeld bool "Written: `jump` as the last step saw it (a jump starts on its rising edge)." {
    readonly true
  }
  field ground Entity "Written: the entity it stands on (0 in the air); a moving one carries it." {
    readonly true
  }
}
```

<sub>`api.ogt` line 3827</sub>

<a id="record-PlatformerController"></a>

### `og.PlatformerController` (component)

A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height (releasing jump while rising cuts the jump), apex hang, air jumps, wall slide and wall jump, one-way platforms (colliders on `oneWayLayers` are solid from above only; hold `down` to drop through), moving platforms (it rides what it stands on) and corner correction (a jump that clips a ceiling corner by less than `cornerCorrection` slides past it). The entity needs an og.Transform and an og.Collider (a box or capsule); it becomes a kinematic body.

Token `og.PlatformerController`, registered under the schema name `og.PlatformerController`; its value type is `PlatformerController`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `moveX` | `number` | `0` | -1 … 1 | — | — | The run input, -1 (left) to 1 (right). Set it every tick. |
| `jump` | `boolean` | `false` | — | — | — | Held to jump: a press is buffered for jumpBuffer; releasing while rising cuts the jump. |
| `down` | `boolean` | `false` | — | — | — | Held to drop through a one-way platform. |
| `maxSpeed` | `number` | `8` | ≥ 0 | `mps` | tunable | The top running speed. |
| `acceleration` | `number` | `70` | ≥ 0 | `mps2` | tunable | How fast it reaches maxSpeed on the ground. |
| `deceleration` | `number` | `80` | ≥ 0 | `mps2` | tunable | How fast it stops on the ground without input. |
| `airAcceleration` | `number` | `45` | ≥ 0 | `mps2` | tunable | Acceleration in the air. |
| `airDeceleration` | `number` | `25` | ≥ 0 | `mps2` | tunable | Deceleration in the air without input. |
| `jumpHeight` | `number` | `3.2` | ≥ 0 | `m` | tunable | How high a full jump goes. |
| `timeToApex` | `number` | `0.4` | ≥ 0.05 | `s` | tunable | How long a full jump takes to reach its height (sets gravity and the jump speed). |
| `fallGravity` | `number` | `1.8` | ≥ 0 | — | tunable | Gravity multiplier while falling (snappier landings above 1). |
| `jumpCut` | `number` | `2.5` | ≥ 1 | — | tunable | Gravity multiplier while rising with jump released (variable jump height; 1 disables it). |
| `apexGravity` | `number` | `0.6` | 0 … 1 | — | tunable | Gravity multiplier near the apex while jump is held (hang time; 1 disables it). |
| `apexThreshold` | `number` | `1.5` | ≥ 0 | `mps` | tunable | The vertical speed below which the apex multiplier applies. |
| `maxFallSpeed` | `number` | `20` | ≥ 0 | `mps` | tunable | The fastest fall. |
| `coyoteTime` | `number` | `0.1` | ≥ 0 | `s` | tunable | After running off an edge, a jump still starts within this time. |
| `jumpBuffer` | `number` | `0.12` | ≥ 0 | `s` | tunable | A jump pressed this long before landing starts on landing. |
| `airJumps` | `number` | `0` | 0 … 8 | — | — | Extra jumps in the air (1: a double jump). |
| `wallJumps` | `boolean` | `true` | — | — | — | Slide down walls it pushes against (and jump off them). |
| `wallSlideSpeed` | `number` | `3` | ≥ 0 | `mps` | tunable | The fastest slide down a wall. |
| `wallJump` | `Vector2` | `{ x: 8, y: 12 }` | — | `mps` | tunable | A wall jump's velocity: x away from the wall, y up. |
| `wallJumpLock` | `number` | `0.14` | ≥ 0 | `s` | tunable | After a wall jump, the run input is ignored for this long (so it leaves the wall). |
| `oneWayLayers` | `number` | `0` | — | — | — | The collision layers that are one-way platforms (solid from above), as bits. |
| `cornerCorrection` | `number` | `0.15` | ≥ 0 | `m` | tunable | How far a rising jump is nudged sideways past a ceiling corner (0: never). |
| `maxSlope` | `number` | `50` | 0 … 89 | `deg` | tunable | The steepest slope it runs up. |
| `snapDistance` | `number` | `0.1` | ≥ 0 | `m` | tunable | How far below it the ground still holds it (slopes, small drops). |
| `skin` | `number` | `0.01` | ≥ 0.001 | `m` | — | The gap kept between it and every surface. |
| `grounded` | `boolean` | `false` | — | — | read only | Written: standing on ground after the step. |
| `onWall` | `number` | `0` | -1 … 1 | — | read only | Written: -1 sliding on a wall to its left, 1 to its right, 0 none. |
| `facing` | `number` | `1` | -1 … 1 | — | read only | Written: -1 facing left, 1 right (the last run direction). |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | read only | Written: the velocity after the step (the platform's carry excluded). |
| `jumping` | `boolean` | `false` | — | — | read only | Written: rising from a jump (until the apex or a release cut). |
| `airJumpsLeft` | `number` | `0` | — | — | read only | Written: air jumps left before landing. |
| `coyote` | `number` | `0` | — | — | read only | Written: seconds of coyote time left. |
| `buffer` | `number` | `0` | — | — | read only | Written: seconds a buffered jump press stays valid. |
| `wallLock` | `number` | `0` | — | — | read only | Written: seconds the run input stays locked after a wall jump. |
| `dropTime` | `number` | `0` | — | — | read only | Written: seconds one-way platforms stay passable after dropping through. |
| `jumpWasHeld` | `boolean` | `false` | — | — | read only | Written: `jump` as the last step saw it. |
| `ground` | `Entity` | `0` | — | — | read only | Written: the entity it stands on (0 in the air); a moving one carries it. |

```ogt
component PlatformerController {
  doc "A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height (releasing jump while rising cuts the jump), apex hang, air jumps, wall slide and wall jump, one-way platforms (colliders on `oneWayLayers` are solid from above only; hold `down` to drop through), moving platforms (it rides what it stands on) and corner correction (a jump that clips a ceiling corner by less than `cornerCorrection` slides past it). The entity needs an og.Transform and an og.Collider (a box or capsule); it becomes a kinematic body."
  field moveX f32 "The run input, -1 (left) to 1 (right). Set it every tick." {
    min -1
    max 1
  }
  field jump bool "Held to jump: a press is buffered for jumpBuffer; releasing while rising cuts the jump."
  field down bool "Held to drop through a one-way platform."
  field maxSpeed f32 "The top running speed." {
    default 8
    min 0
    unit mps
    tunable true
  }
  field acceleration f32 "How fast it reaches maxSpeed on the ground." {
    default 70
    min 0
    unit mps2
    tunable true
  }
  field deceleration f32 "How fast it stops on the ground without input." {
    default 80
    min 0
    unit mps2
    tunable true
  }
  field airAcceleration f32 "Acceleration in the air." {
    default 45
    min 0
    unit mps2
    tunable true
  }
  field airDeceleration f32 "Deceleration in the air without input." {
    default 25
    min 0
    unit mps2
    tunable true
  }
  field jumpHeight f32 "How high a full jump goes." {
    default 3.2
    min 0
    unit m
    tunable true
  }
  field timeToApex f32 "How long a full jump takes to reach its height (sets gravity and the jump speed)." {
    default 0.4
    min 0.05
    unit s
    tunable true
  }
  field fallGravity f32 "Gravity multiplier while falling (snappier landings above 1)." {
    default 1.8
    min 0
    tunable true
  }
  field jumpCut f32 "Gravity multiplier while rising with jump released (variable jump height; 1 disables it)." {
    default 2.5
    min 1
    tunable true
  }
  field apexGravity f32 "Gravity multiplier near the apex while jump is held (hang time; 1 disables it)." {
    default 0.6
    min 0
    max 1
    tunable true
  }
  field apexThreshold f32 "The vertical speed below which the apex multiplier applies." {
    default 1.5
    min 0
    unit mps
    tunable true
  }
  field maxFallSpeed f32 "The fastest fall." {
    default 20
    min 0
    unit mps
    tunable true
  }
  field coyoteTime f32 "After running off an edge, a jump still starts within this time." {
    default 0.1
    min 0
    unit s
    tunable true
  }
  field jumpBuffer f32 "A jump pressed this long before landing starts on landing." {
    default 0.12
    min 0
    unit s
    tunable true
  }
  field airJumps u32 "Extra jumps in the air (1: a double jump)." {
    min 0
    max 8
  }
  field wallJumps bool "Slide down walls it pushes against (and jump off them)." {
    default true
  }
  field wallSlideSpeed f32 "The fastest slide down a wall." {
    default 3
    min 0
    unit mps
    tunable true
  }
  field wallJump Vector2 "A wall jump's velocity: x away from the wall, y up." {
    default 8 12
    unit mps
    tunable true
  }
  field wallJumpLock f32 "After a wall jump, the run input is ignored for this long (so it leaves the wall)." {
    default 0.14
    min 0
    unit s
    tunable true
  }
  field oneWayLayers u32 "The collision layers that are one-way platforms (solid from above), as bits."
  field cornerCorrection f32 "How far a rising jump is nudged sideways past a ceiling corner (0: never)." {
    default 0.15
    min 0
    unit m
    tunable true
  }
  field maxSlope f32 "The steepest slope it runs up." {
    default 50
    min 0
    max 89
    unit deg
    tunable true
  }
  field snapDistance f32 "How far below it the ground still holds it (slopes, small drops)." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field skin f32 "The gap kept between it and every surface." {
    default 0.01
    min 0.001
    unit m
  }
  field grounded bool "Written: standing on ground after the step." {
    readonly true
  }
  field onWall i32 "Written: -1 sliding on a wall to its left, 1 to its right, 0 none." {
    min -1
    max 1
    readonly true
  }
  field facing i32 "Written: -1 facing left, 1 right (the last run direction)." {
    default 1
    min -1
    max 1
    readonly true
  }
  field velocity Vector3 "Written: the velocity after the step (the platform's carry excluded)." {
    unit mps
    readonly true
  }
  field jumping bool "Written: rising from a jump (until the apex or a release cut)." {
    readonly true
  }
  field airJumpsLeft u32 "Written: air jumps left before landing." {
    readonly true
  }
  field coyote f32 "Written: seconds of coyote time left." {
    readonly true
  }
  field buffer f32 "Written: seconds a buffered jump press stays valid." {
    readonly true
  }
  field wallLock f32 "Written: seconds the run input stays locked after a wall jump." {
    readonly true
  }
  field dropTime f32 "Written: seconds one-way platforms stay passable after dropping through." {
    readonly true
  }
  field jumpWasHeld bool "Written: `jump` as the last step saw it." {
    readonly true
  }
  field ground Entity "Written: the entity it stands on (0 in the air); a moving one carries it." {
    readonly true
  }
}
```

<sub>`api.ogt` line 3908</sub>

<a id="record-CharacterEvent"></a>

### `og.CharacterEvent` (event)

Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. Read the next tick.

Token `og.CharacterEvent`, registered under the schema name `og.CharacterEvent`; its value type is `CharacterEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The character. |
| `kind` | `number` | `0` | 0 … 6 | — | — | CHARACTER_JUMPED 1, LANDED 2, WALL_JUMPED 3, AIR_JUMPED 4, LEFT_GROUND 5, BUMPED 6. |
| `speed` | `number` | `0` | — | `mps` | — | The vertical speed at the moment (a landing's impact speed is negative). |

```ogt
event CharacterEvent {
  doc "Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. Read the next tick."
  field entity Entity "The character."
  field kind u32 "CHARACTER_JUMPED 1, LANDED 2, WALL_JUMPED 3, AIR_JUMPED 4, LEFT_GROUND 5, BUMPED 6." {
    min 0
    max 6
  }
  field speed f32 "The vertical speed at the moment (a landing's impact speed is negative)." {
    unit mps
  }
}
```

<sub>`api.ogt` line 4098</sub>

<a id="record-Vehicle"></a>

### `og.Vehicle` (component)

A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). 3D: a raycast car with four wheels (suspension, grip, steering on the front axle); its forward is the chassis's local +z (three.js Object3D.lookAt aims +z). 2D: two wheel bodies on sprung wheel joints (front at +x), driven by a motor. The wheels are not entities: og.physics.vehicleWheels gives their poses for drawing.

Token `og.Vehicle`, registered under the schema name `og.Vehicle`; its value type is `Vehicle`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `throttle` | `number` | `0` | -1 … 1 | — | — | The accelerator, -1 (reverse) to 1. Set it every tick. |
| `steer` | `number` | `0` | -1 … 1 | — | — | The steering, -1 (left) to 1 (right); 3D only. |
| `brake` | `number` | `0` | 0 … 1 | — | — | The brake, 0 to 1. |
| `wheelBase` | `number` | `2.6` | ≥ 0 | `m` | tunable | The distance between the front and rear axles. |
| `track` | `number` | `1.6` | ≥ 0 | `m` | tunable | The distance between the left and right wheels (3D). |
| `wheelRadius` | `number` | `0.4` | ≥ 0.01 | `m` | tunable | The wheels' radius. |
| `mountHeight` | `number` | `0.2` | — | `m` | tunable | The height of the wheel mounts below the chassis centre. |
| `suspensionRest` | `number` | `0.35` | ≥ 0 | `m` | tunable | The suspension's length at rest. |
| `suspensionTravel` | `number` | `0.25` | ≥ 0 | `m` | tunable | How far the suspension compresses or extends from rest. |
| `suspensionStiffness` | `number` | `30` | ≥ 0 | — | tunable | The suspension's stiffness. |
| `suspensionDamping` | `number` | `4` | ≥ 0 | — | tunable | The suspension's damping. |
| `grip` | `number` | `2` | ≥ 0 | — | tunable | The tyres' grip (3D: the friction slip; 2D: the wheels' friction). |
| `engineForce` | `number` | `80` | ≥ 0 | — | tunable | The engine's force at full throttle (3D: per driven wheel, N; 2D: the motor's torque). |
| `maxSpeed` | `number` | `20` | ≥ 0 | `mps` | tunable | The top speed: the engine's force fades out towards it (throttle against the motion brakes first). |
| `brakeForce` | `number` | `30` | ≥ 0 | — | tunable | The brakes' force at full brake. |
| `maxSteer` | `number` | `0.55` | 0 … 1.2 | `rad` | tunable | The front wheels' largest steering angle (3D). |
| `drive` | `number` | `0` | 0 … 2 | — | — | DRIVE_REAR 0, DRIVE_FRONT 1, DRIVE_ALL 2. |
| `wheelMass` | `number` | `1` | ≥ 0.01 | `kg` | tunable | The wheels' mass (2D: the wheel bodies; 3D ignores it). |
| `speed` | `number` | `0` | — | `mps` | read only | Written: the forward speed (m/s; negative reversing). |
| `contacts` | `number` | `0` | — | — | read only | Written: how many wheels touch the ground. |

```ogt
component Vehicle {
  doc "A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). 3D: a raycast car with four wheels (suspension, grip, steering on the front axle); its forward is the chassis's local +z (three.js Object3D.lookAt aims +z). 2D: two wheel bodies on sprung wheel joints (front at +x), driven by a motor. The wheels are not entities: og.physics.vehicleWheels gives their poses for drawing."
  field throttle f32 "The accelerator, -1 (reverse) to 1. Set it every tick." {
    min -1
    max 1
  }
  field steer f32 "The steering, -1 (left) to 1 (right); 3D only." {
    min -1
    max 1
  }
  field brake f32 "The brake, 0 to 1." {
    min 0
    max 1
  }
  field wheelBase f32 "The distance between the front and rear axles." {
    default 2.6
    min 0
    unit m
    tunable true
  }
  field track f32 "The distance between the left and right wheels (3D)." {
    default 1.6
    min 0
    unit m
    tunable true
  }
  field wheelRadius f32 "The wheels' radius." {
    default 0.4
    min 0.01
    unit m
    tunable true
  }
  field mountHeight f32 "The height of the wheel mounts below the chassis centre." {
    default 0.2
    unit m
    tunable true
  }
  field suspensionRest f32 "The suspension's length at rest." {
    default 0.35
    min 0
    unit m
    tunable true
  }
  field suspensionTravel f32 "How far the suspension compresses or extends from rest." {
    default 0.25
    min 0
    unit m
    tunable true
  }
  field suspensionStiffness f32 "The suspension's stiffness." {
    default 30
    min 0
    tunable true
  }
  field suspensionDamping f32 "The suspension's damping." {
    default 4
    min 0
    tunable true
  }
  field grip f32 "The tyres' grip (3D: the friction slip; 2D: the wheels' friction)." {
    default 2
    min 0
    tunable true
  }
  field engineForce f32 "The engine's force at full throttle (3D: per driven wheel, N; 2D: the motor's torque)." {
    default 80
    min 0
    tunable true
  }
  field maxSpeed f32 "The top speed: the engine's force fades out towards it (throttle against the motion brakes first)." {
    default 20
    min 0
    unit mps
    tunable true
  }
  field brakeForce f32 "The brakes' force at full brake." {
    default 30
    min 0
    tunable true
  }
  field maxSteer f32 "The front wheels' largest steering angle (3D)." {
    default 0.55
    min 0
    max 1.2
    unit rad
    tunable true
  }
  field drive u32 "DRIVE_REAR 0, DRIVE_FRONT 1, DRIVE_ALL 2." {
    min 0
    max 2
  }
  field wheelMass f32 "The wheels' mass (2D: the wheel bodies; 3D ignores it)." {
    default 1
    min 0.01
    unit kg
    tunable true
  }
  field speed f32 "Written: the forward speed (m/s; negative reversing)." {
    unit mps
    readonly true
  }
  field contacts u32 "Written: how many wheels touch the ground." {
    readonly true
  }
}
```

<sub>`api.ogt` line 4114</sub>

<a id="record-Rope"></a>

### `og.Rope` (component)

A Verlet rope: `segments` links from the entity's position (plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind, colliding with every solid collider at `radius`. It moves nothing (one-way: a rope joint ties bodies together, og.Joint JOINT_ROPE). Its points are world state (the og.physics state block); og.physics.ropePoints reads them and the renderer draws the rope as a strip of `width` and `color`.

Token `og.Rope`, registered under the schema name `og.Rope`; its value type is `Rope`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `segments` | `number` | `16` | 1 … 256 | — | — | The number of links (points minus one). |
| `length` | `number` | `4` | ≥ 0 | `m` | tunable | The rope's length at rest. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The start's offset from the entity's position (world axes). |
| `end` | `Entity` | `0` | — | — | — | The entity the far end is tied to; 0 leaves it free. |
| `endOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The far end's offset from `end`'s position. |
| `iterations` | `number` | `12` | 1 … 64 | — | tunable | Constraint passes per step: more is stiffer, at more cost. |
| `damping` | `number` | `0.99` | 0 … 1 | — | tunable | How much of its velocity each point keeps per tick (1: none lost). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | tunable | A steady acceleration (wind). |
| `radius` | `number` | `0.05` | ≥ 0 | `m` | tunable | The thickness it collides at (0: no collision). |
| `width` | `number` | `0.08` | ≥ 0 | `m` | — | The drawn width. |
| `color` | `Vector3` | `{ x: 0.55, y: 0.42, z: 0.28 }` | — | — | — | The drawn linear colour. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Rope {
  doc "A Verlet rope: `segments` links from the entity's position (plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind, colliding with every solid collider at `radius`. It moves nothing (one-way: a rope joint ties bodies together, og.Joint JOINT_ROPE). Its points are world state (the og.physics state block); og.physics.ropePoints reads them and the renderer draws the rope as a strip of `width` and `color`."
  field segments u32 "The number of links (points minus one)." {
    default 16
    min 1
    max 256
  }
  field length f32 "The rope's length at rest." {
    default 4
    min 0
    unit m
    tunable true
  }
  field offset Vector3 "The start's offset from the entity's position (world axes)." {
    unit m
  }
  field end Entity "The entity the far end is tied to; 0 leaves it free."
  field endOffset Vector3 "The far end's offset from `end`'s position." {
    unit m
  }
  field iterations u32 "Constraint passes per step: more is stiffer, at more cost." {
    default 12
    min 1
    max 64
    tunable true
  }
  field damping f32 "How much of its velocity each point keeps per tick (1: none lost)." {
    default 0.99
    min 0
    max 1
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity." {
    default 1
    min -10
    max 10
    tunable true
  }
  field wind Vector3 "A steady acceleration (wind)." {
    unit mps2
    tunable true
  }
  field radius f32 "The thickness it collides at (0: no collision)." {
    default 0.05
    min 0
    unit m
    tunable true
  }
  field width f32 "The drawn width." {
    default 0.08
    min 0
    unit m
  }
  field color Vector3 "The drawn linear colour." {
    default 0.55 0.42 0.28
  }
  field visible bool "Whether the renderer draws it." {
    default true
  }
}
```

<sub>`api.ogt` line 4220</sub>

<a id="record-Cloth"></a>

### `og.Cloth` (component)

A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's position (its top-left corner, the grid laid out along the entity's local x and down -y), pinned by `pins` (pinned points follow the entity), with stretch and shear constraints, gravity, wind, collision against every solid collider and, with `thickness`, against itself. One-way like og.Rope; its points are world state. Drawn as a double-sided mesh with `material` (og.render.standardMaterial; 0: a plain cloth colour).

Token `og.Cloth`, registered under the schema name `og.Cloth`; its value type is `Cloth`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `columns` | `number` | `12` | 2 … 64 | — | — | Points across. |
| `rows` | `number` | `12` | 2 … 64 | — | — | Points down. |
| `spacing` | `number` | `0.15` | ≥ 0.001 | `m` | tunable | The rest distance between neighbouring points. |
| `pins` | `number` | `1` | — | — | — | Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings). |
| `iterations` | `number` | `8` | 1 … 64 | — | tunable | Constraint passes per step. |
| `damping` | `number` | `0.985` | 0 … 1 | — | tunable | How much of its velocity each point keeps per tick. |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `wind` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | tunable | A steady acceleration (wind); the cloth flutters with it. |
| `gust` | `number` | `0.6` | 0 … 2 | — | tunable | How much the wind varies over the cloth and the tick (0: steady). |
| `radius` | `number` | `0.04` | ≥ 0 | `m` | tunable | The thickness it collides at (0: no collision). |
| `thickness` | `number` | `0` | ≥ 0 | `m` | tunable | Self-collision: how far apart its points keep from the points of the cloth that are not their near neighbours (0: none). About `spacing` keeps a cloth that folds or piles up from passing through itself. |
| `substeps` | `number` | `1` | 1 … 16 | — | — | Simulation steps per tick, each integrating and then solving `iterations` passes; with self-collision enough that a point moves less than `thickness` per step (a point moves at most half the thickness a step). |
| `friction` | `number` | `0` | 0 … 1 | — | tunable | How much of its sliding a point touching a solid loses each step (0: it slides freely; near 1 it sticks where it lands, so a cloth lowered onto the floor folds instead of spreading). |
| `material` | `Material` | `0` | — | — | — | The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Cloth {
  doc "A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's position (its top-left corner, the grid laid out along the entity's local x and down -y), pinned by `pins` (pinned points follow the entity), with stretch and shear constraints, gravity, wind, collision against every solid collider and, with `thickness`, against itself. One-way like og.Rope; its points are world state. Drawn as a double-sided mesh with `material` (og.render.standardMaterial; 0: a plain cloth colour)."
  field columns u32 "Points across." {
    default 12
    min 2
    max 64
  }
  field rows u32 "Points down." {
    default 12
    min 2
    max 64
  }
  field spacing f32 "The rest distance between neighbouring points." {
    default 0.15
    min 0.001
    unit m
    tunable true
  }
  field pins u32 "Which points are pinned, a sum of PIN_* bits (1 top row, 2 top corners, 4 left column, 8 top rings)." {
    default 1
  }
  field iterations u32 "Constraint passes per step." {
    default 8
    min 1
    max 64
    tunable true
  }
  field damping f32 "How much of its velocity each point keeps per tick." {
    default 0.985
    min 0
    max 1
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity." {
    default 1
    min -10
    max 10
    tunable true
  }
  field wind Vector3 "A steady acceleration (wind); the cloth flutters with it." {
    unit mps2
    tunable true
  }
  field gust f32 "How much the wind varies over the cloth and the tick (0: steady)." {
    default 0.6
    min 0
    max 2
    tunable true
  }
  field radius f32 "The thickness it collides at (0: no collision)." {
    default 0.04
    min 0
    unit m
    tunable true
  }
  field thickness f32 "Self-collision: how far apart its points keep from the points of the cloth that are not their near neighbours (0: none). About `spacing` keeps a cloth that folds or piles up from passing through itself." {
    min 0
    unit m
    tunable true
  }
  field substeps u32 "Simulation steps per tick, each integrating and then solving `iterations` passes; with self-collision enough that a point moves less than `thickness` per step (a point moves at most half the thickness a step)." {
    default 1
    min 1
    max 16
  }
  field friction f32 "How much of its sliding a point touching a solid loses each step (0: it slides freely; near 1 it sticks where it lands, so a cloth lowered onto the floor folds instead of spreading)." {
    min 0
    max 1
    tunable true
  }
  field material Material "The material it is drawn with (og.render.standardMaterial); 0 is a plain cloth."
  field visible bool "Whether the renderer draws it." {
    default true
  }
}
```

<sub>`api.ogt` line 4286</sub>

<a id="record-Fluid2D"></a>

### `og.Fluid2D` (component)

A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure, viscosity) on a fixed grid in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider (pushed out of bodies; they push nothing back) and stay inside `bounds` around the entity when it is set. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: the particles blurred into a field the renderer thresholds into one surface of `color`, on sprite layer `layer`.

Token `og.Fluid2D`, registered under the schema name `og.Fluid2D`; its value type is `Fluid2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `1000` | 1 … 8192 | — | — | How many particles (spawned when the component appears or count changes). |
| `size` | `Vector2` | `{ x: 4, y: 3 }` | — | `m` | — | The spawn box, centred on the entity. |
| `bounds` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The container around the entity (0 0: none; colliders still hold it). |
| `radius` | `number` | `0.18` | ≥ 0.01 | `m` | tunable | The interaction radius: particles closer than this push and drag each other. |
| `restDensity` | `number` | `1` | ≥ 0 | — | tunable | The density it settles at (neighbours within radius, weighted). |
| `stiffness` | `number` | `0.1` | ≥ 0 | — | tunable | The pressure's strength (above about 0.15 it gets springy, then unstable). |
| `nearStiffness` | `number` | `0.5` | ≥ 0 | — | tunable | The near pressure's strength (keeps particles apart, gives surface tension). |
| `viscosity` | `number` | `0.15` | 0 … 4 | — | tunable | Linear viscosity (honey above 1). |
| `group` | `number` | `0` | — | — | — | Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone), and are drawn as one surface whose colour follows the mix; 0 keeps the fluid to itself. |
| `density` | `number` | `1` | 0.05 … 20 | — | tunable | How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one. |
| `mixing` | `number` | `0` | 0 … 1 | — | tunable | How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `substeps` | `number` | `2` | 1 … 8 | — | — | Simulation steps per tick. |
| `seed` | `number` | `0` | — | — | — | The seed of the spawn jitter. |
| `color` | `Vector3` | `{ x: 0.15, y: 0.45, z: 0.95 }` | — | — | — | The surface's linear colour. |
| `opacity` | `number` | `0.9` | 0 … 1 | — | — | The surface's opacity. |
| `blobScale` | `number` | `1.6` | 0.1 … 4 | — | — | The drawn size of each particle's blob (times radius). |
| `threshold` | `number` | `0.45` | 0.01 … 4 | — | — | The field value where the surface starts (higher: thinner, separate drops). |
| `layer` | `number` | `0` | — | — | — | The sprite layer it is drawn on. |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Fluid2D {
  doc "A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure, viscosity) on a fixed grid in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider (pushed out of bodies; they push nothing back) and stay inside `bounds` around the entity when it is set. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as metaballs: the particles blurred into a field the renderer thresholds into one surface of `color`, on sprite layer `layer`."
  field count u32 "How many particles (spawned when the component appears or count changes)." {
    default 1000
    min 1
    max 8192
  }
  field size Vector2 "The spawn box, centred on the entity." {
    default 4 3
    unit m
  }
  field bounds Vector2 "The container around the entity (0 0: none; colliders still hold it)." {
    unit m
  }
  field radius f32 "The interaction radius: particles closer than this push and drag each other." {
    default 0.18
    min 0.01
    unit m
    tunable true
  }
  field restDensity f32 "The density it settles at (neighbours within radius, weighted)." {
    default 1
    min 0
    tunable true
  }
  field stiffness f32 "The pressure's strength (above about 0.15 it gets springy, then unstable)." {
    default 0.1
    min 0
    tunable true
  }
  field nearStiffness f32 "The near pressure's strength (keeps particles apart, gives surface tension)." {
    default 0.5
    min 0
    tunable true
  }
  field viscosity f32 "Linear viscosity (honey above 1)." {
    default 0.15
    min 0
    max 4
    tunable true
  }
  field group u32 "Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone), and are drawn as one surface whose colour follows the mix; 0 keeps the fluid to itself."
  field density f32 "How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one." {
    default 1
    min 0.05
    max 20
    tunable true
  }
  field mixing f32 "How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out)." {
    min 0
    max 1
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity." {
    default 1
    min -10
    max 10
    tunable true
  }
  field substeps u32 "Simulation steps per tick." {
    default 2
    min 1
    max 8
  }
  field seed u32 "The seed of the spawn jitter."
  field color Vector3 "The surface's linear colour." {
    default 0.15 0.45 0.95
  }
  field opacity f32 "The surface's opacity." {
    default 0.9
    min 0
    max 1
  }
  field blobScale f32 "The drawn size of each particle's blob (times radius)." {
    default 1.6
    min 0.1
    max 4
  }
  field threshold f32 "The field value where the surface starts (higher: thinner, separate drops)." {
    default 0.45
    min 0.01
    max 4
  }
  field layer i32 "The sprite layer it is drawn on."
  field visible bool "Whether the renderer draws it." {
    default true
  }
}
```

<sub>`api.ogt` line 4362</sub>

<a id="record-Fluid3D"></a>

### `og.Fluid3D` (component)

A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure that gives surface tension, viscosity) over a spatial hash in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider, the terrain's height field included (pushed out; they push nothing back), and stay inside `bounds` around the entity when it is set. An emitter adds particles at `emitRate` up to `maxCount`; a sink box removes those that enter it. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as a screen-space surface: the particles' depth smoothed into one surface, refracting what is behind by its thickness and reflecting the sky, lit by the scene's lights. Cost: about 0.9 ms of CPU a tick for 2000 particles at 1 substep, 1.8 ms at 2 (native and wasm alike), growing with the particles; keep a scene's fluids to a few thousand particles. The surface costs about 1.1 ms of GPU (an Apple M4 Pro, the fluid covering a third of a 1280×720 frame) whatever the particle count, growing with the pixels it covers.

Token `og.Fluid3D`, registered under the schema name `og.Fluid3D`; its value type is `Fluid3D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `2000` | ≤ 32768 | — | — | How many particles to spawn in the box (when the component appears or count or seed changes; 0: none, the emitter fills it). |
| `size` | `Vector3` | `{ x: 2, y: 1, z: 2 }` | — | `m` | — | The spawn box, centred on the entity. |
| `bounds` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The container around the entity (0 on an axis: open on that axis; colliders and the terrain still hold it). |
| `radius` | `number` | `0.2` | ≥ 0.02 | `m` | tunable | The interaction radius: particles closer than this push and drag each other. Cost grows with the particles, not the radius. |
| `restDensity` | `number` | `10` | ≥ 0 | — | tunable | The density it settles at (neighbours within radius, weighted). |
| `stiffness` | `number` | `0.02` | ≥ 0 | — | tunable | The pressure's strength (above about 0.05 it jitters, then springs apart). |
| `nearStiffness` | `number` | `0.4` | ≥ 0 | — | tunable | The near pressure's strength (keeps particles apart, gives surface tension). |
| `viscosity` | `number` | `0.4` | 0 … 4 | — | tunable | Linear viscosity (it also calms the surface; honey above 1). |
| `group` | `number` | `0` | — | — | — | Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone); 0 keeps the fluid to itself. |
| `density` | `number` | `1` | 0.05 … 20 | — | tunable | How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one. |
| `mixing` | `number` | `0` | 0 … 1 | — | tunable | How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out). |
| `gravityScale` | `number` | `1` | -10 … 10 | — | tunable | Multiplies og.Physics.gravity. |
| `substeps` | `number` | `2` | 1 … 8 | — | — | Simulation steps per tick (each costs a full pass over the particles). |
| `seed` | `number` | `0` | — | — | — | The seed of the spawn and emitter jitter. |
| `maxCount` | `number` | `8192` | 1 … 32768 | — | — | The most particles alive at once (the emitter stops there). |
| `emitRate` | `number` | `0` | 0 … 20000 | — | tunable | Particles the emitter adds per second (0: no emitter). |
| `emitOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the emitter is, from the entity (world axes). |
| `emitVelocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | The velocity emitted particles start with. |
| `emitRadius` | `number` | `0.1` | ≥ 0 | `m` | tunable | The half-size of the cube new particles appear in around the emitter. |
| `sinkOffset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sink's centre, from the entity (world axes). |
| `sinkSize` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sink box: particles that enter it are removed (0 0 0: no sink). |
| `color` | `Vector3` | `{ x: 0.2, y: 0.55, z: 0.9 }` | — | — | — | The fluid's linear colour (what light passing through it keeps). |
| `opacity` | `number` | `0.6` | 0 … 1 | — | — | How opaque a metre of it is (0: clear as glass, 1: opaque). |
| `reflectivity` | `number` | `1` | 0 … 2 | — | — | How much it reflects (scales the Fresnel term). |
| `refraction` | `number` | `1` | 0 … 4 | — | — | How far refraction bends what is seen through it. |
| `drawScale` | `number` | `0.6` | 0.1 … 2 | — | — | The drawn size of each particle (times radius). |
| `smoothing` | `number` | `1` | 0 … 2 | — | — | How much the surface is smoothed (0: beads, 1: one surface). |
| `visible` | `boolean` | `true` | — | — | — | Whether the renderer draws it. |

```ogt
component Fluid3D {
  doc "A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. 2005: pressure, near pressure that gives surface tension, viscosity) over a spatial hash in a fixed order, so it is deterministic and part of the og.physics state block. Particles collide with every solid collider, the terrain's height field included (pushed out; they push nothing back), and stay inside `bounds` around the entity when it is set. An emitter adds particles at `emitRate` up to `maxCount`; a sink box removes those that enter it. Fluids of one `group` interact: by `density` they layer, by `mixing` they blend. Drawn as a screen-space surface: the particles' depth smoothed into one surface, refracting what is behind by its thickness and reflecting the sky, lit by the scene's lights. Cost: about 0.9 ms of CPU a tick for 2000 particles at 1 substep, 1.8 ms at 2 (native and wasm alike), growing with the particles; keep a scene's fluids to a few thousand particles. The surface costs about 1.1 ms of GPU (an Apple M4 Pro, the fluid covering a third of a 1280×720 frame) whatever the particle count, growing with the pixels it covers."
  field count u32 "How many particles to spawn in the box (when the component appears or count or seed changes; 0: none, the emitter fills it)." {
    default 2000
    max 32768
  }
  field size Vector3 "The spawn box, centred on the entity." {
    default 2 1 2
    unit m
  }
  field bounds Vector3 "The container around the entity (0 on an axis: open on that axis; colliders and the terrain still hold it)." {
    unit m
  }
  field radius f32 "The interaction radius: particles closer than this push and drag each other. Cost grows with the particles, not the radius." {
    default 0.2
    min 0.02
    unit m
    tunable true
  }
  field restDensity f32 "The density it settles at (neighbours within radius, weighted)." {
    default 10
    min 0
    tunable true
  }
  field stiffness f32 "The pressure's strength (above about 0.05 it jitters, then springs apart)." {
    default 0.02
    min 0
    tunable true
  }
  field nearStiffness f32 "The near pressure's strength (keeps particles apart, gives surface tension)." {
    default 0.4
    min 0
    tunable true
  }
  field viscosity f32 "Linear viscosity (it also calms the surface; honey above 1)." {
    default 0.4
    min 0
    max 4
    tunable true
  }
  field group u32 "Fluids sharing a group above 0 push, drag and (by `mixing`) mix with each other as one fluid, at most four to a group (in entity order; a fifth is simulated alone); 0 keeps the fluid to itself."
  field density f32 "How heavy its particles are against the group's other fluids: a heavier fluid sinks under a lighter one." {
    default 1
    min 0.05
    max 20
    tunable true
  }
  field mixing f32 "How readily it mixes with the group's other fluids: 0 keeps them apart in layers (the boundary held), 1 blends them where they touch (their mix, colour and weight even out)." {
    min 0
    max 1
    tunable true
  }
  field gravityScale f32 "Multiplies og.Physics.gravity." {
    default 1
    min -10
    max 10
    tunable true
  }
  field substeps u32 "Simulation steps per tick (each costs a full pass over the particles)." {
    default 2
    min 1
    max 8
  }
  field seed u32 "The seed of the spawn and emitter jitter."
  field maxCount u32 "The most particles alive at once (the emitter stops there)." {
    default 8192
    min 1
    max 32768
  }
  field emitRate f32 "Particles the emitter adds per second (0: no emitter)." {
    min 0
    max 20000
    tunable true
  }
  field emitOffset Vector3 "Where the emitter is, from the entity (world axes)." {
    unit m
  }
  field emitVelocity Vector3 "The velocity emitted particles start with." {
    unit mps
    tunable true
  }
  field emitRadius f32 "The half-size of the cube new particles appear in around the emitter." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field sinkOffset Vector3 "The sink's centre, from the entity (world axes)." {
    unit m
  }
  field sinkSize Vector3 "The sink box: particles that enter it are removed (0 0 0: no sink)." {
    unit m
  }
  field color Vector3 "The fluid's linear colour (what light passing through it keeps)." {
    default 0.2 0.55 0.9
  }
  field opacity f32 "How opaque a metre of it is (0: clear as glass, 1: opaque)." {
    default 0.6
    min 0
    max 1
  }
  field reflectivity f32 "How much it reflects (scales the Fresnel term)." {
    default 1
    min 0
    max 2
  }
  field refraction f32 "How far refraction bends what is seen through it." {
    default 1
    min 0
    max 4
  }
  field drawScale f32 "The drawn size of each particle (times radius)." {
    default 0.6
    min 0.1
    max 2
  }
  field smoothing f32 "How much the surface is smoothed (0: beads, 1: one surface)." {
    default 1
    min 0
    max 2
  }
  field visible bool "Whether the renderer draws it." {
    default true
  }
}
```

<sub>`api.ogt` line 4451</sub>

<a id="record-WaterSurface"></a>

### `og.WaterSurface` (component)

Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies crossing it push waves (splashes; og.Splash events) and float by `buoyancy`. The heights are world state; an og.Water on the same entity draws them.

Token `og.WaterSurface`, registered under the schema name `og.WaterSurface`; its value type is `WaterSurface`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `resolution` | `number` | `64` | 8 … 256 | — | — | Grid points along each side. |
| `size` | `number` | `16` | ≥ 0.1 | `m` | tunable | The side of the square of water. |
| `waveSpeed` | `number` | `3` | ≥ 0 | `mps` | tunable | How fast waves travel. |
| `damping` | `number` | `0.4` | 0 … 1 | — | tunable | How much of a wave survives a second (1: no loss). |
| `splash` | `number` | `1` | ≥ 0 | — | tunable | How strongly bodies push waves. |
| `buoyancy` | `number` | `9.81` | ≥ 0 | — | tunable | The upward force on a submerged body per cubic metre of its box under water (0: bodies sink). |
| `drag` | `number` | `1.5` | ≥ 0 | `perSecond` | tunable | How much a submerged body's velocity is slowed. |

```ogt
component WaterSurface {
  doc "Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. Dynamic bodies crossing it push waves (splashes; og.Splash events) and float by `buoyancy`. The heights are world state; an og.Water on the same entity draws them."
  field resolution u32 "Grid points along each side." {
    default 64
    min 8
    max 256
  }
  field size f32 "The side of the square of water." {
    default 16
    min 0.1
    unit m
    tunable true
  }
  field waveSpeed f32 "How fast waves travel." {
    default 3
    min 0
    unit mps
    tunable true
  }
  field damping f32 "How much of a wave survives a second (1: no loss)." {
    default 0.4
    min 0
    max 1
    tunable true
  }
  field splash f32 "How strongly bodies push waves." {
    default 1
    min 0
    tunable true
  }
  field buoyancy f32 "The upward force on a submerged body per cubic metre of its box under water (0: bodies sink)." {
    default 9.81
    min 0
    tunable true
  }
  field drag f32 "How much a submerged body's velocity is slowed." {
    default 1.5
    min 0
    unit perSecond
    tunable true
  }
}
```

<sub>`api.ogt` line 4577</sub>

<a id="record-WaterLine"></a>

### `og.WaterLine` (component)

2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and passing waves to its neighbours by `spread`. Bodies (and characters) crossing the surface splash it (og.Splash events) and float by `buoyancy`. The springs are world state; an og.Water2D on the same entity draws it.

Token `og.WaterLine`, registered under the schema name `og.WaterLine`; its value type is `WaterLine`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `12` | ≥ 0.1 | `m` | tunable | The pool's width. |
| `depth` | `number` | `3` | ≥ 0 | `m` | tunable | The pool's depth below the surface. |
| `columns` | `number` | `96` | 4 … 512 | — | — | Springs along the surface. |
| `tension` | `number` | `60` | ≥ 0 | — | tunable | How hard each spring pulls back to rest. |
| `damping` | `number` | `2.5` | ≥ 0 | — | tunable | How fast a spring's motion dies. |
| `spread` | `number` | `0.25` | 0 … 0.5 | — | tunable | How much of a spring's height its neighbours feel each pass (waves travel). |
| `splash` | `number` | `0.04` | ≥ 0 | — | tunable | How strongly crossing bodies splash. |
| `buoyancy` | `number` | `12` | ≥ 0 | — | tunable | The upward force on a submerged body per square metre of its box under water (0: bodies sink). |
| `drag` | `number` | `2` | ≥ 0 | `perSecond` | tunable | How much a submerged body's velocity is slowed. |

```ogt
component WaterLine {
  doc "2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and passing waves to its neighbours by `spread`. Bodies (and characters) crossing the surface splash it (og.Splash events) and float by `buoyancy`. The springs are world state; an og.Water2D on the same entity draws it."
  field width f32 "The pool's width." {
    default 12
    min 0.1
    unit m
    tunable true
  }
  field depth f32 "The pool's depth below the surface." {
    default 3
    min 0
    unit m
    tunable true
  }
  field columns u32 "Springs along the surface." {
    default 96
    min 4
    max 512
  }
  field tension f32 "How hard each spring pulls back to rest." {
    default 60
    min 0
    tunable true
  }
  field damping f32 "How fast a spring's motion dies." {
    default 2.5
    min 0
    tunable true
  }
  field spread f32 "How much of a spring's height its neighbours feel each pass (waves travel)." {
    default 0.25
    min 0
    max 0.5
    tunable true
  }
  field splash f32 "How strongly crossing bodies splash." {
    default 0.04
    min 0
    tunable true
  }
  field buoyancy f32 "The upward force on a submerged body per square metre of its box under water (0: bodies sink)." {
    default 12
    min 0
    tunable true
  }
  field drag f32 "How much a submerged body's velocity is slowed." {
    default 2
    min 0
    unit perSecond
    tunable true
  }
}
```

<sub>`api.ogt` line 4620</sub>

<a id="record-Splash"></a>

### `og.Splash` (event)

Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. Read the next tick.

Token `og.Splash`, registered under the schema name `og.Splash`; its value type is `Splash`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `water` | `Entity` | `0` | — | — | — | The water's entity. |
| `entity` | `Entity` | `0` | — | — | — | What crossed it. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it crossed the surface. |
| `speed` | `number` | `0` | — | `mps` | — | Its vertical speed (negative going in). |

```ogt
event Splash {
  doc "Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. Read the next tick."
  field water Entity "The water's entity."
  field entity Entity "What crossed it."
  field point Vector3 "Where it crossed the surface." {
    unit m
  }
  field speed f32 "Its vertical speed (negative going in)." {
    unit mps
  }
}
```

<sub>`api.ogt` line 4673</sub>

<a id="record-Ragdoll"></a>

### `og.Ragdoll` (component)

A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy plane at their animated z, for a skeleton laid out in that plane), built from the skeleton's rest pose. `weight` is where the pose comes from: 0 the animation (the bodies follow it kinematically), 1 the physics (the bodies are dynamic, starting with the animation's velocity); the pose drawn blends between them over `blendTime`, both ways. The bodies are world state (the og.physics state block), not entities.

Token `og.Ragdoll`, registered under the schema name `og.Ragdoll`; its value type is `Ragdoll`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `weight` | `number` | `0` | 0 … 1 | — | tunable | 0 animated, 1 ragdoll. Set it to 1 on a hit; back to 0 to recover. |
| `blendTime` | `number` | `0.25` | ≥ 0 | `s` | tunable | How long the blend between animation and physics takes, both ways. |
| `mass` | `number` | `60` | ≥ 0.1 | `kg` | tunable | The whole body's mass, shared by the bones by length. |
| `thickness` | `number` | `0.22` | 0.01 … 1 | — | tunable | A bone capsule's radius as a fraction of its length. |
| `jointLimit` | `number` | `0.9` | 0 … 3.1 | `rad` | tunable | How far each joint bends from the rest pose. |
| `damping` | `number` | `0.6` | 0 … 20 | `perSecond` | tunable | How much of their motion the bones lose per second. |
| `impulse` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | An impulse given to every bone when weight first goes up (a hit). |
| `blend` | `number` | `0` | — | — | read only | Written: the blend in use (0 animation .. 1 physics). |

```ogt
component Ragdoll {
  doc "A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy plane at their animated z, for a skeleton laid out in that plane), built from the skeleton's rest pose. `weight` is where the pose comes from: 0 the animation (the bodies follow it kinematically), 1 the physics (the bodies are dynamic, starting with the animation's velocity); the pose drawn blends between them over `blendTime`, both ways. The bodies are world state (the og.physics state block), not entities."
  field weight f32 "0 animated, 1 ragdoll. Set it to 1 on a hit; back to 0 to recover." {
    min 0
    max 1
    tunable true
  }
  field blendTime f32 "How long the blend between animation and physics takes, both ways." {
    default 0.25
    min 0
    unit s
    tunable true
  }
  field mass f32 "The whole body's mass, shared by the bones by length." {
    default 60
    min 0.1
    unit kg
    tunable true
  }
  field thickness f32 "A bone capsule's radius as a fraction of its length." {
    default 0.22
    min 0.01
    max 1
    tunable true
  }
  field jointLimit f32 "How far each joint bends from the rest pose." {
    default 0.9
    min 0
    max 3.1
    unit rad
    tunable true
  }
  field damping f32 "How much of their motion the bones lose per second." {
    default 0.6
    min 0
    max 20
    unit perSecond
    tunable true
  }
  field impulse Vector3 "An impulse given to every bone when weight first goes up (a hit)."
  field blend f32 "Written: the blend in use (0 animation .. 1 physics)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 4685</sub>

## Types

<a id="type-QueryFilter"></a>

### `QueryFilter` (desc struct)

Which colliders a scene query considers. The zero value considers every solid collider on every layer.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mask` | `number` | `0` | — | — | — | Layers to consider: a collider counts when its layers meet this; 0 means every layer. |
| `includeSensors` | `boolean` | `false` | — | — | — | Also report sensors. |
| `exclude` | `Entity \| null` | `null` | — | — | — | An entity to ignore, such as the caster itself. |

```ogt
struct QueryFilter {
  doc "Which colliders a scene query considers. The zero value considers every solid collider on every layer."
  desc true
  field mask u32 "Layers to consider: a collider counts when its layers meet this; 0 means every layer."
  field includeSensors bool "Also report sensors."
  field exclude Option.Entity "An entity to ignore, such as the caster itself."
}
```

<sub>`api.ogt` line 4730</sub>

<a id="type-RaycastDesc"></a>

### `RaycastDesc` (desc struct)

A ray for og.physics.raycast.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the ray starts. |
| `direction` | `Vector3` | `{ x: 0, y: -1, z: 0 }` | — | — | — | Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far it reaches; 0 is unbounded. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct RaycastDesc {
  doc "A ray for og.physics.raycast."
  desc true
  field origin Vector3 "Where the ray starts." {
    unit m
  }
  field direction Vector3 "Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy)." {
    default 0 -1 0
  }
  field maxDistance f32 "How far it reaches; 0 is unbounded." {
    min 0
    unit m
  }
  field filter QueryFilter "Which colliders count."
}
```

<sub>`api.ogt` line 4738</sub>

<a id="type-RaycastHit"></a>

### `RaycastHit` (struct)

The nearest collider a ray meets.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The collider's entity. |
| `distance` | `number` | `0` | — | `m` | — | From the origin along the normalised direction. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the ray meets the collider. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The surface normal there (z is 0 in 2D). |

```ogt
struct RaycastHit {
  doc "The nearest collider a ray meets."
  field entity Entity "The collider's entity."
  field distance f32 "From the origin along the normalised direction." {
    unit m
  }
  field point Vector3 "Where the ray meets the collider."
  field normal Vector3 "The surface normal there (z is 0 in 2D)."
}
```

<sub>`api.ogt` line 4754</sub>

<a id="type-ShapeDesc"></a>

### `ShapeDesc` (desc struct)

The shape of a shape cast or an overlap test: the shape fields of a Collider. The zero value is a 1 m box.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `number` | `0` | 0 … 5 | — | — | One of the SHAPE_* codes. |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | `m` | — | Full extents of a box. |
| `radius` | `number` | `0.5` | — | `m` | — | Radius of a sphere, circle, capsule or cylinder. |
| `height` | `number` | `1` | — | `m` | — | Capsule or cylinder height. |
| `mesh` | `Geometry` | `0` | — | — | — | The geometry of a convex or triangle-mesh shape. |

```ogt
struct ShapeDesc {
  doc "The shape of a shape cast or an overlap test: the shape fields of a Collider. The zero value is a 1 m box."
  desc true
  field shape u32 "One of the SHAPE_* codes." {
    min 0
    max 5
  }
  field size Vector3 "Full extents of a box." {
    default 1 1 1
    unit m
  }
  field radius f32 "Radius of a sphere, circle, capsule or cylinder." {
    default 0.5
    unit m
  }
  field height f32 "Capsule or cylinder height." {
    default 1
    unit m
  }
  field mesh Geometry "The geometry of a convex or triangle-mesh shape."
}
```

<sub>`api.ogt` line 4764</sub>

<a id="type-ShapeCastDesc"></a>

### `ShapeCastDesc` (desc struct)

A shape swept along a direction for og.physics.shapeCast.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `ShapeDesc` | `{}` | — | — | — | The swept shape. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the sweep starts. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The shape's rotation; a zero quaternion is no rotation. |
| `direction` | `Vector3` | `{ x: 0, y: -1, z: 0 }` | — | — | — | Where it sweeps; it need not be normalised and must not be zero. |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | How far it sweeps; 0 is unbounded. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct ShapeCastDesc {
  doc "A shape swept along a direction for og.physics.shapeCast."
  desc true
  field shape ShapeDesc "The swept shape."
  field position Vector3 "Where the sweep starts." {
    unit m
  }
  field rotation Quaternion "The shape's rotation; a zero quaternion is no rotation."
  field direction Vector3 "Where it sweeps; it need not be normalised and must not be zero." {
    default 0 -1 0
  }
  field maxDistance f32 "How far it sweeps; 0 is unbounded." {
    min 0
    unit m
  }
  field filter QueryFilter "Which colliders count."
}
```

<sub>`api.ogt` line 4786</sub>

<a id="type-ShapeCastHit"></a>

### `ShapeCastHit` (struct)

The first collider a swept shape meets.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The collider's entity. |
| `distance` | `number` | `0` | — | `m` | — | How far the shape travelled before touching. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The contact point on the hit collider, in world space. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The hit collider's surface normal there, in world space. |

```ogt
struct ShapeCastHit {
  doc "The first collider a swept shape meets."
  field entity Entity "The collider's entity."
  field distance f32 "How far the shape travelled before touching." {
    unit m
  }
  field point Vector3 "The contact point on the hit collider, in world space."
  field normal Vector3 "The hit collider's surface normal there, in world space."
}
```

<sub>`api.ogt` line 4804</sub>

<a id="type-OverlapDesc"></a>

### `OverlapDesc` (desc struct)

A shape placed in the world for og.physics.overlap.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `ShapeDesc` | `{}` | — | — | — | The shape. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation; a zero quaternion is no rotation. |
| `filter` | `QueryFilter` | `{}` | — | — | — | Which colliders count. |

```ogt
struct OverlapDesc {
  doc "A shape placed in the world for og.physics.overlap."
  desc true
  field shape ShapeDesc "The shape."
  field position Vector3 "Where it is." {
    unit m
  }
  field rotation Quaternion "Its rotation; a zero quaternion is no rotation."
  field filter QueryFilter "Which colliders count."
}
```

<sub>`api.ogt` line 4814</sub>

<a id="type-MeshDesc"></a>

### `MeshDesc` (desc struct)

Geometry for convex and triangle-mesh colliders, in the mesh's own space.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `positions` | `readonly Vector3[]` | `[]` | — | — | — | Vertex positions (2D polygons use x and y). |
| `indices` | `readonly number[]` | `[]` | — | — | — | Triangles as triples of indices into positions; a convex hull ignores them. |

```ogt
struct MeshDesc {
  doc "Geometry for convex and triangle-mesh colliders, in the mesh's own space."
  desc true
  field positions List.Vector3 "Vertex positions (2D polygons use x and y)."
  field indices List.u32 "Triangles as triples of indices into positions; a convex hull ignores them."
}
```

<sub>`api.ogt` line 4825</sub>

<a id="type-WheelPose"></a>

### `WheelPose` (struct)

A wheel of an og.Vehicle as the last step left it, for drawing.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The wheel's centre in world space. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation (steering and spin) in world space. |
| `contact` | `boolean` | `false` | — | — | — | Whether it touches the ground. |

```ogt
struct WheelPose {
  doc "A wheel of an og.Vehicle as the last step left it, for drawing."
  field position Vector3 "The wheel's centre in world space." {
    unit m
  }
  field rotation Quaternion "Its rotation (steering and spin) in world space."
  field contact bool "Whether it touches the ground."
}
```

<sub>`api.ogt` line 4861</sub>

<a id="type-RagdollBoneDesc"></a>

### `RagdollBoneDesc` (desc struct)

One bone of a ragdoll rig: a capsule from the bone's position along `axis` for `length`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `parent` | `number` | `-1` | — | — | — | The parent bone's index (it must come earlier); -1 for the root. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bone's rest position relative to the entity. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The bone's rest rotation relative to the entity; a zero quaternion is no rotation. |
| `length` | `number` | `0.25` | — | `m` | — | How long the bone is (to its child). |
| `axis` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The direction the bone points in its own frame (towards its child). |
| `joint` | `number` | `-1` | — | — | — | The skeleton joint (its index in the skin) the bone stands for, so the model is drawn with it; -1 for none. |

```ogt
struct RagdollBoneDesc {
  doc "One bone of a ragdoll rig: a capsule from the bone's position along `axis` for `length`."
  desc true
  field parent i32 "The parent bone's index (it must come earlier); -1 for the root." {
    default -1
  }
  field position Vector3 "The bone's rest position relative to the entity." {
    unit m
  }
  field rotation Quaternion "The bone's rest rotation relative to the entity; a zero quaternion is no rotation."
  field length f32 "How long the bone is (to its child)." {
    default 0.25
    unit m
  }
  field axis Vector3 "The direction the bone points in its own frame (towards its child)." {
    default 0 1 0
  }
  field joint i32 "The skeleton joint (its index in the skin) the bone stands for, so the model is drawn with it; -1 for none." {
    default -1
  }
}
```

<sub>`api.ogt` line 4870</sub>

<a id="type-RagdollRigDesc"></a>

### `RagdollRigDesc` (desc struct)

The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bones` | `readonly RagdollBoneDesc[]` | `[]` | — | — | — | The bones. |
| `skeleton` | `SkeletonAsset` | `0` | — | — | — | The skeleton the bones' joints belong to (0: none): the entity's posed model is drawn blended towards the bones by og.Ragdoll.blend. |

```ogt
struct RagdollRigDesc {
  doc "The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton."
  desc true
  field bones List.RagdollBoneDesc "The bones."
  field skeleton SkeletonAsset "The skeleton the bones' joints belong to (0: none): the entity's posed model is drawn blended towards the bones by og.Ragdoll.blend."
}
```

<sub>`api.ogt` line 4892</sub>

<a id="type-BonePose"></a>

### `BonePose` (struct)

A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bone's position in world space. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation in world space. |

```ogt
struct BonePose {
  doc "A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight)."
  field position Vector3 "The bone's position in world space." {
    unit m
  }
  field rotation Quaternion "Its rotation in world space."
}
```

<sub>`api.ogt` line 4899</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-BODY_DYNAMIC"></a>`BODY_DYNAMIC` | `number` | `0` | RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick. |
| <a id="const-BODY_STATIC"></a>`BODY_STATIC` | `number` | `1` | RigidBody.kind: never moves; collides with dynamic bodies. |
| <a id="const-BODY_KINEMATIC"></a>`BODY_KINEMATIC` | `number` | `2` | RigidBody.kind: moved by game code through its Transform; its Velocity, if any, is how fast it pushes. |
| <a id="const-LOCK_TRANSLATION_X"></a>`LOCK_TRANSLATION_X` | `number` | `1` | RigidBody.lock bit: no translation along x. |
| <a id="const-LOCK_TRANSLATION_Y"></a>`LOCK_TRANSLATION_Y` | `number` | `2` | RigidBody.lock bit: no translation along y. |
| <a id="const-LOCK_TRANSLATION_Z"></a>`LOCK_TRANSLATION_Z` | `number` | `4` | RigidBody.lock bit: no translation along z. |
| <a id="const-LOCK_ROTATION_X"></a>`LOCK_ROTATION_X` | `number` | `8` | RigidBody.lock bit: no rotation about x. |
| <a id="const-LOCK_ROTATION_Y"></a>`LOCK_ROTATION_Y` | `number` | `16` | RigidBody.lock bit: no rotation about y. |
| <a id="const-LOCK_ROTATION_Z"></a>`LOCK_ROTATION_Z` | `number` | `32` | RigidBody.lock bit: no rotation about z (the 2D profile's only rotation). |
| <a id="const-SHAPE_BOX"></a>`SHAPE_BOX` | `number` | `0` | Collider.shape: a box of `size` (both profiles). |
| <a id="const-SHAPE_SPHERE"></a>`SHAPE_SPHERE` | `number` | `1` | Collider.shape: a sphere of `radius` (a circle in 2D). |
| <a id="const-SHAPE_CAPSULE"></a>`SHAPE_CAPSULE` | `number` | `2` | Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles). |
| <a id="const-SHAPE_CYLINDER"></a>`SHAPE_CYLINDER` | `number` | `3` | Collider.shape: a cylinder along y of `radius` and `height` (3D only). |
| <a id="const-SHAPE_CONVEX"></a>`SHAPE_CONVEX` | `number` | `4` | Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D). |
| <a id="const-SHAPE_TRIMESH"></a>`SHAPE_TRIMESH` | `number` | `5` | Collider.shape: the triangles of `mesh` (3D only; for static and kinematic bodies). |
| <a id="const-SHAPE_HEIGHTFIELD"></a>`SHAPE_HEIGHTFIELD` | `number` | `6` | Collider.shape: the height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry). |
| <a id="const-PROFILE_2D"></a>`PROFILE_2D` | `number` | `2` | Physics.profile: rapier2d, bodies in the xy plane rotating about z. |
| <a id="const-PROFILE_3D"></a>`PROFILE_3D` | `number` | `3` | Physics.profile: rapier3d. |
| <a id="const-JOINT_FIXED"></a>`JOINT_FIXED` | `number` | `0` | `Joint.kind`: the two bodies move as one. |
| <a id="const-JOINT_REVOLUTE"></a>`JOINT_REVOLUTE` | `number` | `1` | `Joint.kind`: a hinge about `axis` (2D: about z), with optional limits and a motor. |
| <a id="const-JOINT_PRISMATIC"></a>`JOINT_PRISMATIC` | `number` | `2` | `Joint.kind`: a slider along `axis`, with optional limits and a motor. |
| <a id="const-JOINT_SPRING"></a>`JOINT_SPRING` | `number` | `3` | `Joint.kind`: a damped spring pulling the anchors to `restLength` apart. |
| <a id="const-JOINT_ROPE"></a>`JOINT_ROPE` | `number` | `4` | `Joint.kind`: a rope: the anchors stay at most `length` apart, free inside it. |
| <a id="const-JOINT_BALL"></a>`JOINT_BALL` | `number` | `5` | `Joint.kind`: a ball and socket (3D; the 2D profile treats it as a hinge). |
| <a id="const-MOTOR_NONE"></a>`MOTOR_NONE` | `number` | `0` | `Joint.motor`: no motor. |
| <a id="const-MOTOR_VELOCITY"></a>`MOTOR_VELOCITY` | `number` | `1` | `Joint.motor`: drive towards `motorTarget` speed (rad/s or m/s). |
| <a id="const-MOTOR_POSITION"></a>`MOTOR_POSITION` | `number` | `2` | `Joint.motor`: spring towards `motorTarget` angle or position (a servo). |
| <a id="const-CHARACTER_JUMPED"></a>`CHARACTER_JUMPED` | `number` | `1` | `CharacterEvent.kind`: a jump from the ground (or in coyote time). |
| <a id="const-CHARACTER_LANDED"></a>`CHARACTER_LANDED` | `number` | `2` | `CharacterEvent.kind`: touched the ground after being in the air. |
| <a id="const-CHARACTER_WALL_JUMPED"></a>`CHARACTER_WALL_JUMPED` | `number` | `3` | `CharacterEvent.kind`: a jump off a wall. |
| <a id="const-CHARACTER_AIR_JUMPED"></a>`CHARACTER_AIR_JUMPED` | `number` | `4` | `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps). |
| <a id="const-CHARACTER_LEFT_GROUND"></a>`CHARACTER_LEFT_GROUND` | `number` | `5` | `CharacterEvent.kind`: left the ground without jumping (ran off an edge). |
| <a id="const-CHARACTER_BUMPED"></a>`CHARACTER_BUMPED` | `number` | `6` | `CharacterEvent.kind`: hit a ceiling while rising. |
| <a id="const-DRIVE_REAR"></a>`DRIVE_REAR` | `number` | `0` | `Vehicle.drive`: the rear wheels drive. |
| <a id="const-DRIVE_FRONT"></a>`DRIVE_FRONT` | `number` | `1` | `Vehicle.drive`: the front wheels drive. |
| <a id="const-DRIVE_ALL"></a>`DRIVE_ALL` | `number` | `2` | `Vehicle.drive`: every wheel drives. |
| <a id="const-PIN_TOP"></a>`PIN_TOP` | `number` | `1` | `Cloth.pins` bit: the whole top row is pinned. |
| <a id="const-PIN_TOP_CORNERS"></a>`PIN_TOP_CORNERS` | `number` | `2` | `Cloth.pins` bit: the top-left and top-right corners are pinned. |
| <a id="const-PIN_LEFT"></a>`PIN_LEFT` | `number` | `4` | `Cloth.pins` bit: the whole left column is pinned (a flag on a pole). |
| <a id="const-PIN_TOP_RINGS"></a>`PIN_TOP_RINGS` | `number` | `8` | `Cloth.pins` bit: every fourth point of the top row (curtain rings). |

```ogt
const BODY_DYNAMIC u32 0 "RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick."
```

<sub>`api.ogt` line 3549</sub>

```ogt
const BODY_STATIC u32 1 "RigidBody.kind: never moves; collides with dynamic bodies."
```

<sub>`api.ogt` line 3550</sub>

```ogt
const BODY_KINEMATIC u32 2 "RigidBody.kind: moved by game code through its Transform; its Velocity, if any, is how fast it pushes."
```

<sub>`api.ogt` line 3551</sub>

```ogt
const LOCK_TRANSLATION_X u32 1 "RigidBody.lock bit: no translation along x."
```

<sub>`api.ogt` line 3552</sub>

```ogt
const LOCK_TRANSLATION_Y u32 2 "RigidBody.lock bit: no translation along y."
```

<sub>`api.ogt` line 3553</sub>

```ogt
const LOCK_TRANSLATION_Z u32 4 "RigidBody.lock bit: no translation along z."
```

<sub>`api.ogt` line 3554</sub>

```ogt
const LOCK_ROTATION_X u32 8 "RigidBody.lock bit: no rotation about x."
```

<sub>`api.ogt` line 3555</sub>

```ogt
const LOCK_ROTATION_Y u32 16 "RigidBody.lock bit: no rotation about y."
```

<sub>`api.ogt` line 3556</sub>

```ogt
const LOCK_ROTATION_Z u32 32 "RigidBody.lock bit: no rotation about z (the 2D profile's only rotation)."
```

<sub>`api.ogt` line 3557</sub>

```ogt
const SHAPE_BOX u32 0 "Collider.shape: a box of `size` (both profiles)."
```

<sub>`api.ogt` line 3558</sub>

```ogt
const SHAPE_SPHERE u32 1 "Collider.shape: a sphere of `radius` (a circle in 2D)."
```

<sub>`api.ogt` line 3559</sub>

```ogt
const SHAPE_CAPSULE u32 2 "Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles)."
```

<sub>`api.ogt` line 3560</sub>

```ogt
const SHAPE_CYLINDER u32 3 "Collider.shape: a cylinder along y of `radius` and `height` (3D only)."
```

<sub>`api.ogt` line 3561</sub>

```ogt
const SHAPE_CONVEX u32 4 "Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D)."
```

<sub>`api.ogt` line 3562</sub>

```ogt
const SHAPE_TRIMESH u32 5 "Collider.shape: the triangles of `mesh` (3D only; for static and kinematic bodies)."
```

<sub>`api.ogt` line 3563</sub>

```ogt
const SHAPE_HEIGHTFIELD u32 6 "Collider.shape: the height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry)."
```

<sub>`api.ogt` line 3564</sub>

```ogt
const PROFILE_2D u32 2 "Physics.profile: rapier2d, bodies in the xy plane rotating about z."
```

<sub>`api.ogt` line 3565</sub>

```ogt
const PROFILE_3D u32 3 "Physics.profile: rapier3d."
```

<sub>`api.ogt` line 3566</sub>

```ogt
const JOINT_FIXED u32 0 "`Joint.kind`: the two bodies move as one."
```

<sub>`api.ogt` line 3747</sub>

```ogt
const JOINT_REVOLUTE u32 1 "`Joint.kind`: a hinge about `axis` (2D: about z), with optional limits and a motor."
```

<sub>`api.ogt` line 3748</sub>

```ogt
const JOINT_PRISMATIC u32 2 "`Joint.kind`: a slider along `axis`, with optional limits and a motor."
```

<sub>`api.ogt` line 3749</sub>

```ogt
const JOINT_SPRING u32 3 "`Joint.kind`: a damped spring pulling the anchors to `restLength` apart."
```

<sub>`api.ogt` line 3750</sub>

```ogt
const JOINT_ROPE u32 4 "`Joint.kind`: a rope: the anchors stay at most `length` apart, free inside it."
```

<sub>`api.ogt` line 3751</sub>

```ogt
const JOINT_BALL u32 5 "`Joint.kind`: a ball and socket (3D; the 2D profile treats it as a hinge)."
```

<sub>`api.ogt` line 3752</sub>

```ogt
const MOTOR_NONE u32 0 "`Joint.motor`: no motor."
```

<sub>`api.ogt` line 3753</sub>

```ogt
const MOTOR_VELOCITY u32 1 "`Joint.motor`: drive towards `motorTarget` speed (rad/s or m/s)."
```

<sub>`api.ogt` line 3754</sub>

```ogt
const MOTOR_POSITION u32 2 "`Joint.motor`: spring towards `motorTarget` angle or position (a servo)."
```

<sub>`api.ogt` line 3755</sub>

```ogt
const CHARACTER_JUMPED u32 1 "`CharacterEvent.kind`: a jump from the ground (or in coyote time)."
```

<sub>`api.ogt` line 4091</sub>

```ogt
const CHARACTER_LANDED u32 2 "`CharacterEvent.kind`: touched the ground after being in the air."
```

<sub>`api.ogt` line 4092</sub>

```ogt
const CHARACTER_WALL_JUMPED u32 3 "`CharacterEvent.kind`: a jump off a wall."
```

<sub>`api.ogt` line 4093</sub>

```ogt
const CHARACTER_AIR_JUMPED u32 4 "`CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps)."
```

<sub>`api.ogt` line 4094</sub>

```ogt
const CHARACTER_LEFT_GROUND u32 5 "`CharacterEvent.kind`: left the ground without jumping (ran off an edge)."
```

<sub>`api.ogt` line 4095</sub>

```ogt
const CHARACTER_BUMPED u32 6 "`CharacterEvent.kind`: hit a ceiling while rising."
```

<sub>`api.ogt` line 4096</sub>

```ogt
const DRIVE_REAR u32 0 "`Vehicle.drive`: the rear wheels drive."
```

<sub>`api.ogt` line 4110</sub>

```ogt
const DRIVE_FRONT u32 1 "`Vehicle.drive`: the front wheels drive."
```

<sub>`api.ogt` line 4111</sub>

```ogt
const DRIVE_ALL u32 2 "`Vehicle.drive`: every wheel drives."
```

<sub>`api.ogt` line 4112</sub>

```ogt
const PIN_TOP u32 1 "`Cloth.pins` bit: the whole top row is pinned."
```

<sub>`api.ogt` line 4281</sub>

```ogt
const PIN_TOP_CORNERS u32 2 "`Cloth.pins` bit: the top-left and top-right corners are pinned."
```

<sub>`api.ogt` line 4282</sub>

```ogt
const PIN_LEFT u32 4 "`Cloth.pins` bit: the whole left column is pinned (a flag on a pole)."
```

<sub>`api.ogt` line 4283</sub>

```ogt
const PIN_TOP_RINGS u32 8 "`Cloth.pins` bit: every fourth point of the top row (curtain rings)."
```

<sub>`api.ogt` line 4284</sub>
