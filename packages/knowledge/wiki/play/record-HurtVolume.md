<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`play`](../play.md) · [index](../index.md)

<a id="record-HurtVolume"></a>

### `og.HurtVolume` (component)

What can be struck: a shape at its entity's simulation pose, as og.HitVolume's (a bone through og.BoneAttachment). An og.HitVolume of another team and owner strikes its owner once a swing, at the first of the owner's hurt volumes it meets. Sizes are scaled by the entity's world scale.

Also known as: hurtbox, damage box, body hitbox, weak point, Godot Hurtbox

Token `og.HurtVolume`, registered under the schema name `og.HurtVolume`; its value type is `HurtVolume`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `VolumeShape` | `"sphere"` | — | — | — | Its shape. |
| `size` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | ≥ 0 | `m` | tunable | A box's full extents. |
| `radius` | `number` | `0.5` | ≥ 0 | `m` | tunable | A sphere's or capsule's radius. |
| `height` | `number` | `1` | ≥ 0 | `m` | tunable | A capsule's distance between its cap centres. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The shape's centre in its entity's frame. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The shape's rotation in its entity's frame; a zero quaternion is no rotation. |
| `team` | `number` | `0` | — | — | — | Its team: an og.HitVolume of the same team never strikes it. |
| `owner` | `Entity` | `0` | — | — | — | Who is struck (og.HitEvent.target): a body's hurt volumes share their owner, so a swing strikes it once; 0 is its own entity. |

```ogt
component HurtVolume {
  aliases "hurtbox, damage box, body hitbox, weak point, Godot Hurtbox"
  doc "What can be struck: a shape at its entity's simulation pose, as og.HitVolume's (a bone through og.BoneAttachment). An og.HitVolume of another team and owner strikes its owner once a swing, at the first of the owner's hurt volumes it meets. Sizes are scaled by the entity's world scale."
  field shape VolumeShape "Its shape."
  field size Vector3 "A box's full extents." {
    default 1 1 1
    min 0
    unit m
    tunable true
  }
  field radius f32 "A sphere's or capsule's radius." {
    default 0.5
    min 0
    unit m
    tunable true
  }
  field height f32 "A capsule's distance between its cap centres." {
    default 1
    min 0
    unit m
    tunable true
  }
  field offset Vector3 "The shape's centre in its entity's frame." {
    unit m
  }
  field quaternion Quaternion "The shape's rotation in its entity's frame; a zero quaternion is no rotation."
  field team u32 "Its team: an og.HitVolume of the same team never strikes it."
  field owner Entity "Who is struck (og.HitEvent.target): a body's hurt volumes share their owner, so a swing strikes it once; 0 is its own entity."
}
```

<sub>Module `play` line 62</sub>
