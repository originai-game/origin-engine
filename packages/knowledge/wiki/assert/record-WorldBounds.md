<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="record-WorldBounds"></a>

### `og.WorldBounds` (resource)

Where the world's entities may be: naming it turns on the default invariant that every entity's place (its og.MatrixWorld translation, else a root's og.Transform.position) lies inside min..max on every tick. A world that never names it has no bounds and keeps its hash.

Also known as: kill zone, kill plane, Unreal KillZ, out of bounds, fell out of the world, level limits

Token `og.WorldBounds`, registered under the schema name `og.WorldBounds`; its value type is `WorldBounds`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector3` | `{ x: -1000, y: -1000, z: -1000 }` | — | `m` | worldspace `world` | The lowest corner. |
| `max` | `Vector3` | `{ x: 1000, y: 1000, z: 1000 }` | — | `m` | worldspace `world` | The highest corner. |
| `off` | `boolean` | `false` | — | — | — | Leave the bounds unchecked. |

```ogt
resource WorldBounds {
  doc "Where the world's entities may be: naming it turns on the default invariant that every entity's place (its og.MatrixWorld translation, else a root's og.Transform.position) lies inside min..max on every tick. A world that never names it has no bounds and keeps its hash."
  aliases "kill zone, kill plane, Unreal KillZ, out of bounds, fell out of the world, level limits"
  field min Vector3 "The lowest corner." {
    default -1000 -1000 -1000
    worldspace world
    unit m
  }
  field max Vector3 "The highest corner." {
    default 1000 1000 1000
    worldspace world
    unit m
  }
  field off bool "Leave the bounds unchecked."
}
```

<sub>Module `assert` line 116</sub>
