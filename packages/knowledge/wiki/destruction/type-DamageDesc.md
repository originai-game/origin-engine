<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-DamageDesc"></a>

### `DamageDesc` (desc struct)

A radial damage for og.destruction.damage.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it lands, in world space. |
| `radius` | `number` | `0.4` | ≥ 0 | `m` | — | How far it reaches; it falls linearly to nothing here from innerRadius. |
| `innerRadius` | `number` | `0.1` | ≥ 0 | `m` | — | Within this the bonds take the whole amount. |
| `amount` | `number` | `500` | ≥ 0 | `newtonSecond` | — | The damage, as an impulse through a bond (Destructible.toughness says how much a square metre takes). |
| `impulse` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `newtonSecond` | — | The momentum the blow carries, along its direction (a hammer, a wrecking ball, a shot): the pieces it breaks loose share it by its falloff and fly on, as a hit's do. Zero, the default, crushes without pushing, and what it breaks out of a standing prop can stay standing on what holds it. |

```ogt
struct DamageDesc {
  doc "A radial damage for og.destruction.damage."
  desc true
  field point Vector3 "Where it lands, in world space." {
    unit m
  }
  field radius f32 "How far it reaches; it falls linearly to nothing here from innerRadius." {
    default 0.4
    min 0
    unit m
  }
  field innerRadius f32 "Within this the bonds take the whole amount." {
    default 0.1
    min 0
    unit m
  }
  field amount f32 "The damage, as an impulse through a bond (Destructible.toughness says how much a square metre takes)." {
    default 500
    min 0
    unit newtonSecond
  }
  field impulse Vector3 "The momentum the blow carries, along its direction (a hammer, a wrecking ball, a shot): the pieces it breaks loose share it by its falloff and fly on, as a hit's do. Zero, the default, crushes without pushing, and what it breaks out of a standing prop can stay standing on what holds it." {
    unit newtonSecond
  }
}
```

<sub>`api.ogt` line 27239</sub>
