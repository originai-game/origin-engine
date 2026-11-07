<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DamageHitDesc"></a>

### `DamageHitDesc` (desc struct)

A damage number to throw.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the hit landed (a world point). |
| `amount` | `number` | `0` | — | — | — | How much. |
| `kind` | `DamageKind` | `"normal"` | — | — | — | What it is. |
| `word` | `string` | `""` | — | — | — | A miss's word (Miss, Dodge, Block, Immune); empty is Miss. |
| `color` | `Paint \| null` | `null` | — | — | — | A damage type's colour (fire, frost); null is the kind's. |
| `seed` | `number` | `0` | — | — | — | Which way it drifts (a hit's index). |

```ogt
struct DamageHitDesc {
  doc "A damage number to throw."
  desc true
  field position Vector3 "Where the hit landed (a world point)."
  field amount f64 "How much."
  field kind DamageKind "What it is."
  field word String "A miss's word (Miss, Dodge, Block, Immune); empty is Miss."
  field color Option.Paint "A damage type's colour (fire, frost); null is the kind's."
  field seed u32 "Which way it drifts (a hit's index)."
}
```

<sub>Module `ui` line 6521</sub>
