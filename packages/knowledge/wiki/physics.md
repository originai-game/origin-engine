<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `physics` module

Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.

The profile is 2D in a `form "2d"` project. In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.

The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.

See `physics`.

- **TypeScript**: functions are the namespace `og.physics`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.RigidBody`, `ColliderShape`, `ColliderShape()`, `og.Collider`, `og.Velocity`, `og.CharacterController`, `og.Vehicle`; 127 more are advanced (`og api show physics --all`).
- **Index**: [every module and guide](index.md).

```ogt
module physics {
  doc "Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back."
  doc "The profile is 2D in a `form \"2d\"` project. In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform."
  doc "The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1."
  # declarations follow, each on its own page
```

<sub>Module `physics` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`MAX_RAYCAST_HITS`](#const-MAX_RAYCAST_HITS) | constant |  | The most hits og.physics.castRayAll returns. |
| [`LOCK_TRANSLATION_X`](#const-LOCK_TRANSLATION_X) | constant |  | RigidBody.lock bit: no translation along x. |
| [`LOCK_TRANSLATION_Y`](#const-LOCK_TRANSLATION_Y) | constant |  | RigidBody.lock bit: no translation along y. |
| [`LOCK_TRANSLATION_Z`](#const-LOCK_TRANSLATION_Z) | constant |  | RigidBody.lock bit: no translation along z. |
| [`LOCK_ROTATION_X`](#const-LOCK_ROTATION_X) | constant |  | RigidBody.lock bit: no rotation about x. |
| [`LOCK_ROTATION_Y`](#const-LOCK_ROTATION_Y) | constant |  | RigidBody.lock bit: no rotation about y. |
| [`LOCK_ROTATION_Z`](#const-LOCK_ROTATION_Z) | constant |  | RigidBody.lock bit: no rotation about z (the 2D profile's only rotation). |
| [`PROFILE_2D`](#const-PROFILE_2D) | constant |  | Physics.profile: rapier2d, bodies in the xy plane rotating about z. |
| [`PROFILE_3D`](#const-PROFILE_3D) | constant |  | Physics.profile: rapier3d. |
| [`RigidBodyType`](physics/type-RigidBodyType.md) | enum |  | A body's simulation mode. |
| [`RigidBodyType()`](#column-RigidBodyType) | column kind |  | A RigidBodyType field: the variant's index. |
| [`og.RigidBody`](physics/record-RigidBody.md) | component |  | A body the physics step simulates. |
| [`ColliderShape`](physics/type-ColliderShape.md) | enum |  | What a collider is shaped as: Collider.shape and ShapeDesc.shape. |
| [`ColliderShape()`](#column-ColliderShape) | column kind |  | A ColliderShape field: the variant's index. |
| [`CombineRule`](physics/type-CombineRule.md) | enum |  | How two colliders' friction or restitution coefficients combine. |
| [`CombineRule()`](#column-CombineRule) | column kind |  | A CombineRule field: the variant's index. |
| [`og.Collider`](physics/record-Collider.md) | component |  | The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). |
| [`ForceFieldKind`](physics/type-ForceFieldKind.md) | enum |  | How an og.ForceField accelerates. |
| [`ForceFieldKind()`](#column-ForceFieldKind) | column kind |  | A ForceFieldKind field: the variant's index. |
| [`og.ForceField`](physics/record-ForceField.md) | component |  | An acceleration on what is inside the og.Volume on its entity (none: everywhere), whatever its mass, as gravity is. |
| [`og.Velocity`](physics/record-Velocity.md) | component |  | Linear and angular velocity. |
| [`og.Physics`](physics/record-Physics.md) | resource |  | The physics settings of the world. |
| [`og.CollisionStarted`](physics/record-CollisionStarted.md) | event |  | Two solid colliders started touching during the tick. |
| [`og.CollisionEnded`](physics/record-CollisionEnded.md) | event |  | Two solid colliders stopped touching, or one of them went away. |
| [`og.SensorEntered`](physics/record-SensorEntered.md) | event |  | A collider started overlapping a sensor: something walked into a trigger area (a pressure plate, a pickup, a zone that starts a cutscene). |
| [`og.SensorExited`](physics/record-SensorExited.md) | event |  | A collider stopped overlapping a sensor, or one of them went away. |
| [`og.JointBroken`](physics/record-JointBroken.md) | event |  | A joint broke during the tick: the solver's force or torque on it passed its breakForce or breakTorque. |
| [`og.ContactHook`](physics/record-ContactHook.md) | component |  | What a collider's contacts do beyond its og.Collider: a one-way platform, a conveyor belt, friction and restitution of its own, entities it passes through. |
| [`og.BodySlept`](physics/record-BodySlept.md) | event |  | A body fell asleep during the tick (Physics.sleepTicks), with every dynamic body it touches; sent in entity order. |
| [`og.BodyWoke`](physics/record-BodyWoke.md) | event |  | A sleeping body woke during the tick: something touched, pushed or moved it, game code wrote to it, or sleep was turned off; sent in entity order. |
| [`og.Contact`](physics/record-Contact.md) | event |  | One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now. |
| [`JointType`](physics/type-JointType.md) | enum |  | How a joint holds its two bodies: Joint.kind. |
| [`JointType()`](#column-JointType) | column kind |  | A JointType field: the variant's index. |
| [`JointMotor`](physics/type-JointMotor.md) | enum |  | What drives a joint: Joint.motor. |
| [`JointMotor()`](#column-JointMotor) | column kind |  | A JointMotor field: the variant's index. |
| [`JointMotorModel`](physics/type-JointMotorModel.md) | enum |  | How a joint motor interprets stiffness and damping: Joint.motorModel. |
| [`JointMotorModel()`](#column-JointMotorModel) | column kind |  | A JointMotorModel field: the variant's index. |
| [`og.Joint`](physics/record-Joint.md) | component |  | A joint between two bodies (or a body and a point of the world). |
| [`og.CharacterController`](physics/record-CharacterController.md) | component |  | A kinematic character (3D, or top-down 2D): the entity's capsule (or any og.Collider) moved by `move` each tick, sliding along walls, climbing slopes up to `maxSlope` and steps up to `stepHeight`, sn… |
| [`og.PlatformerController`](physics/record-PlatformerController.md) | component |  | A 2D platformer character (the 2D profile): run, jump and fall with designer units (jump height and time to apex, not gravity), coyote time, jump buffering, variable jump height, apex hang, air jumps… |
| [`CHARACTER_JUMPED`](#const-CHARACTER_JUMPED) | constant |  | `CharacterEvent.kind`: a jump from the ground (or in coyote time). |
| [`CHARACTER_LANDED`](#const-CHARACTER_LANDED) | constant |  | `CharacterEvent.kind`: touched the ground after being in the air. |
| [`CHARACTER_WALL_JUMPED`](#const-CHARACTER_WALL_JUMPED) | constant |  | `CharacterEvent.kind`: a jump off a wall. |
| [`CHARACTER_AIR_JUMPED`](#const-CHARACTER_AIR_JUMPED) | constant |  | `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps). |
| [`CHARACTER_LEFT_GROUND`](#const-CHARACTER_LEFT_GROUND) | constant |  | `CharacterEvent.kind`: left the ground without jumping (ran off an edge). |
| [`CHARACTER_BUMPED`](#const-CHARACTER_BUMPED) | constant |  | `CharacterEvent.kind`: hit a ceiling while rising. |
| [`CHARACTER_CRUSHED`](#const-CHARACTER_CRUSHED) | constant |  | `CharacterEvent.kind`: opposing solid surfaces crushed the character. |
| [`og.CharacterEvent`](physics/record-CharacterEvent.md) | event |  | Something a CharacterController or PlatformerController did this step: the hooks for dust, squash, sounds and camera kicks. |
| [`og.physics.resizeCharacter`](physics/fn-resizeCharacter.md) | function | `(world: World, entity: Entity, height: number, radius: number): Result<boolean>` | Resize a character capsule after checking headroom and the final volume. |
| [`VehicleDrive`](physics/type-VehicleDrive.md) | enum |  | Which wheels the engine drives: Vehicle.drive. |
| [`VehicleDrive()`](#column-VehicleDrive) | column kind |  | A VehicleDrive field: the variant's index. |
| [`VehicleModel`](physics/type-VehicleModel.md) | enum |  | How a vehicle's engine is modelled: Vehicle.model. |
| [`VehicleModel()`](#column-VehicleModel) | column kind |  | A VehicleModel field: the variant's index. |
| [`og.Vehicle`](physics/record-Vehicle.md) | component |  | Independent wheels on a dynamic chassis: forward +z in 3D, +x in 2D. |
| [`og.VehicleWheel`](physics/record-VehicleWheel.md) | component |  | A wheel of its nearest Vehicle ancestor in either profile. |
| [`VehicleWheelCast`](physics/type-VehicleWheelCast.md) | enum |  | The 3D wheel contact cast shape. |
| [`VehicleWheelCast()`](#column-VehicleWheelCast) | column kind |  | A VehicleWheelCast variant index. |
| [`VehicleDifferential`](physics/type-VehicleDifferential.md) | enum |  | Driven-wheel angular coupling. |
| [`VehicleDifferential()`](#column-VehicleDifferential) | column kind |  | A VehicleDifferential variant index. |
| [`og.Rope`](physics/record-Rope.md) | component |  | A small-step XPBD rope with explicit velocities and point masses. |
| [`Simulation`](physics/type-Simulation.md) | enum |  | Where a cloth, a garment, a soft body or a 3D fluid is simulated: Cloth.simulation, Garment.simulation, SoftBody.simulation and Fluid3D.simulation. |
| [`Simulation()`](#column-Simulation) | column kind |  | A Simulation field: the variant's index. |
| [`PIN_TOP`](#const-PIN_TOP) | constant |  | `Cloth.pins` bit: the whole top row is pinned. |
| [`PIN_TOP_CORNERS`](#const-PIN_TOP_CORNERS) | constant |  | `Cloth.pins` bit: the top-left and top-right corners are pinned. |
| [`PIN_LEFT`](#const-PIN_LEFT) | constant |  | `Cloth.pins` bit: the whole left column is pinned (a flag on a pole). |
| [`PIN_TOP_RINGS`](#const-PIN_TOP_RINGS) | constant |  | `Cloth.pins` bit: every fourth point of the top row (curtain rings). |
| [`og.Cloth`](physics/record-Cloth.md) | component |  | A small-step XPBD cloth: a columns by rows grid, pinned to its entity, with stretch, dihedral bending, long-range attachments, relative-air drag and lift, and self-collision. |
| [`og.Garment`](physics/record-Garment.md) | component |  | A skinned small-step XPBD garment with explicit velocities, area-lumped masses, dihedral bending and relative-air drag and lift. |
| [`SoftBodyShape`](physics/type-SoftBodyShape.md) | enum |  | What a soft body is built as: SoftBody.shape. |
| [`SoftBodyShape()`](#column-SoftBodyShape) | column kind |  | A SoftBodyShape field: the variant's index. |
| [`og.SoftBody`](physics/record-SoftBody.md) | component |  | An XPBD soft volume with explicit velocities, area-lumped masses, edge constraints and centroid-relative signed volume (2D: area). |
| [`og.Fluid2D`](physics/record-Fluid2D.md) | component |  | Deterministic 2D Position Based Fluids in the xy plane: predicted-position neighbour hashing, symmetric mass-weighted density projection, artificial pressure, stable XSPH viscosity and optional vorti… |
| [`og.Fluid3D`](physics/record-Fluid3D.md) | component |  | Deterministic 3D Position Based Fluids with predicted-position neighbour hashing, symmetric mass-weighted density projection, artificial pressure, stable XSPH viscosity and optional vorticity confine… |
| [`og.Ragdoll`](physics/record-Ragdoll.md) | component |  | An explicit rest-pose rig of shaped bones and limited joints. |
| [`og.WaterSurface`](physics/record-WaterSurface.md) | component |  | 3D height-field water centred on the entity at its rest level. |
| [`og.WaterLine`](physics/record-WaterLine.md) | component |  | 2D shallow water: a width × depth pool at the entity's y, centred on its x. |
| [`og.Buoyancy`](physics/record-Buoyancy.md) | component |  | Floats a rigid body in 2D or 3D, or a limp ragdoll's actual bone volumes; a 3D character swims. |
| [`og.Splash`](physics/record-Splash.md) | event |  | Something crossed a water surface (an og.Water, og.WaterSurface or og.WaterLine): for splash particles and sounds. |
| [`WaterPoint`](physics/type-WaterPoint.md) | struct |  | The water at a point (og.physics.water): whose it is, where its surface stands over the point and how it moves there. |
| [`og.physics.water`](physics/fn-water.md) | function | `(world: World, point: Vector3): Result<WaterPoint \| null>` | The highest water volume enclosing the world point, between its surface and depth below it: its entity, surface height, submerged depth, motion and slope. |
| [`og.physics.waterHeight`](physics/fn-waterHeight.md) | function | `(world: World, entity: Entity, x: number, z: number): Result<number>` | The water's surface at a point (world y), as it is drawn: an og.Water's level, with its sea's waves (og.Water.wind) or its river's course at this tick, plus the waves bodies raised on an og.WaterSurf… |
| [`QUERY_DYNAMIC`](#const-QUERY_DYNAMIC) | constant |  | QueryFilter.kindFlags bit: include dynamic bodies. |
| [`QUERY_FIXED`](#const-QUERY_FIXED) | constant |  | QueryFilter.kindFlags bit: include fixed bodies and static colliders. |
| [`QUERY_KINEMATIC`](#const-QUERY_KINEMATIC) | constant |  | QueryFilter.kindFlags bit: include kinematic bodies and controllers. |
| [`QueryFilter`](physics/type-QueryFilter.md) | desc struct |  | Which colliders a scene query considers. |
| [`RaycastDesc`](physics/type-RaycastDesc.md) | desc struct |  | A ray for og.physics.castRay. |
| [`RaycastHit`](physics/type-RaycastHit.md) | struct |  | The nearest collider a ray meets. |
| [`RaycastAllDesc`](physics/type-RaycastAllDesc.md) | desc struct |  | A ray for og.physics.castRayAll: og.physics.castRay's fields and a cap on the hits. |
| [`ProjectPointDesc`](physics/type-ProjectPointDesc.md) | desc struct |  | A point for og.physics.projectPoint. |
| [`PointProjection`](physics/type-PointProjection.md) | struct |  | Where a point projects onto the nearest collider. |
| [`ShapeDesc`](physics/type-ShapeDesc.md) | desc struct |  | The shape of a shape cast or an overlap test: the shape fields of a Collider. |
| [`ShapeCastDesc`](physics/type-ShapeCastDesc.md) | desc struct |  | A shape swept along a direction for og.physics.castShape. |
| [`ShapeCastAllDesc`](physics/type-ShapeCastAllDesc.md) | desc struct |  | A shape swept along a direction for og.physics.castShapeAll: castShape's fields and a cap on the hits. |
| [`ShapeCastHit`](physics/type-ShapeCastHit.md) | struct |  | A collider a swept shape meets. |
| [`OverlapDesc`](physics/type-OverlapDesc.md) | desc struct |  | A shape placed in the world for og.physics.overlap. |
| [`PointOverlapDesc`](physics/type-PointOverlapDesc.md) | desc struct |  | A point placed in the world for og.physics.overlapPoint. |
| [`MassProperties`](physics/type-MassProperties.md) | struct |  | A body's mass properties as computed by the solver. |
| [`MeshDesc`](physics/type-MeshDesc.md) | desc struct |  | Geometry for convex, triangle-mesh and polyline colliders, in the mesh's own space. |
| [`og.physics.castRay`](physics/fn-castRay.md) | function | `(world: World, desc?: RaycastDesc): Result<RaycastHit \| null>` | The nearest collider along a ray in the live Rapier simulation. |
| [`og.physics.castRayAll`](physics/fn-castRayAll.md) | function | `(world: World, desc?: RaycastAllDesc): Result<readonly RaycastHit[]>` | Every collider along a ray within its reach, one hit each at its first crossing, nearest first (ties in collider entity order, so every host and run list them alike), at most `maxHits`: what a pierci… |
| [`og.physics.projectPoint`](physics/fn-projectPoint.md) | function | `(world: World, desc?: ProjectPointDesc): Result<PointProjection \| null>` | The nearest point on any collider to a point, within maxDistance: snapping to a surface, the closest cover, whether a point is inside a solid. |
| [`og.physics.castShape`](physics/fn-castShape.md) | function | `(world: World, desc?: ShapeCastDesc): Result<ShapeCastHit \| null>` | The first collider a shape meets when swept along a direction; the same view of the colliders as castRay. |
| [`og.physics.castShapeAll`](physics/fn-castShapeAll.md) | function | `(world: World, desc?: ShapeCastAllDesc): Result<readonly ShapeCastHit[]>` | Every collider a shape meets when swept along a direction, nearest first (ties in entity order), at most `maxHits`; the same view of the colliders as castRay. |
| [`og.physics.overlap`](physics/fn-overlap.md) | function | `(world: World, desc?: OverlapDesc): Result<readonly Entity[]>` | Every collider overlapping a shape, sorted by entity id; the same view of the colliders as castRay. |
| [`og.physics.overlapPoint`](physics/fn-overlapPoint.md) | function | `(world: World, desc?: PointOverlapDesc): Result<readonly Entity[]>` | Every collider containing a point, sorted by entity id; the same view of the colliders as castRay. |
| [`og.physics.massProperties`](physics/fn-massProperties.md) | function | `(world: World, entity: Entity): Result<MassProperties \| null>` | The live simulation's computed mass, world centre of mass and local principal inertia for a RigidBody entity, or null if not a body. |
| [`og.physics.velocityAtPoint`](physics/fn-velocityAtPoint.md) | function | `(world: World, entity: Entity, point: Vector3): Result<Vector3 \| null>` | The live simulation's world-space linear velocity of a RigidBody entity at a world-space point (linear + angular × (point - com)), or null if not a body. |
| [`og.physics.sensorContents`](physics/fn-sensorContents.md) | function | `(world: World, sensor: Entity): Result<readonly Entity[]>` | Every entity inside a sensor now, each once, sorted by entity id: who stands on a pressure plate, what lies in a pickup zone, how many are in a capture area. |
| [`og.physics.applyForce`](physics/fn-applyForce.md) | function | `(world: World, entity: Entity, force: Vector3, point?: Vector3): Result<void>` | Push a dynamic body with a force (N, world axes) over the next physics step, at a world point or its centre of mass: call it every tick for a steady force (thrust, wind, a car's downforce or an anti-… |
| [`og.physics.applyImpulse`](physics/fn-applyImpulse.md) | function | `(world: World, entity: Entity, impulse: Vector3, point?: Vector3): Result<void>` | Give a dynamic body an impulse (N·s, world axes) at a world point or its centre of mass: a sudden change of momentum (a kick, a hit, a jump: its velocity changes by impulse / mass), applied by the ne… |
| [`og.physics.applyTorque`](physics/fn-applyTorque.md) | function | `(world: World, entity: Entity, torque: Vector3): Result<void>` | Turn a dynamic body with a torque (N·m about each world axis; 2D uses z) over the next physics step, as applyForce: call it every tick for a steady twist. |
| [`og.physics.applyTorqueImpulse`](physics/fn-applyTorqueImpulse.md) | function | `(world: World, entity: Entity, torque: Vector3): Result<void>` | Give a dynamic body an angular impulse (N·m·s about each world axis; 2D uses z): its spin changes at once, as applyImpulse. |
| [`og.physics.setMesh`](physics/fn-setMesh.md) | function | `(world: World, mesh: Geometry, desc?: MeshDesc): Result<void>` | Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. |
| [`WheelPose`](physics/type-WheelPose.md) | struct |  | A wheel of an og.Vehicle as the last step left it, for drawing. |
| [`RagdollBoneShape`](physics/type-RagdollBoneShape.md) | enum |  | The collider fitted along a ragdoll bone. |
| [`RagdollJointKind`](physics/type-RagdollJointKind.md) | enum |  | A ragdoll joint's angular degrees of freedom. |
| [`RagdollBoneDesc`](physics/type-RagdollBoneDesc.md) | desc struct |  | A rest-pose bone: a shaped collider along axis for length, with explicit mass and joint constraints. |
| [`RagdollRigDesc`](physics/type-RagdollRigDesc.md) | desc struct |  | The bones of a ragdoll, parents first, for og.physics.setRagdollRig; og.physics.ragdollRigFromSkeleton makes one from a model's skeleton. |
| [`BonePose`](physics/type-BonePose.md) | struct |  | A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight). |
| [`og.physics.ropePoints`](physics/fn-ropePoints.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Rope's points after the last step, from its entity to its end (empty before its first step). |
| [`og.physics.garmentPoints`](physics/fn-garmentPoints.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Garment's points after the last step, in the world: one per position of the garment's mesh (vertices at the same position, split along a seam, share one), in the order the mesh first uses them. |
| [`og.physics.clothPoints`](physics/fn-clothPoints.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Cloth's points after the last step, row by row (`columns` per row, rows downwards from the top edge). |
| [`SoftBodyPoints`](physics/type-SoftBodyPoints.md) | struct |  | An og.SoftBody's surface after the last step. |
| [`og.physics.softBodyPoints`](physics/fn-softBodyPoints.md) | function | `(world: World, entity: Entity): Result<SoftBodyPoints>` | An og.SoftBody's points and the triangles of its surface after the last step (none before its first step). |
| [`og.physics.fluidParticles`](physics/fn-fluidParticles.md) | function | `(world: World, entity: Entity): Result<readonly Vector2[]>` | An og.Fluid2D's particle positions after the last step, in world space. |
| [`og.physics.fluid3dParticles`](physics/fn-fluid3dParticles.md) | function | `(world: World, entity: Entity): Result<readonly Vector3[]>` | An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared). |
| [`og.physics.vehicleWheels`](physics/fn-vehicleWheels.md) | function | `(world: World, entity: Entity): Result<readonly WheelPose[]>` | An og.Vehicle's wheels after the last step (front left, front right, rear left, rear right; or its og.VehicleWheel children in entity order), for drawing wheel models. |
| [`og.physics.ragdollRigFromSkeleton`](physics/fn-ragdollRigFromSkeleton.md) | function | `(world: World, skeleton: SkeletonAsset, bones: readonly number[]): Result<RagdollRigDesc>` | Build a rest-pose rig from explicitly selected joints of the first skin, parents first. |
| [`og.physics.setRagdollRig`](physics/fn-setRagdollRig.md) | function | `(world: World, entity: Entity, desc?: RagdollRigDesc): Result<void>` | Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ra… |
| [`og.physics.ragdollPose`](physics/fn-ragdollPose.md) | function | `(world: World, entity: Entity): Result<readonly BonePose[]>` | An og.Ragdoll's bones after the last step, in rig order (empty before its first step). |
| [`og.physics.applyRagdollImpulse`](physics/fn-applyRagdollImpulse.md) | function | `(world: World, entity: Entity, boneIndex: number, impulse: Vector3, point: Vector3): Result<void>` | Apply a world-space impulse at a world-space point to one dynamic ragdoll bone. |
| [`PhysicsPhase`](physics/type-PhysicsPhase.md) | struct |  | One phase of the last physics step, for og.physics.stats. |
| [`PhysicsStats`](physics/type-PhysicsStats.md) | struct |  | What the last physics step cost, phase by phase, and where it ran: for finding where physics time goes. |
| [`og.physics.stats`](physics/fn-stats.md) | function | `(world: World): Result<PhysicsStats>` | What the last physics step cost: each phase's time and whether it ran on the lanes pool. |

## Column kinds

Each is a factory, `RigidBodyType(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-RigidBodyType"></a>`RigidBodyType` | `RigidBodyType` | `Uint32Array` | 1 | A RigidBodyType field: the variant's index. |
| <a id="column-ColliderShape"></a>`ColliderShape` | `ColliderShape` | `Uint32Array` | 1 | A ColliderShape field: the variant's index. |
| <a id="column-CombineRule"></a>`CombineRule` | `CombineRule` | `Uint32Array` | 1 | A CombineRule field: the variant's index. |
| <a id="column-ForceFieldKind"></a>`ForceFieldKind` | `ForceFieldKind` | `Uint32Array` | 1 | A ForceFieldKind field: the variant's index. |
| <a id="column-JointType"></a>`JointType` | `JointType` | `Uint32Array` | 1 | A JointType field: the variant's index. |
| <a id="column-JointMotor"></a>`JointMotor` | `JointMotor` | `Uint32Array` | 1 | A JointMotor field: the variant's index. |
| <a id="column-JointMotorModel"></a>`JointMotorModel` | `JointMotorModel` | `Uint32Array` | 1 | A JointMotorModel field: the variant's index. |
| <a id="column-VehicleDrive"></a>`VehicleDrive` | `VehicleDrive` | `Uint32Array` | 1 | A VehicleDrive field: the variant's index. |
| <a id="column-VehicleModel"></a>`VehicleModel` | `VehicleModel` | `Uint32Array` | 1 | A VehicleModel field: the variant's index. |
| <a id="column-VehicleWheelCast"></a>`VehicleWheelCast` | `VehicleWheelCast` | `Uint32Array` | 1 | A VehicleWheelCast variant index. |
| <a id="column-VehicleDifferential"></a>`VehicleDifferential` | `VehicleDifferential` | `Uint32Array` | 1 | A VehicleDifferential variant index. |
| <a id="column-Simulation"></a>`Simulation` | `Simulation` | `Uint32Array` | 1 | A Simulation field: the variant's index. |
| <a id="column-SoftBodyShape"></a>`SoftBodyShape` | `SoftBodyShape` | `Uint32Array` | 1 | A SoftBodyShape field: the variant's index. |

```ogt
column RigidBodyType {
  doc "A RigidBodyType field: the variant's index."
  array Uint32Array
  stride 1
  value RigidBodyType
}
```

<sub>Module `physics` line 27</sub>

```ogt
column ColliderShape {
  doc "A ColliderShape field: the variant's index."
  array Uint32Array
  stride 1
  value ColliderShape
}
```

<sub>Module `physics` line 119</sub>

```ogt
column CombineRule {
  doc "A CombineRule field: the variant's index."
  array Uint32Array
  stride 1
  value CombineRule
}
```

<sub>Module `physics` line 134</sub>

```ogt
column ForceFieldKind {
  doc "A ForceFieldKind field: the variant's index."
  array Uint32Array
  stride 1
  value ForceFieldKind
}
```

<sub>Module `physics` line 217</sub>

```ogt
column JointType {
  doc "A JointType field: the variant's index."
  array Uint32Array
  stride 1
  value JointType
}
```

<sub>Module `physics` line 405</sub>

```ogt
column JointMotor {
  doc "A JointMotor field: the variant's index."
  array Uint32Array
  stride 1
  value JointMotor
}
```

<sub>Module `physics` line 420</sub>

```ogt
column JointMotorModel {
  doc "A JointMotorModel field: the variant's index."
  array Uint32Array
  stride 1
  value JointMotorModel
}
```

<sub>Module `physics` line 433</sub>

```ogt
column VehicleDrive {
  doc "A VehicleDrive field: the variant's index."
  array Uint32Array
  stride 1
  value VehicleDrive
}
```

<sub>Module `physics` line 930</sub>

```ogt
column VehicleModel {
  doc "A VehicleModel field: the variant's index."
  array Uint32Array
  stride 1
  value VehicleModel
}
```

<sub>Module `physics` line 943</sub>

```ogt
column VehicleWheelCast {
  doc "A VehicleWheelCast variant index."
  array Uint32Array
  stride 1
  value VehicleWheelCast
}
```

<sub>Module `physics` line 1266</sub>

```ogt
column VehicleDifferential {
  doc "A VehicleDifferential variant index."
  array Uint32Array
  stride 1
  value VehicleDifferential
}
```

<sub>Module `physics` line 1278</sub>

```ogt
column Simulation {
  doc "A Simulation field: the variant's index."
  array Uint32Array
  stride 1
  value Simulation
}
```

<sub>Module `physics` line 1374</sub>

```ogt
column SoftBodyShape {
  doc "A SoftBodyShape field: the variant's index."
  array Uint32Array
  stride 1
  value SoftBodyShape
}
```

<sub>Module `physics` line 1606</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_RAYCAST_HITS"></a>`MAX_RAYCAST_HITS` | `number` | `256` | The most hits og.physics.castRayAll returns. |
| <a id="const-LOCK_TRANSLATION_X"></a>`LOCK_TRANSLATION_X` | `number` | `1` | RigidBody.lock bit: no translation along x. |
| <a id="const-LOCK_TRANSLATION_Y"></a>`LOCK_TRANSLATION_Y` | `number` | `2` | RigidBody.lock bit: no translation along y. |
| <a id="const-LOCK_TRANSLATION_Z"></a>`LOCK_TRANSLATION_Z` | `number` | `4` | RigidBody.lock bit: no translation along z. |
| <a id="const-LOCK_ROTATION_X"></a>`LOCK_ROTATION_X` | `number` | `8` | RigidBody.lock bit: no rotation about x. |
| <a id="const-LOCK_ROTATION_Y"></a>`LOCK_ROTATION_Y` | `number` | `16` | RigidBody.lock bit: no rotation about y. |
| <a id="const-LOCK_ROTATION_Z"></a>`LOCK_ROTATION_Z` | `number` | `32` | RigidBody.lock bit: no rotation about z (the 2D profile's only rotation). |
| <a id="const-PROFILE_2D"></a>`PROFILE_2D` | `number` | `2` | Physics.profile: rapier2d, bodies in the xy plane rotating about z. |
| <a id="const-PROFILE_3D"></a>`PROFILE_3D` | `number` | `3` | Physics.profile: rapier3d. |
| <a id="const-CHARACTER_JUMPED"></a>`CHARACTER_JUMPED` | `number` | `1` | `CharacterEvent.kind`: a jump from the ground (or in coyote time). |
| <a id="const-CHARACTER_LANDED"></a>`CHARACTER_LANDED` | `number` | `2` | `CharacterEvent.kind`: touched the ground after being in the air. |
| <a id="const-CHARACTER_WALL_JUMPED"></a>`CHARACTER_WALL_JUMPED` | `number` | `3` | `CharacterEvent.kind`: a jump off a wall. |
| <a id="const-CHARACTER_AIR_JUMPED"></a>`CHARACTER_AIR_JUMPED` | `number` | `4` | `CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps). |
| <a id="const-CHARACTER_LEFT_GROUND"></a>`CHARACTER_LEFT_GROUND` | `number` | `5` | `CharacterEvent.kind`: left the ground without jumping (ran off an edge). |
| <a id="const-CHARACTER_BUMPED"></a>`CHARACTER_BUMPED` | `number` | `6` | `CharacterEvent.kind`: hit a ceiling while rising. |
| <a id="const-CHARACTER_CRUSHED"></a>`CHARACTER_CRUSHED` | `number` | `7` | `CharacterEvent.kind`: opposing solid surfaces crushed the character. |
| <a id="const-PIN_TOP"></a>`PIN_TOP` | `number` | `1` | `Cloth.pins` bit: the whole top row is pinned. |
| <a id="const-PIN_TOP_CORNERS"></a>`PIN_TOP_CORNERS` | `number` | `2` | `Cloth.pins` bit: the top-left and top-right corners are pinned. |
| <a id="const-PIN_LEFT"></a>`PIN_LEFT` | `number` | `4` | `Cloth.pins` bit: the whole left column is pinned (a flag on a pole). |
| <a id="const-PIN_TOP_RINGS"></a>`PIN_TOP_RINGS` | `number` | `8` | `Cloth.pins` bit: every fourth point of the top row (curtain rings). |
| <a id="const-QUERY_DYNAMIC"></a>`QUERY_DYNAMIC` | `number` | `1` | QueryFilter.kindFlags bit: include dynamic bodies. |
| <a id="const-QUERY_FIXED"></a>`QUERY_FIXED` | `number` | `2` | QueryFilter.kindFlags bit: include fixed bodies and static colliders. |
| <a id="const-QUERY_KINEMATIC"></a>`QUERY_KINEMATIC` | `number` | `4` | QueryFilter.kindFlags bit: include kinematic bodies and controllers. |

```ogt
const MAX_RAYCAST_HITS u32 256 "The most hits og.physics.castRayAll returns."
```

<sub>Module `physics` line 9</sub>

```ogt
const LOCK_TRANSLATION_X u32 1 "RigidBody.lock bit: no translation along x."
```

<sub>Module `physics` line 10</sub>

```ogt
const LOCK_TRANSLATION_Y u32 2 "RigidBody.lock bit: no translation along y."
```

<sub>Module `physics` line 11</sub>

```ogt
const LOCK_TRANSLATION_Z u32 4 "RigidBody.lock bit: no translation along z."
```

<sub>Module `physics` line 12</sub>

```ogt
const LOCK_ROTATION_X u32 8 "RigidBody.lock bit: no rotation about x."
```

<sub>Module `physics` line 13</sub>

```ogt
const LOCK_ROTATION_Y u32 16 "RigidBody.lock bit: no rotation about y."
```

<sub>Module `physics` line 14</sub>

```ogt
const LOCK_ROTATION_Z u32 32 "RigidBody.lock bit: no rotation about z (the 2D profile's only rotation)."
```

<sub>Module `physics` line 15</sub>

```ogt
const PROFILE_2D u32 2 "Physics.profile: rapier2d, bodies in the xy plane rotating about z."
```

<sub>Module `physics` line 16</sub>

```ogt
const PROFILE_3D u32 3 "Physics.profile: rapier3d."
```

<sub>Module `physics` line 17</sub>

```ogt
const CHARACTER_JUMPED u32 1 "`CharacterEvent.kind`: a jump from the ground (or in coyote time)."
```

<sub>Module `physics` line 893</sub>

```ogt
const CHARACTER_LANDED u32 2 "`CharacterEvent.kind`: touched the ground after being in the air."
```

<sub>Module `physics` line 894</sub>

```ogt
const CHARACTER_WALL_JUMPED u32 3 "`CharacterEvent.kind`: a jump off a wall."
```

<sub>Module `physics` line 895</sub>

```ogt
const CHARACTER_AIR_JUMPED u32 4 "`CharacterEvent.kind`: a jump in the air (PlatformerController.airJumps)."
```

<sub>Module `physics` line 896</sub>

```ogt
const CHARACTER_LEFT_GROUND u32 5 "`CharacterEvent.kind`: left the ground without jumping (ran off an edge)."
```

<sub>Module `physics` line 897</sub>

```ogt
const CHARACTER_BUMPED u32 6 "`CharacterEvent.kind`: hit a ceiling while rising."
```

<sub>Module `physics` line 898</sub>

```ogt
const CHARACTER_CRUSHED u32 7 "`CharacterEvent.kind`: opposing solid surfaces crushed the character."
```

<sub>Module `physics` line 899</sub>

```ogt
const PIN_TOP u32 1 "`Cloth.pins` bit: the whole top row is pinned."
```

<sub>Module `physics` line 1381</sub>

```ogt
const PIN_TOP_CORNERS u32 2 "`Cloth.pins` bit: the top-left and top-right corners are pinned."
```

<sub>Module `physics` line 1382</sub>

```ogt
const PIN_LEFT u32 4 "`Cloth.pins` bit: the whole left column is pinned (a flag on a pole)."
```

<sub>Module `physics` line 1383</sub>

```ogt
const PIN_TOP_RINGS u32 8 "`Cloth.pins` bit: every fourth point of the top row (curtain rings)."
```

<sub>Module `physics` line 1384</sub>

```ogt
const QUERY_DYNAMIC u32 1 "QueryFilter.kindFlags bit: include dynamic bodies."
```

<sub>Module `physics` line 2202</sub>

```ogt
const QUERY_FIXED u32 2 "QueryFilter.kindFlags bit: include fixed bodies and static colliders."
```

<sub>Module `physics` line 2203</sub>

```ogt
const QUERY_KINEMATIC u32 4 "QueryFilter.kindFlags bit: include kinematic bodies and controllers."
```

<sub>Module `physics` line 2204</sub>
