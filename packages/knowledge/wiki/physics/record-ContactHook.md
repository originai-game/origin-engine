<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-ContactHook"></a>

### `og.ContactHook` (component)

What a collider's contacts do beyond its og.Collider: a one-way platform, a conveyor belt, friction and restitution of its own, entities it passes through. Put it on the collider's entity (a static collider, a body, or a compound child). The solver reads a table of the hooks frozen at the start of the step, on its worker threads: hooks are data, not game callbacks, and run the same on both hosts and in every replay.

Token `og.ContactHook`, registered under the schema name `og.ContactHook`; its value type is `ContactHook`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `oneWay` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | A one-way surface: solid only to what is on this side of it, within 45° (world space; up for a platform jumped through from below). What came in from elsewhere passes until it is clear of it. Zero is off. |
| `surfaceVelocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | tunable | The surface's own velocity along its contacts (world space): what lies on it is carried, as on a conveyor belt, a treadmill or a moving walkway. Zero is off. |
| `friction` | `number` | `-1` | -1 … 2 | — | tunable | Its contacts' friction, in place of the pair's mean; negative keeps the colliders' own. |
| `restitution` | `number` | `-1` | -1 … 1 | — | tunable | Its contacts' restitution, in place of the pair's mean; negative keeps the colliders' own. |
| `ignore` | `readonly Entity[]` | `[]` | — | — | a list of at most 8 `Entity` (`List.Entity.8`): reads a frozen copy, writes a whole array | Entities it never collides with: colliders or their bodies (a projectile and its shooter, a ghost and a door); 0s are skipped. |

```ogt
component ContactHook {
  doc "What a collider's contacts do beyond its og.Collider: a one-way platform, a conveyor belt, friction and restitution of its own, entities it passes through. Put it on the collider's entity (a static collider, a body, or a compound child). The solver reads a table of the hooks frozen at the start of the step, on its worker threads: hooks are data, not game callbacks, and run the same on both hosts and in every replay."
  field oneWay Vector3 "A one-way surface: solid only to what is on this side of it, within 45° (world space; up for a platform jumped through from below). What came in from elsewhere passes until it is clear of it. Zero is off."
  field surfaceVelocity Vector3 "The surface's own velocity along its contacts (world space): what lies on it is carried, as on a conveyor belt, a treadmill or a moving walkway. Zero is off." {
    unit mps
    tunable true
  }
  field friction f32 "Its contacts' friction, in place of the pair's mean; negative keeps the colliders' own." {
    default -1
    min -1
    max 2
    tunable true
  }
  field restitution f32 "Its contacts' restitution, in place of the pair's mean; negative keeps the colliders' own." {
    default -1
    min -1
    max 1
    tunable true
  }
  field ignore List.Entity.8 "Entities it never collides with: colliders or their bodies (a projectile and its shooter, a ghost and a door); 0s are skipped."
}
```

<sub>Module `physics` line 267</sub>
