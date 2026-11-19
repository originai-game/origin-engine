<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="record-ScatterClearing"></a>

### `og.ScatterClearing` (component)

Grows nothing inside a box: no og.TerrainScatter copy whose root stands in it and no og.Grass blade (a house's floor, a paved yard, a slab or a road laid over a terrain). The box is centred on the entity and follows its whole world transform, turned and scaled with it, so a slab's own entity can carry it. A copy is kept out by its root on the ground, so the box reaches below the ground and above the floor. Moving one places the copies again. Visual only. Grass skips the 16 clearings nearest the camera; scatters read every one, placed on the CPU past 16.

Also known as: no-grow zone, foliage blocker, clear the grass, keep plants off a floor, Unreal procedural foliage blocking volume, Unity detail exclusion, Unity tree exclusion

Token `og.ScatterClearing`, registered under the schema name `og.ScatterClearing`; its value type is `ScatterClearing`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | ≥ 0 | `m` | — | The box's full extents along the entity's own x, y and z, before its world scale. 0 on any axis: no clearing. |

```ogt
component ScatterClearing {
  aliases "no-grow zone, foliage blocker, clear the grass, keep plants off a floor, Unreal procedural foliage blocking volume, Unity detail exclusion, Unity tree exclusion"
  doc "Grows nothing inside a box: no og.TerrainScatter copy whose root stands in it and no og.Grass blade (a house's floor, a paved yard, a slab or a road laid over a terrain). The box is centred on the entity and follows its whole world transform, turned and scaled with it, so a slab's own entity can carry it. A copy is kept out by its root on the ground, so the box reaches below the ground and above the floor. Moving one places the copies again. Visual only. Grass skips the 16 clearings nearest the camera; scatters read every one, placed on the CPU past 16."
  field size Vector3 "The box's full extents along the entity's own x, y and z, before its world scale. 0 on any axis: no clearing." {
    default 1 1 1
    min 0
    unit m
  }
}
```

<sub>Module `terrain` line 419</sub>
