<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeTunable"></a>

### `ProbeTunable` (struct)

One published tunable parameter and the range it means something over.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Where the value lives, as `Health.hp`. |
| `label` | `string` | `""` | — | — | optional | The person-facing name. |
| `min` | `number` | `0` | — | — | optional | The smallest meaningful value. |
| `max` | `number` | `0` | — | — | optional | The largest meaningful value. |
| `step` | `number` | `0` | — | — | optional | Tune's increment. |

```ogt
struct ProbeTunable {
  doc "One published tunable parameter and the range it means something over."
  field path String "Where the value lives, as `Health.hp`."
  field label String "The person-facing name." {
    optional true
  }
  field min f32 "The smallest meaningful value." {
    optional true
  }
  field max f32 "The largest meaningful value." {
    optional true
  }
  field step f32 "Tune's increment." {
    optional true
  }
}
```

<sub>`api.ogt` line 24045</sub>
