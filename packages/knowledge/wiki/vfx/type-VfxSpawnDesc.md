<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`vfx`](../vfx.md) · [index](../index.md)

<a id="type-VfxSpawnDesc"></a>

### `VfxSpawnDesc` (desc struct)

Where and how og.vfx.spawn plays an effect.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `VfxAsset` | `0` | — | — | — | The effect (og.load.vfx). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where: in the world, or in the parent's space with a parent. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its turn, as position. |
| `parent` | `Entity \| null` | `null` | — | — | — | An entity to follow (a sword's tip, a character's chest): the effect's entity becomes its child, so it moves with it and despawns with it. |
| `scale` | `number` | `1` | ≥ 0 | — | — | Its size (og.Vfx.scale). |
| `tint` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | Its tint (og.Vfx.tint). |
| `seed` | `number` | `0` | — | — | — | Its seed (og.Vfx.seed); 0 takes one from the new entity's id, so each instance differs and a replay repeats it. |

```ogt
struct VfxSpawnDesc {
  doc "Where and how og.vfx.spawn plays an effect."
  desc true
  field asset VfxAsset "The effect (og.load.vfx)."
  field position Vector3 "Where: in the world, or in the parent's space with a parent." {
    unit m
  }
  field rotation Quaternion "Its turn, as position."
  field parent Option.Entity "An entity to follow (a sword's tip, a character's chest): the effect's entity becomes its child, so it moves with it and despawns with it."
  field scale f32 "Its size (og.Vfx.scale)." {
    default 1
    min 0
  }
  field tint Color "Its tint (og.Vfx.tint)." {
    default 1 1 1
  }
  field seed u32 "Its seed (og.Vfx.seed); 0 takes one from the new entity's id, so each instance differs and a replay repeats it."
}
```

<sub>`api.ogt` line 27448</sub>
