<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `physics` module

Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.

The profile is the project's (`profile "2d"` or `"3d"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.

The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.

- **TypeScript**: functions are the namespace `og.physics`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module physics {
  doc "Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back."
  doc "The profile is the project's (`profile \"2d\"` or `\"3d\"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform."
  doc "The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 8479</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`MAX_RAYCAST_HITS`](#const-MAX_RAYCAST_HITS) | constant |  | The most hits og.physics.raycastAll returns. |
| [`BODY_DYNAMIC`](#const-BODY_DYNAMIC) | constant |  | RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick. |
| [`BODY_STATIC`](#const-BODY_STATIC) | constant |  | RigidBody.kind: never moves; collides with dynamic bodies. |
| [`BODY_KINEMATIC`](#const-BODY_KINEMATIC) | constant |  | RigidBody.kind: not moved by the solver. |
| [`LOCK_TRANSLATION_X`](#const-LOCK_TRANSLATION_X) | constant |  | RigidBody.lock bit: no translation along x. |
| [`LOCK_TRANSLATION_Y`](#const-LOCK_TRANSLATION_Y) | constant |  | RigidBody.lock bit: no translation along y. |
| [`LOCK_TRANSLATION_Z`](#const-LOCK_TRANSLATION_Z) | constant |  | RigidBody.lock bit: no translation along z. |
| [`LOCK_ROTATION_X`](#const-LOCK_ROTATION_X) | constant |  | RigidBody.lock bit: no rotation about x. |
| [`LOCK_ROTATION_Y`](#const-LOCK_ROTATION_Y) | constant |  | RigidBody.lock bit: no rotation about y. |
| [`LOCK_ROTATION_Z`](#const-LOCK_ROTATION_Z) | constant |  | RigidBody.lock bit: no rotation about z (the 2D profile's only rotation). |
| [`SHAPE_BOX`](#const-SHAPE_BOX) | constant |  | Collider.shape: a box of `size` (both profiles). |
| [`SHAPE_SPHERE`](#const-SHAPE_SPHERE) | constant |  | Collider.shape: a sphere of `radius` (a circle in 2D). |
| [`SHAPE_CAPSULE`](#const-SHAPE_CAPSULE) | constant |  | Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles). |
| [`SHAPE_CYLINDER`](#const-SHAPE_CYLINDER) | constant |  | Collider.shape: a cylinder along y of `radius` and `height` (3D only). |
| [`SHAPE_CONVEX`](#const-SHAPE_CONVEX) | constant |  | Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D). |
| [`SHAPE_TRIMESH`](#const-SHAPE_TRIMESH) | constant |  | Collider.shape: the triangles of `mesh` (for static and kinematic bodies); in 2D the outline of their xy (the edges no two triangles share) as a hollow polyline, which stops what comes from outside. |
| [`SHAPE_HEIGHTFIELD`](#const-SHAPE_HEIGHTFIELD) | constant |  | Collider.shape: the height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry). |
| [`SHAPE_DIAMOND`](#const-SHAPE_DIAMOND) | constant |  | Collider.shape: a rhombus whose diagonals are size.x (along x) and size.y (along y), an isometric tile's diamond; in 3D a prism size.z deep (both profiles). |
| [`SHAPE_HEXAGON`](#const-SHAPE_HEXAGON) | constant |  | Collider.shape: a pointy-top hexagon in the box size.x by size.y, corners at the top and bottom middles and a quarter down each side, a hex tile's (turn it a quarter about z for a flat-top one); in 3… |
| [`PROFILE_2D`](#const-PROFILE_2D) | constant |  | Physics.profile: rapier2d, bodies in the xy plane rotating about z. |
| [`PROFILE_3D`](#const-PROFILE_3D) | constant |  | Physics.profile: rapier3d. |
| [`og.RigidBody`](physics/record-RigidBody.md) | component |  | A body the physics step simulates. |
| [`og.Collider`](physics/record-Collider.md) | component |  | The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). |
| [`og.Velocity`](physics/record-Velocity.md) | component |  | Linear and angular velocity. |
| [`og.Physics`](physics/record-Physics.md) | resource |  | The physics settings of the world. |
| [`og.CollisionStarted`](physics/record-CollisionStarted.md) | event |  | Two solid colliders started touching during the tick. |
| [`og.CollisionEnded`](physics/record-CollisionEnded.md) | event |  | Two solid colliders stopped touching, or one of them went away. |
| [`og.SensorEntered`](physics/record-SensorEntered.md) | event |  | A collider started overlapping a sensor. |
| [`og.SensorExited`](physics/record-SensorExited.md) | event |  | A collider stopped overlapping a sensor, or one of them went away. |
| [`og.JointBroken`](physics/record-JointBroken.md) | event |  | A joint broke during the tick: the solver's force or torque on it passed its breakForce or breakTorque. |
| [`og.ContactHook`](physics/record-ContactHook.md) | component |  | What a collider's contacts do beyond its og.Collider: a one-way platform, a conveyor belt, friction and restitution of its own, entities it passes through. |
| [`og.BodySlept`](physics/record-BodySlept.md) | event |  | A body fell asleep during the tick (Physics.sleepTicks), with every dynamic body it touches; sent in entity order. |
| [`og.BodyWoke`](physics/record-BodyWoke.md) | event |  | A sleeping body woke during the tick: something touched, pushed or moved it, game code wrote to it, or sleep was turned off; sent in entity order. |
| [`og.Contact`](physics/record-Contact.md) | event |  | One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now. |
| [`JOINT_FIXED`](#const-JOINT_FIXED) | constant |  | `Joint.kind`: the two bodies move as one. |
| [`JOINT_REVOLUTE`](#const-JOINT_REVOLUTE) | constant |  | `Joint.kind`: a hinge about `axis` (2D: about z), with optional limits and a motor. |
| [`JOINT_PRISMATIC`](#const-JOINT_PRISMATIC) | constant |  | `Joint.kind`: a slider along `axis`, with optional limits and a motor. |
| [`JOINT_SPRING`](#const-JOINT_SPRING) | constant |  | `Joint.kind`: a damped spring pulling the anchors to `restLength` apart. |
| [`JOINT_ROPE`](#const-JOINT_ROPE) | constant |  | `Joint.kind`: a rope: the anchors stay at most `length` apart, free inside it. |
| [`JOINT_BALL`](#const-JOINT_BALL) | constant |  | `Joint.kind`: a ball and socket (3D; the 2D profile treats it as a hinge). |
| [`MOTOR_NONE`](#const-MOTOR_NONE) | constant |  | `Joint.motor`: no motor. |
| [`MOTOR_VELOCITY`](#const-MOTOR_VELOCITY) | constant |  | `Joint.motor`: drive towards `motorTarget` speed (rad/s or m/s). |
| [`MOTOR_POSITION`](#const-MOTOR_POSITION) | constant |  | `Joint.motor`: spring towards `motorTarget` angle or position (a servo). |
| [`og.Joint`](physics/record-Joint.md) | component |  | A joint between two bodies (or a body and a point of the world). |
| [`og.CharacterController`](physics/record-CharacterController.md) | component |  | A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, sn… |
| [`og.PlatformerController`](physics/record-PlatformerController.md) | component |  | A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height (releasing jump while… |
| [`CHARACTER_JUMPED`](#const-CHARACTER_JUMPED) | constant |  | `CharacterEvent.kind`: a jump from the ground (or in coyote time). |
| [`CHARACTER_LANDED`](#const-CHARACTER_LANDED) | constant |  | `CharacterEvent.kind`: touched the ground after being in the air. |
| [`CHARACTER_WALL_JUMPED`](#const-CHARACTER_WALL_JUMPED) | constant |  | `CharacterEvent.kind`: a jump off a wall. |
| [`CHARACTER_AIR_JUMPED`](#const-CHARACTER_AIR_JUMPED) | constant |  | `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps). |
| [`CHARACTER_LEFT_GROUND`](#const-CHARACTER_LEFT_GROUND) | constant |  | `CharacterEvent.kind`: left the ground without jumping (ran off an edge). |
| [`CHARACTER_BUMPED`](#const-CHARACTER_BUMPED) | constant |  | `CharacterEvent.kind`: hit a ceiling while rising. |
| [`og.CharacterEvent`](physics/record-CharacterEvent.md) | event |  | Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. |
| [`DRIVE_REAR`](#const-DRIVE_REAR) | constant |  | `Vehicle.drive`: the rear wheels drive. |
| [`DRIVE_FRONT`](#const-DRIVE_FRONT) | constant |  | `Vehicle.drive`: the front wheels drive. |
| [`DRIVE_ALL`](#const-DRIVE_ALL) | constant |  | `Vehicle.drive`: every wheel drives. |
| [`VEHICLE_ARCADE`](#const-VEHICLE_ARCADE) | constant |  | `Vehicle.model`: the arcade car: engineForce, faded out towards maxSpeed. |
| [`VEHICLE_DRIVETRAIN`](#const-VEHICLE_DRIVETRAIN) | constant |  | `Vehicle.model`: an engine with a torque curve driving through a gearbox. |
| [`og.Vehicle`](physics/record-Vehicle.md) | component |  | A car on a dynamic chassis (the entity's og.RigidBody and og.Collider). |
| [`og.VehicleWheel`](physics/record-VehicleWheel.md) | component |  | A wheel of the og.Vehicle it is a child of (down its og.Parent chain), at its og.Transform relative to the chassis: the hard point its suspension hangs from. |
| [`og.Rope`](physics/record-Rope.md) | component |  | A Verlet rope: `segments` links from the entity's world position (its og.Transform after its parents'; plus `offset`) to `end` (an entity, plus `endOffset`) or hanging free, swung by gravity and wind… |
| [`SIMULATION_GAMEPLAY`](#const-SIMULATION_GAMEPLAY) | constant |  | `Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: simulated in the physics step on the CPU, deterministic: its points are world state that snapshots, replays, lockstep and the hash car… |
| [`SIMULATION_VISUAL`](#const-SIMULATION_VISUAL) | constant |  | `Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: simulated by the renderer on the GPU (XPBD in small steps), for looks only, never read back. |
| [`PIN_TOP`](#const-PIN_TOP) | constant |  | `Cloth.pins` bit: the whole top row is pinned. |
| [`PIN_TOP_CORNERS`](#const-PIN_TOP_CORNERS) | constant |  | `Cloth.pins` bit: the top-left and top-right corners are pinned. |
| [`PIN_LEFT`](#const-PIN_LEFT) | constant |  | `Cloth.pins` bit: the whole left column is pinned (a flag on a pole). |
| [`PIN_TOP_RINGS`](#const-PIN_TOP_RINGS) | constant |  | `Cloth.pins` bit: every fourth point of the top row (curtain rings). |
| [`og.Cloth`](physics/record-Cloth.md) | component |  | A Verlet cloth: a `columns` × `rows` grid of points, `spacing` apart, hanging from the entity's world position (its og.Transform after its parents': its top-left corner, the grid laid out along the e… |
| [`og.Garment`](physics/record-Garment.md) | component |  | Cloth on a skinned character: glTF mesh `mesh` of the entity's og.Model (a skirt, a cape, a coat's tails), posed by the entity's og.Skeleton and simulated from there. |
| [`SOFT_SPHERE`](#const-SOFT_SPHERE) | constant |  | `SoftBody.shape`: a sphere of `radius` (an icosphere; 2D a ring). |
| [`SOFT_BOX`](#const-SOFT_BOX) | constant |  | `SoftBody.shape`: a box of `size` (each face a grid of `detail` + 1 cells; 2D a rectangle). |
| [`SOFT_MESH`](#const-SOFT_MESH) | constant |  | `SoftBody.shape`: the closed surface of geometry `mesh` (3D only). |
| [`og.SoftBody`](physics/record-SoftBody.md) | component |  | A PBD soft volume: a closed surface of points (2D: a ring in the xy plane) held by distance constraints along its edges and by one constraint on the volume it encloses (2D: its area), dropped under g… |
| [`og.Fluid2D`](physics/record-Fluid2D.md) | component |  | A 2D particle fluid (any profile, in the xy plane): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. |
| [`og.Fluid3D`](physics/record-Fluid3D.md) | component |  | A 3D particle fluid (the 3D profile): `count` particles spawned in a `size` box centred on the entity, simulated with double density relaxation (Clavet et al. |
| [`og.WaterSurface`](physics/record-WaterSurface.md) | component |  | Height-field water (3D): a `resolution` × `resolution` grid of heights over a `size` square centred on the entity (its y is the rest level), waves travelling at `waveSpeed` and fading by `damping`. |
| [`og.WaterLine`](physics/record-WaterLine.md) | component |  | 2D water for side-scrollers: a `width` × `depth` pool whose top edge (at the entity's y, centred on its x) is `columns` springs, each pulled to its rest height by `tension`, damped by `damping` and p… |
| [`og.Buoyancy`](physics/record-Buoyancy.md) | component |  | Floats a dynamic rigid body (3D profile) on the water under it: an og.WaterSurface (at its rest level), an og.Water (a flat lake, or a sea with a wind: its waves as drawn, og.physics.waterHeight's) o… |
| [`og.Splash`](physics/record-Splash.md) | event |  | Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. |
| [`og.Ragdoll`](physics/record-Ragdoll.md) | component |  | A ragdoll over an og.Skeleton: a capsule body per bone with a parent, joints with limits between them (ball joints with cone limits in 3D; in 2D hinges about z, the bones rapier2d bodies in the xy pl… |
| [`QueryFilter`](physics/type-QueryFilter.md) | desc struct |  | Which colliders a scene query considers. |
| [`RaycastDesc`](physics/type-RaycastDesc.md) | desc struct |  | A ray for og.physics.raycast. |
| [`RaycastHit`](physics/type-RaycastHit.md) | struct |  | The nearest collider a ray meets. |
| [`RaycastAllDesc`](physics/type-RaycastAllDesc.md) | desc struct |  | A ray for og.physics.raycastAll: og.physics.raycast's fields and a cap on the hits. |
| [`ProjectPointDesc`](physics/type-ProjectPointDesc.md) | desc struct |  | A point for og.physics.projectPoint. |
| [`PointProjection`](physics/type-PointProjection.md) | struct |  | Where a point projects onto the nearest collider. |
| [`ShapeDesc`](physics/type-ShapeDesc.md) | desc struct |  | The shape of a shape cast or an overlap test: the shape fields of a Collider. |
| [`ShapeCastDesc`](physics/type-ShapeCastDesc.md) | desc struct |  | A shape swept along a direction for og.physics.shapeCast. |
| [`ShapeCastHit`](physics/type-ShapeCastHit.md) | struct |  | The first collider a swept shape meets. |
| [`OverlapDesc`](physics/type-OverlapDesc.md) | desc struct |  | A shape placed in the world for og.physics.overlap. |
| [`MeshDesc`](physics/type-MeshDesc.md) | desc struct |  | Geometry for convex and triangle-mesh colliders, in the mesh's own space. |
| [`og.physics.raycast`](physics/fn-raycast.md) | function | `(world: World, desc?: RaycastDesc): Result<RaycastHit \| null>` | The nearest collider along a ray. |
| [`og.physics.raycastAll`](physics/fn-raycastAll.md) | function | `(world: World, desc?: RaycastAllDesc): Result<readonly RaycastHit[]>` | Every collider along a ray within its reach, one hit each at its first crossing, nearest first (ties in collider entity order, so every host and run list them alike), at most `maxHits`: what a pierci… |
| [`og.physics.projectPoint`](physics/fn-projectPoint.md) | function | `(world: World, desc?: ProjectPointDesc): Result<PointProjection \| null>` | The nearest point on any collider to a point, within maxDistance: snapping to a surface, the closest cover, whether a point is inside a solid. |
| [`og.physics.shapeCast`](physics/fn-shapeCast.md) | function | `(world: World, desc?: ShapeCastDesc): Result<ShapeCastHit \| null>` | The first collider a shape meets when swept along a direction; the same view of the colliders as raycast. |
| [`og.physics.overlap`](physics/fn-overlap.md) | function | `(world: World, desc?: OverlapDesc): Result<readonly Entity[]>` | Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast. |
| [`og.physics.applyForce`](physics/fn-applyForce.md) | function | `(world: World, entity: Entity, force: Vector3, point?: Vector3): Result<void>` | Push a dynamic body with a force (N, world axes) over the next physics step, at a world point or its centre of mass: call it every tick for a steady force (thrust, wind, a car's downforce or an anti-… |
| [`og.physics.applyImpulse`](physics/fn-applyImpulse.md) | function | `(world: World, entity: Entity, impulse: Vector3, point?: Vector3): Result<void>` | Give a dynamic body an impulse (N·s, world axes) at a world point or its centre of mass: a sudden change of momentum (a kick, a hit, a jump: its velocity changes by impulse / mass), applied by the ne… |
| [`og.physics.applyTorque`](physics/fn-applyTorque.md) | function | `(world: World, entity: Entity, torque: Vector3): Result<void>` | Turn a dynamic body with a torque (N·m about each world axis; 2D uses z) over the next physics step, as applyForce: call it every tick for a steady twist. |
| [`og.physics.applyTorqueImpulse`](physics/fn-applyTorqueImpulse.md) | function | `(world: World, entity: Entity, torque: Vector3): Result<void>` | Give a dynamic body an angular impulse (N·m·s about each world axis; 2D uses z): its spin changes at once, as applyImpulse. |
| [`og.physics.setMesh`](physics/fn-setMesh.md) | function | `(world: World, mesh: Geometry, desc?: MeshDesc): Result<void>` | Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. |
| [`WheelPose`](physics/type-WheelPose.md) | struct |  | A wheel of an og.Vehicle as the last step left it, for drawing. |
| [`RagdollBoneDesc`](physics/type-RagdollBoneDesc.md) | desc struct |  | One bone of a ragdoll rig: a capsule from the bone's position along `axis` for `length`. |
| [`RagdollRigDesc`](physics/type-RagdollRigDesc.md) | desc struct |  | The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton. |
| [`BonePose`](physics/type-BonePose.md) | struct |  | A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight). |
| [`og.physics.ropePoints`](physics/fn-ropePoints.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Rope's points after the last step, from its entity to its end (empty before its first step). |
| [`og.physics.garmentPoints`](physics/fn-garmentPoints.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Garment's points after the last step, in the world: one per position of the garment's mesh (vertices at the same position, split along a seam, share one), in the order the mesh first uses them. |
| [`og.physics.clothPoints`](physics/fn-clothPoints.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge). |
| [`SoftBodyPoints`](physics/type-SoftBodyPoints.md) | struct |  | An og.SoftBody's surface after the last step. |
| [`og.physics.softBodyPoints`](physics/fn-softBodyPoints.md) | function | `(world: World, entity: Entity): Result<SoftBodyPoints>` | An og.SoftBody's points and the triangles of its surface after the last step (none before its first step). |
| [`og.physics.fluidParticles`](physics/fn-fluidParticles.md) | function | `(world: World, entity: Entity): Result<readonly Vector2[]>` | An og.Fluid2D's particle positions after the last step, in world space. |
| [`og.physics.fluid3dParticles`](physics/fn-fluid3dParticles.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared). |
| [`og.physics.waterHeight`](physics/fn-waterHeight.md) | function | `(world: World, entity: Entity, x: number, z: number): Result<number>` | The surface height of an og.WaterSurface (world y) or an og.WaterLine (world y at `x`; `z` ignored) at a point, as the last step left it, plus an og.Water's sea (og.Water.wind) at this tick, the surf… |
| [`og.physics.vehicleWheels`](physics/fn-vehicleWheels.md) | function | `(world: World, entity: Entity): Result<readonly WheelPose[]>` | An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right; or its og.VehicleWheel children in entity order), for drawing wheel models. |
| [`og.physics.ragdollRigFromSkeleton`](physics/fn-ragdollRigFromSkeleton.md) | function | `(world: World, skeleton: SkeletonAsset): Result<RagdollRigDesc>` | A ragdoll rig from a loaded skeleton (og.load.animation's `skeleton`): a bone per joint of its first skin, parents first, each from its joint's rest pose to its first child joint, every bone naming i… |
| [`og.physics.setRagdollRig`](physics/fn-setRagdollRig.md) | function | `(world: World, entity: Entity, desc?: RagdollRigDesc): Result<void>` | Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ra… |
| [`og.physics.ragdollPose`](physics/fn-ragdollPose.md) | function | `(world: World, entity: Entity): Result<readonly BonePose[]>` | An og.Ragdoll's bones after the last step, in rig order (empty before its first step). |
| [`PhysicsPhase`](physics/type-PhysicsPhase.md) | struct |  | One phase of the last physics step, for og.physics.stats. |
| [`PhysicsStats`](physics/type-PhysicsStats.md) | struct |  | What the last physics step cost, phase by phase, and where it ran: for finding where physics time goes. |
| [`og.physics.stats`](physics/fn-stats.md) | function | `(world: World): Result<PhysicsStats>` | What the last physics step cost: each phase's time and whether it ran on the lanes pool. |

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_RAYCAST_HITS"></a>`MAX_RAYCAST_HITS` | `number` | `256` | The most hits og.physics.raycastAll returns. |
| <a id="const-BODY_DYNAMIC"></a>`BODY_DYNAMIC` | `number` | `0` | RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick. |
| <a id="const-BODY_STATIC"></a>`BODY_STATIC` | `number` | `1` | RigidBody.kind: never moves; collides with dynamic bodies. |
| <a id="const-BODY_KINEMATIC"></a>`BODY_KINEMATIC` | `number` | `2` | RigidBody.kind: not moved by the solver. With an og.Velocity the step moves it by it each tick and writes its Transform back (a lift, a conveyor, a turning gear); a Transform game code writes moves it there first (a teleport). Without one it stays where its Transform puts it. Either way it pushes dynamic bodies and carries the controllers standing on it. |
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
| <a id="const-SHAPE_TRIMESH"></a>`SHAPE_TRIMESH` | `number` | `5` | Collider.shape: the triangles of `mesh` (for static and kinematic bodies); in 2D the outline of their xy (the edges no two triangles share) as a hollow polyline, which stops what comes from outside. |
| <a id="const-SHAPE_HEIGHTFIELD"></a>`SHAPE_HEIGHTFIELD` | `number` | `6` | Collider.shape: the height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry). |
| <a id="const-SHAPE_DIAMOND"></a>`SHAPE_DIAMOND` | `number` | `7` | Collider.shape: a rhombus whose diagonals are size.x (along x) and size.y (along y), an isometric tile's diamond; in 3D a prism size.z deep (both profiles). |
| <a id="const-SHAPE_HEXAGON"></a>`SHAPE_HEXAGON` | `number` | `8` | Collider.shape: a pointy-top hexagon in the box size.x by size.y, corners at the top and bottom middles and a quarter down each side, a hex tile's (turn it a quarter about z for a flat-top one); in 3D a prism size.z deep (both profiles). |
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
| <a id="const-VEHICLE_ARCADE"></a>`VEHICLE_ARCADE` | `number` | `0` | `Vehicle.model`: the arcade car: engineForce, faded out towards maxSpeed. |
| <a id="const-VEHICLE_DRIVETRAIN"></a>`VEHICLE_DRIVETRAIN` | `number` | `1` | `Vehicle.model`: an engine with a torque curve driving through a gearbox. |
| <a id="const-SIMULATION_GAMEPLAY"></a>`SIMULATION_GAMEPLAY` | `number` | `0` | `Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: simulated in the physics step on the CPU, deterministic: its points are world state that snapshots, replays, lockstep and the hash carry, and that og.physics.clothPoints, softBodyPoints and garmentPoints read. The default. |
| <a id="const-SIMULATION_VISUAL"></a>`SIMULATION_VISUAL` | `number` | `1` | `Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: simulated by the renderer on the GPU (XPBD in small steps), for looks only, never read back. The tick leaves it unsolved: its points (og.physics.clothPoints, softBodyPoints, garmentPoints) are its rest shape carried rigidly by the entity (a garment's: the skinned pose), SoftBody.volume stays 1 and SoftBody.center is that shape's mean, so gameplay reads a stable proxy and a headless server pays nothing. The drawn motion runs on tick time: frames that skip ticks step those too, up to eight, and a world restored, rolled back or more than eight ticks ahead starts again from the rest shape. It meets the colliders drawn this frame (og.Collider boxes, spheres, capsules and height fields; convex, cylinder and triangle-mesh shapes are passed through) and, with thickness, itself. A visual cloth's or garment's wind is air moving along `wind` at its magnitude in metres a second, meeting each triangle with drag and lift against the triangle's own motion and scaled so a still face square to it feels `wind`; `gust` sends bands of stronger, veering air down it, so it ripples and flaps rather than leaning. Where RenderCapabilities.visualSimulation is false (no compute or no vertex storage) it draws that rest shape. |
| <a id="const-PIN_TOP"></a>`PIN_TOP` | `number` | `1` | `Cloth.pins` bit: the whole top row is pinned. |
| <a id="const-PIN_TOP_CORNERS"></a>`PIN_TOP_CORNERS` | `number` | `2` | `Cloth.pins` bit: the top-left and top-right corners are pinned. |
| <a id="const-PIN_LEFT"></a>`PIN_LEFT` | `number` | `4` | `Cloth.pins` bit: the whole left column is pinned (a flag on a pole). |
| <a id="const-PIN_TOP_RINGS"></a>`PIN_TOP_RINGS` | `number` | `8` | `Cloth.pins` bit: every fourth point of the top row (curtain rings). |
| <a id="const-SOFT_SPHERE"></a>`SOFT_SPHERE` | `number` | `0` | `SoftBody.shape`: a sphere of `radius` (an icosphere; 2D a ring). |
| <a id="const-SOFT_BOX"></a>`SOFT_BOX` | `number` | `1` | `SoftBody.shape`: a box of `size` (each face a grid of `detail` + 1 cells; 2D a rectangle). |
| <a id="const-SOFT_MESH"></a>`SOFT_MESH` | `number` | `2` | `SoftBody.shape`: the closed surface of geometry `mesh` (3D only). |

```ogt
const MAX_RAYCAST_HITS u32 256 "The most hits og.physics.raycastAll returns."
```

<sub>`api.ogt` line 8484</sub>

```ogt
const BODY_DYNAMIC u32 0 "RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick."
```

<sub>`api.ogt` line 8485</sub>

```ogt
const BODY_STATIC u32 1 "RigidBody.kind: never moves; collides with dynamic bodies."
```

<sub>`api.ogt` line 8486</sub>

```ogt
const BODY_KINEMATIC u32 2 "RigidBody.kind: not moved by the solver. With an og.Velocity the step moves it by it each tick and writes its Transform back (a lift, a conveyor, a turning gear); a Transform game code writes moves it there first (a teleport). Without one it stays where its Transform puts it. Either way it pushes dynamic bodies and carries the controllers standing on it."
```

<sub>`api.ogt` line 8487</sub>

```ogt
const LOCK_TRANSLATION_X u32 1 "RigidBody.lock bit: no translation along x."
```

<sub>`api.ogt` line 8488</sub>

```ogt
const LOCK_TRANSLATION_Y u32 2 "RigidBody.lock bit: no translation along y."
```

<sub>`api.ogt` line 8489</sub>

```ogt
const LOCK_TRANSLATION_Z u32 4 "RigidBody.lock bit: no translation along z."
```

<sub>`api.ogt` line 8490</sub>

```ogt
const LOCK_ROTATION_X u32 8 "RigidBody.lock bit: no rotation about x."
```

<sub>`api.ogt` line 8491</sub>

```ogt
const LOCK_ROTATION_Y u32 16 "RigidBody.lock bit: no rotation about y."
```

<sub>`api.ogt` line 8492</sub>

```ogt
const LOCK_ROTATION_Z u32 32 "RigidBody.lock bit: no rotation about z (the 2D profile's only rotation)."
```

<sub>`api.ogt` line 8493</sub>

```ogt
const SHAPE_BOX u32 0 "Collider.shape: a box of `size` (both profiles)."
```

<sub>`api.ogt` line 8494</sub>

```ogt
const SHAPE_SPHERE u32 1 "Collider.shape: a sphere of `radius` (a circle in 2D)."
```

<sub>`api.ogt` line 8495</sub>

```ogt
const SHAPE_CAPSULE u32 2 "Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles)."
```

<sub>`api.ogt` line 8496</sub>

```ogt
const SHAPE_CYLINDER u32 3 "Collider.shape: a cylinder along y of `radius` and `height` (3D only)."
```

<sub>`api.ogt` line 8497</sub>

```ogt
const SHAPE_CONVEX u32 4 "Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D)."
```

<sub>`api.ogt` line 8498</sub>

```ogt
const SHAPE_TRIMESH u32 5 "Collider.shape: the triangles of `mesh` (for static and kinematic bodies); in 2D the outline of their xy (the edges no two triangles share) as a hollow polyline, which stops what comes from outside."
```

<sub>`api.ogt` line 8499</sub>

```ogt
const SHAPE_HEIGHTFIELD u32 6 "Collider.shape: the height field registered as `mesh`, centred on the entity (3D only; a terrain: og.terrain.load's TerrainInfo.geometry)."
```

<sub>`api.ogt` line 8500</sub>

```ogt
const SHAPE_DIAMOND u32 7 "Collider.shape: a rhombus whose diagonals are size.x (along x) and size.y (along y), an isometric tile's diamond; in 3D a prism size.z deep (both profiles)."
```

<sub>`api.ogt` line 8501</sub>

```ogt
const SHAPE_HEXAGON u32 8 "Collider.shape: a pointy-top hexagon in the box size.x by size.y, corners at the top and bottom middles and a quarter down each side, a hex tile's (turn it a quarter about z for a flat-top one); in 3D a prism size.z deep (both profiles)."
```

<sub>`api.ogt` line 8502</sub>

```ogt
const PROFILE_2D u32 2 "Physics.profile: rapier2d, bodies in the xy plane rotating about z."
```

<sub>`api.ogt` line 8503</sub>

```ogt
const PROFILE_3D u32 3 "Physics.profile: rapier3d."
```

<sub>`api.ogt` line 8504</sub>

```ogt
const JOINT_FIXED u32 0 "`Joint.kind`: the two bodies move as one."
```

<sub>`api.ogt` line 8761</sub>

```ogt
const JOINT_REVOLUTE u32 1 "`Joint.kind`: a hinge about `axis` (2D: about z), with optional limits and a motor."
```

<sub>`api.ogt` line 8762</sub>

```ogt
const JOINT_PRISMATIC u32 2 "`Joint.kind`: a slider along `axis`, with optional limits and a motor."
```

<sub>`api.ogt` line 8763</sub>

```ogt
const JOINT_SPRING u32 3 "`Joint.kind`: a damped spring pulling the anchors to `restLength` apart."
```

<sub>`api.ogt` line 8764</sub>

```ogt
const JOINT_ROPE u32 4 "`Joint.kind`: a rope: the anchors stay at most `length` apart, free inside it."
```

<sub>`api.ogt` line 8765</sub>

```ogt
const JOINT_BALL u32 5 "`Joint.kind`: a ball and socket (3D; the 2D profile treats it as a hinge)."
```

<sub>`api.ogt` line 8766</sub>

```ogt
const MOTOR_NONE u32 0 "`Joint.motor`: no motor."
```

<sub>`api.ogt` line 8767</sub>

```ogt
const MOTOR_VELOCITY u32 1 "`Joint.motor`: drive towards `motorTarget` speed (rad/s or m/s)."
```

<sub>`api.ogt` line 8768</sub>

```ogt
const MOTOR_POSITION u32 2 "`Joint.motor`: spring towards `motorTarget` angle or position (a servo)."
```

<sub>`api.ogt` line 8769</sub>

```ogt
const CHARACTER_JUMPED u32 1 "`CharacterEvent.kind`: a jump from the ground (or in coyote time)."
```

<sub>`api.ogt` line 9136</sub>

```ogt
const CHARACTER_LANDED u32 2 "`CharacterEvent.kind`: touched the ground after being in the air."
```

<sub>`api.ogt` line 9137</sub>

```ogt
const CHARACTER_WALL_JUMPED u32 3 "`CharacterEvent.kind`: a jump off a wall."
```

<sub>`api.ogt` line 9138</sub>

```ogt
const CHARACTER_AIR_JUMPED u32 4 "`CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps)."
```

<sub>`api.ogt` line 9139</sub>

```ogt
const CHARACTER_LEFT_GROUND u32 5 "`CharacterEvent.kind`: left the ground without jumping (ran off an edge)."
```

<sub>`api.ogt` line 9140</sub>

```ogt
const CHARACTER_BUMPED u32 6 "`CharacterEvent.kind`: hit a ceiling while rising."
```

<sub>`api.ogt` line 9141</sub>

```ogt
const DRIVE_REAR u32 0 "`Vehicle.drive`: the rear wheels drive."
```

<sub>`api.ogt` line 9155</sub>

```ogt
const DRIVE_FRONT u32 1 "`Vehicle.drive`: the front wheels drive."
```

<sub>`api.ogt` line 9156</sub>

```ogt
const DRIVE_ALL u32 2 "`Vehicle.drive`: every wheel drives."
```

<sub>`api.ogt` line 9157</sub>

```ogt
const VEHICLE_ARCADE u32 0 "`Vehicle.model`: the arcade car: engineForce, faded out towards maxSpeed."
```

<sub>`api.ogt` line 9158</sub>

```ogt
const VEHICLE_DRIVETRAIN u32 1 "`Vehicle.model`: an engine with a torque curve driving through a gearbox."
```

<sub>`api.ogt` line 9159</sub>

```ogt
const SIMULATION_GAMEPLAY u32 0 "`Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: simulated in the physics step on the CPU, deterministic: its points are world state that snapshots, replays, lockstep and the hash carry, and that og.physics.clothPoints, softBodyPoints and garmentPoints read. The default."
```

<sub>`api.ogt` line 9461</sub>

```ogt
const SIMULATION_VISUAL u32 1 "`Cloth.simulation`, `SoftBody.simulation`, `Garment.simulation`: simulated by the renderer on the GPU (XPBD in small steps), for looks only, never read back. The tick leaves it unsolved: its points (og.physics.clothPoints, softBodyPoints, garmentPoints) are its rest shape carried rigidly by the entity (a garment's: the skinned pose), SoftBody.volume stays 1 and SoftBody.center is that shape's mean, so gameplay reads a stable proxy and a headless server pays nothing. The drawn motion runs on tick time: frames that skip ticks step those too, up to eight, and a world restored, rolled back or more than eight ticks ahead starts again from the rest shape. It meets the colliders drawn this frame (og.Collider boxes, spheres, capsules and height fields; convex, cylinder and triangle-mesh shapes are passed through) and, with thickness, itself. A visual cloth's or garment's wind is air moving along `wind` at its magnitude in metres a second, meeting each triangle with drag and lift against the triangle's own motion and scaled so a still face square to it feels `wind`; `gust` sends bands of stronger, veering air down it, so it ripples and flaps rather than leaning. Where RenderCapabilities.visualSimulation is false (no compute or no vertex storage) it draws that rest shape."
```

<sub>`api.ogt` line 9462</sub>

```ogt
const PIN_TOP u32 1 "`Cloth.pins` bit: the whole top row is pinned."
```

<sub>`api.ogt` line 9463</sub>

```ogt
const PIN_TOP_CORNERS u32 2 "`Cloth.pins` bit: the top-left and top-right corners are pinned."
```

<sub>`api.ogt` line 9464</sub>

```ogt
const PIN_LEFT u32 4 "`Cloth.pins` bit: the whole left column is pinned (a flag on a pole)."
```

<sub>`api.ogt` line 9465</sub>

```ogt
const PIN_TOP_RINGS u32 8 "`Cloth.pins` bit: every fourth point of the top row (curtain rings)."
```

<sub>`api.ogt` line 9466</sub>

```ogt
const SOFT_SPHERE u32 0 "`SoftBody.shape`: a sphere of `radius` (an icosphere; 2D a ring)."
```

<sub>`api.ogt` line 9629</sub>

```ogt
const SOFT_BOX u32 1 "`SoftBody.shape`: a box of `size` (each face a grid of `detail` + 1 cells; 2D a rectangle)."
```

<sub>`api.ogt` line 9630</sub>

```ogt
const SOFT_MESH u32 2 "`SoftBody.shape`: the closed surface of geometry `mesh` (3D only)."
```

<sub>`api.ogt` line 9631</sub>
