<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="record-GrassType"></a>

### `og.GrassType` (component)

Another kind of blade for og.Grass fields that list this entity in their `types`: its shape and colours replace the field's for the clumps that grow it, the rest (density, wind, lighting, dryness) is the field's.

Token `og.GrassType`, registered under the schema name `og.GrassType`; its value type is `GrassType`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `height` | `number` | `0.45` | 0 … 4 | `m` | tunable | The tallest blade of this kind. |
| `width` | `number` | `0.012` | 0 … 0.5 | `m` | tunable | A blade's width at its root. |
| `curve` | `number` | `0.5` | 0 … 1 | `ratio` | tunable | How far its blades droop along their length: 0 straight, 1 arching over. |
| `baseColor` | `Color` | `{ r: 0.04, g: 0.085, b: 0.024 }` | — | — | — | The roots' colour (linear). |
| `tipColor` | `Color` | `{ r: 0.235, g: 0.39, b: 0.11 }` | — | — | — | The tips' colour (linear): a flower's petals. |
| `share` | `number` | `1` | 0 … 100 | — | tunable | How many clumps grow this kind for each one of the field's own, where no typeMap says. |

```ogt
component GrassType {
  doc "Another kind of blade for og.Grass fields that list this entity in their `types`: its shape and colours replace the field's for the clumps that grow it, the rest (density, wind, lighting, dryness) is the field's."
  field height f32 "The tallest blade of this kind." {
    default 0.45
    min 0
    max 4
    unit m
    tunable true
  }
  field width f32 "A blade's width at its root." {
    default 0.012
    min 0
    max 0.5
    unit m
    tunable true
  }
  field curve f32 "How far its blades droop along their length: 0 straight, 1 arching over." {
    default 0.5
    min 0
    max 1
    unit ratio
    tunable true
  }
  field baseColor Color "The roots' colour (linear)." {
    default 0.04 0.085 0.024
  }
  field tipColor Color "The tips' colour (linear): a flower's petals." {
    default 0.235 0.39 0.11
  }
  field share f32 "How many clumps grow this kind for each one of the field's own, where no typeMap says." {
    default 1
    min 0
    max 100
    tunable true
  }
}
```

<sub>`api.ogt` line 25089</sub>
