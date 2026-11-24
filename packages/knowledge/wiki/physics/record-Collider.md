<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Collider"></a>

### `og.Collider` (component)

The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.

A Collider without a RigidBody whose og.Parent chain reaches one is part of the nearest such body (a compound), placed relative to it; its density adds to the body's mass, and its friction, restitution, layers and sensor flag are its own. Events and queries name the collider's entity; RaycastHit.body and PointProjection.body name the body. A controller's body takes no children.

See `physics §1`.

Also known as: trigger, sensor, hitbox, collision shape, Unity Collider isTrigger, Godot CollisionShape

Token `og.Collider`, registered under the schema name `og.Collider`; its value type is `Collider`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `ColliderShape` | `"box"` | — | — | label “Shape” | What it is shaped as: box, sphere (a circle in 2D), capsule, cylinder (3D), convex (the hull of `mesh`; a polygon in 2D), trimesh (the triangles of `mesh`; its outline in 2D), heightfield (`mesh` is a height field), diamond, hexagon, cone (3D) or polyline. |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | ≥ 0 | `m` | tunable | Full extents of a box, as three.js BoxGeometry(width, height, depth). |
| `radius` | `number` | `0.5` | ≥ 0 | `m` | tunable | Radius of a sphere, circle, capsule, cylinder or cone. |
| `height` | `number` | `1` | ≥ 0 | `m` | tunable | A capsule's distance between its cap centres (three.js CapsuleGeometry length); a cylinder's or cone's height. |
| `mesh` | `Geometry` | `0` | — | — | — | The geometry whose vertices a convex, triangle-mesh or polyline shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh; for a heightfield shape, a terrain's height field (og.terrain.load's TerrainInfo.geometry). |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Position of the shape relative to the entity. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Rotation of the shape relative to the entity. |
| `friction` | `number` | `0.5` | 0 … 2 | — | tunable | Coulomb friction (0 is ice, 1 rubber); combined according to `frictionCombine`. |
| `restitution` | `number` | `0` | 0 … 1 | — | tunable | Bounciness (0 no bounce, 1 elastic); combined according to `restitutionCombine`. |
| `density` | `number` | `1` | ≥ 0 | `kgPerM3` | tunable | Density, used when RigidBody.mass is 0. |
| `sensor` | `boolean` | `false` | — | — | — | A sensor reports SensorEntered and SensorExited and pushes nothing; false is a solid. |
| `layers` | `number` | `1` | — | — | — | The collision layers this collider is on, as bits. |
| `mask` | `number` | `4294967295` | — | — | — | The layers it collides with, as bits. Two colliders interact when each one's layers meet the other's mask. |
| `roundRadius` | `number` | `0` | ≥ 0 | `m` | tunable | Rounding radius added around polygonal shapes (box, cylinder, cone, convex, diamond, hexagon); 0 is sharp. |
| `frictionCombine` | `CombineRule` | `"average"` | — | — | — | How friction combines with another collider: average, min, multiply or max. |
| `restitutionCombine` | `CombineRule` | `"average"` | — | — | — | How restitution combines with another collider: average, min, multiply or max. |
| `solverLayers` | `number` | `0` | — | — | — | Solver layers (bits) for contact force resolution; 0 inherits `layers`. |
| `solverMask` | `number` | `0` | — | — | — | Solver mask (bits) for contact force resolution; 0 inherits `mask`. |
| `enabled` | `boolean` | `true` | — | — | — | Whether this collider is active in collision detection and queries. |

```ogt
component Collider {
  doc "The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale."
  doc "A Collider without a RigidBody whose og.Parent chain reaches one is part of the nearest such body (a compound), placed relative to it; its density adds to the body's mass, and its friction, restitution, layers and sensor flag are its own. Events and queries name the collider's entity; RaycastHit.body and PointProjection.body name the body. A controller's body takes no children."
  guide "physics §1"
  aliases "trigger, sensor, hitbox, collision shape, Unity Collider isTrigger, Godot CollisionShape"
  field shape ColliderShape "What it is shaped as: box, sphere (a circle in 2D), capsule, cylinder (3D), convex (the hull of `mesh`; a polygon in 2D), trimesh (the triangles of `mesh`; its outline in 2D), heightfield (`mesh` is a height field), diamond, hexagon, cone (3D) or polyline." {
    label "Shape"
  }
  field size Vector3 "Full extents of a box, as three.js BoxGeometry(width, height, depth)." {
    default 1 1 1
    min 0
    unit m
    tunable true
  }
  field radius f32 "Radius of a sphere, circle, capsule, cylinder or cone." {
    default 0.5
    min 0
    unit m
    tunable true
  }
  field height f32 "A capsule's distance between its cap centres (three.js CapsuleGeometry length); a cylinder's or cone's height." {
    default 1
    min 0
    unit m
    tunable true
  }
  field mesh Geometry "The geometry whose vertices a convex, triangle-mesh or polyline shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh; for a heightfield shape, a terrain's height field (og.terrain.load's TerrainInfo.geometry)."
  field offset Vector3 "Position of the shape relative to the entity." {
    unit m
  }
  field quaternion Quaternion "Rotation of the shape relative to the entity."
  field friction f32 "Coulomb friction (0 is ice, 1 rubber); combined according to `frictionCombine`." {
    default 0.5
    min 0
    max 2
    tunable true
  }
  field restitution f32 "Bounciness (0 no bounce, 1 elastic); combined according to `restitutionCombine`." {
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
  field sensor bool "A sensor reports SensorEntered and SensorExited and pushes nothing; false is a solid."
  field layers u32 "The collision layers this collider is on, as bits." {
    default 1
  }
  field mask u32 "The layers it collides with, as bits. Two colliders interact when each one's layers meet the other's mask." {
    default 4294967295
  }
  field roundRadius f32 "Rounding radius added around polygonal shapes (box, cylinder, cone, convex, diamond, hexagon); 0 is sharp." {
    min 0
    unit m
    tunable true
  }
  field frictionCombine CombineRule "How friction combines with another collider: average, min, multiply or max."
  field restitutionCombine CombineRule "How restitution combines with another collider: average, min, multiply or max."
  field solverLayers u32 "Solver layers (bits) for contact force resolution; 0 inherits `layers`."
  field solverMask u32 "Solver mask (bits) for contact force resolution; 0 inherits `mask`."
  field enabled bool "Whether this collider is active in collision detection and queries." {
    default true
  }
}
```

<sub>Module `physics` line 141</sub>
