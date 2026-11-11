<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="type-NoiseDesc"></a>

### `NoiseDesc` (desc struct)

A noise agents may hear next tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it is. |
| `loudness` | `number` | `1` | 0 … 100 | — | — | How loud: heard within Perception.hearing times this. |
| `source` | `Entity` | `0` | — | — | — | Who makes it (0: nobody): that entity, and the agents it is under (og.Parent), do not hear it. |

```ogt
struct NoiseDesc {
  doc "A noise agents may hear next tick."
  desc true
  field position Vector3 "Where it is." {
    unit m
  }
  field loudness f32 "How loud: heard within Perception.hearing times this." {
    default 1
    min 0
    max 100
  }
  field source Entity "Who makes it (0: nobody): that entity, and the agents it is under (og.Parent), do not hear it."
}
```

<sub>Module `ai` line 166</sub>
