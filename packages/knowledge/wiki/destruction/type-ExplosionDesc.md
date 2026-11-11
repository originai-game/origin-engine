<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-ExplosionDesc"></a>

### `ExplosionDesc` (desc struct)

An explosion for og.destruction.explode.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its centre, in world space. |
| `radius` | `number` | `3` | ≥ 0 | `m` | — | How far it reaches. |
| `innerRadius` | `number` | `0.5` | ≥ 0 | `m` | — | Within this everything takes the whole damage and push. |
| `damage` | `number` | `3000` | ≥ 0 | `newtonSecond` | — | The damage to every destructible's bonds in reach, as og.destruction.damage's amount. |
| `impulse` | `number` | `2000` | ≥ 0 | `newtonSecondPerM2` | — | The push outwards on every loose piece in reach, per square metre of the face it shows the blast (the two-thirds power of its volume), at its centre, falling off as the damage does. The pieces it drives apart pull on the bonds between them. See `physics §9`. |

```ogt
struct ExplosionDesc {
  doc "An explosion for og.destruction.explode."
  desc true
  field point Vector3 "Its centre, in world space." {
    unit m
  }
  field radius f32 "How far it reaches." {
    default 3
    min 0
    unit m
  }
  field innerRadius f32 "Within this everything takes the whole damage and push." {
    default 0.5
    min 0
    unit m
  }
  field damage f32 "The damage to every destructible's bonds in reach, as og.destruction.damage's amount." {
    default 3000
    min 0
    unit newtonSecond
  }
  field impulse f32 "The push outwards on every loose piece in reach, per square metre of the face it shows the blast (the two-thirds power of its volume), at its centre, falling off as the damage does. The pieces it drives apart pull on the bonds between them." {
    guide "physics §9"
    default 2000
    min 0
    unit newtonSecondPerM2
  }
}
```

<sub>Module `destruction` line 192</sub>
