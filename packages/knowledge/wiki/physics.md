<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `physics` module

Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.

The profile is the project's (`profile "2d"` or `"3d"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.

The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.

- **TypeScript**: functions are the namespace `og.physics`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module physics {
  doc "Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back."
  doc "The profile is the project's (`profile \"2d\"` or `\"3d\"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform."
  doc "The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 2902</sub>

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

<sub>`api.ogt` line 3207</sub>

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

<sub>`api.ogt` line 3214</sub>

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

<sub>`api.ogt` line 3221</sub>

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

<sub>`api.ogt` line 3228</sub>

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

<sub>`api.ogt` line 2925</sub>

<a id="record-Collider"></a>

### `og.Collider` (component)

The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.

Token `og.Collider`, registered under the schema name `og.Collider`; its value type is `Collider`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `number` | `0` | 0 … 5 | — | label “Shape” | One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (3D). |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | ≥ 0 | `m` | tunable | Full extents of a box, as three.js BoxGeometry(width, height, depth). |
| `radius` | `number` | `0.5` | ≥ 0 | `m` | tunable | Radius of a sphere, circle, capsule or cylinder. |
| `height` | `number` | `1` | ≥ 0 | `m` | tunable | A capsule's distance between its cap centres (three.js CapsuleGeometry length); a cylinder's height. |
| `mesh` | `Geometry` | `0` | — | — | — | The geometry whose vertices a convex or triangle-mesh shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh. |
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
  field shape u32 "One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (3D)." {
    min 0
    max 5
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
  field mesh Geometry "The geometry whose vertices a convex or triangle-mesh shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh."
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

<sub>`api.ogt` line 2966</sub>

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

<sub>`api.ogt` line 3025</sub>

<a id="record-Physics"></a>

### `og.Physics` (resource)

The physics settings of the world. The profile is the project's; the same components serve both profiles.

Token `og.Physics`, registered under the schema name `og.Physics`; its value type is `Physics`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `profile` | `number` | `3` | 2 … 3 | — | read only, label “Profile” | PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt. |
| `gravity` | `Vector3` | `{ x: 0, y: -9.81, z: 0 }` | — | `mps2` | tunable | The acceleration of every dynamic body, times its gravityScale; 2D uses x and y. |
| `substeps` | `number` | `1` | 1 … 16 | — | tunable | Physics steps per tick, each of dt / substeps: more is stiffer stacks and less tunnelling, at more cost. |
| `solverIterations` | `number` | `4` | 1 … 32 | — | tunable | Solver iterations per step. |
| `restSpeed` | `number` | `0.02` | 0 … 1 | `mps` | tunable | A dynamic body touching something, supported with at most 2 mm of overlap and slower than this before and after a step, keeps its pose and loses its residual velocity (the same number in m/s and rad/s); 0 disables it. |

```ogt
resource Physics {
  doc "The physics settings of the world. The profile is the project's; the same components serve both profiles."
  field profile u32 "PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt." {
    default 3
    min 2
    max 3
    label "Profile"
    readonly true
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

<sub>`api.ogt` line 3037</sub>

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

<sub>`api.ogt` line 3072</sub>

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

<sub>`api.ogt` line 3080</sub>

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

<sub>`api.ogt` line 3086</sub>

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

<sub>`api.ogt` line 3092</sub>

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

<sub>`api.ogt` line 3098</sub>

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

<sub>`api.ogt` line 3105</sub>

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

<sub>`api.ogt` line 3113</sub>

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

<sub>`api.ogt` line 3129</sub>

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

<sub>`api.ogt` line 3139</sub>

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

<sub>`api.ogt` line 3161</sub>

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

<sub>`api.ogt` line 3179</sub>

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

<sub>`api.ogt` line 3189</sub>

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

<sub>`api.ogt` line 3200</sub>

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
| <a id="const-PROFILE_2D"></a>`PROFILE_2D` | `number` | `2` | Physics.profile: rapier2d, bodies in the xy plane rotating about z. |
| <a id="const-PROFILE_3D"></a>`PROFILE_3D` | `number` | `3` | Physics.profile: rapier3d. |

```ogt
const BODY_DYNAMIC u32 0 "RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick."
```

<sub>`api.ogt` line 2907</sub>

```ogt
const BODY_STATIC u32 1 "RigidBody.kind: never moves; collides with dynamic bodies."
```

<sub>`api.ogt` line 2908</sub>

```ogt
const BODY_KINEMATIC u32 2 "RigidBody.kind: moved by game code through its Transform; its Velocity, if any, is how fast it pushes."
```

<sub>`api.ogt` line 2909</sub>

```ogt
const LOCK_TRANSLATION_X u32 1 "RigidBody.lock bit: no translation along x."
```

<sub>`api.ogt` line 2910</sub>

```ogt
const LOCK_TRANSLATION_Y u32 2 "RigidBody.lock bit: no translation along y."
```

<sub>`api.ogt` line 2911</sub>

```ogt
const LOCK_TRANSLATION_Z u32 4 "RigidBody.lock bit: no translation along z."
```

<sub>`api.ogt` line 2912</sub>

```ogt
const LOCK_ROTATION_X u32 8 "RigidBody.lock bit: no rotation about x."
```

<sub>`api.ogt` line 2913</sub>

```ogt
const LOCK_ROTATION_Y u32 16 "RigidBody.lock bit: no rotation about y."
```

<sub>`api.ogt` line 2914</sub>

```ogt
const LOCK_ROTATION_Z u32 32 "RigidBody.lock bit: no rotation about z (the 2D profile's only rotation)."
```

<sub>`api.ogt` line 2915</sub>

```ogt
const SHAPE_BOX u32 0 "Collider.shape: a box of `size` (both profiles)."
```

<sub>`api.ogt` line 2916</sub>

```ogt
const SHAPE_SPHERE u32 1 "Collider.shape: a sphere of `radius` (a circle in 2D)."
```

<sub>`api.ogt` line 2917</sub>

```ogt
const SHAPE_CAPSULE u32 2 "Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles)."
```

<sub>`api.ogt` line 2918</sub>

```ogt
const SHAPE_CYLINDER u32 3 "Collider.shape: a cylinder along y of `radius` and `height` (3D only)."
```

<sub>`api.ogt` line 2919</sub>

```ogt
const SHAPE_CONVEX u32 4 "Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D)."
```

<sub>`api.ogt` line 2920</sub>

```ogt
const SHAPE_TRIMESH u32 5 "Collider.shape: the triangles of `mesh` (3D only; for static and kinematic bodies)."
```

<sub>`api.ogt` line 2921</sub>

```ogt
const PROFILE_2D u32 2 "Physics.profile: rapier2d, bodies in the xy plane rotating about z."
```

<sub>`api.ogt` line 2922</sub>

```ogt
const PROFILE_3D u32 3 "Physics.profile: rapier3d."
```

<sub>`api.ogt` line 2923</sub>
