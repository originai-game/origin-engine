<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SpawnDesc"></a>

### `SpawnDesc` (desc struct)

How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed. |
| `parent` | `Entity \| null` | `null` | — | — | — | The parent entity; the new entity's Transform is then relative to it. It gives the new entity an og.Parent, so inside a system og.Parent must be among the system's writes (as og.Transform must, unless `bare`). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The initial Transform.position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The initial Transform.quaternion. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The initial Transform.scale. |
| `bare` | `boolean` | `false` | — | — | — | Spawn without a Transform: a pure data entity. |

```ogt
struct SpawnDesc {
  doc "How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform."
  desc true
  field name String "A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed."
  field parent Option.Entity "The parent entity; the new entity's Transform is then relative to it. It gives the new entity an og.Parent, so inside a system og.Parent must be among the system's writes (as og.Transform must, unless `bare`)."
  field position Vector3 "The initial Transform.position." {
    unit m
  }
  field quaternion Quaternion "The initial Transform.quaternion."
  field scale Vector3 "The initial Transform.scale." {
    default 1 1 1
  }
  field bare bool "Spawn without a Transform: a pure data entity."
}
```

<sub>Module `world` line 17</sub>
