<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Collider"></a>

### `og.Collider` (component)

The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.

Compound colliders: a Collider without a RigidBody on a child entity whose og.Parent chain reaches a RigidBody is part of that body (the nearest such ancestor), placed at its transform relative to the body (the transforms between them composed, the body's scale applied): a table of a top and four legs, a ship of a hull and a mast, are one body. Its mass adds to the body's from its density, or the body's RigidBody.mass is shared among its colliders by theirs; its friction, restitution, layers and sensor flag are its own. Children join in entity order, whatever the query order. Changing a child (its collider, or its transform relative to the body) rebuilds the body, which loses its contacts' warm starts. Events and queries name the collider's entity; RaycastHit.body and PointProjection.body name the body. A controller's body takes no children; a collider whose chain reaches no body stays static at its world transform.

Also known as: trigger, sensor, hitbox, collision shape, Unity Collider isTrigger, Godot CollisionShape

Token `og.Collider`, registered under the schema name `og.Collider`; its value type is `Collider`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `number` | `0` | 0 … 8 | — | label “Shape” | One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (its outline in 2D), 6 the height field `mesh` (3D: a terrain), 7 a diamond and 8 a hexagon in `size` (both). |
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
  doc "Compound colliders: a Collider without a RigidBody on a child entity whose og.Parent chain reaches a RigidBody is part of that body (the nearest such ancestor), placed at its transform relative to the body (the transforms between them composed, the body's scale applied): a table of a top and four legs, a ship of a hull and a mast, are one body. Its mass adds to the body's from its density, or the body's RigidBody.mass is shared among its colliders by theirs; its friction, restitution, layers and sensor flag are its own. Children join in entity order, whatever the query order. Changing a child (its collider, or its transform relative to the body) rebuilds the body, which loses its contacts' warm starts. Events and queries name the collider's entity; RaycastHit.body and PointProjection.body name the body. A controller's body takes no children; a collider whose chain reaches no body stays static at its world transform."
  aliases "trigger, sensor, hitbox, collision shape, Unity Collider isTrigger, Godot CollisionShape"
  field shape u32 "One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (its outline in 2D), 6 the height field `mesh` (3D: a terrain), 7 a diamond and 8 a hexagon in `size` (both)." {
    min 0
    max 8
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

<sub>`api.ogt` line 8591</sub>
