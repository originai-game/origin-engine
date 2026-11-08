<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PlayDesc"></a>

### `PlayDesc` (desc struct)

How a play runs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reverse` | `boolean` | `false` | — | — | — | Plays backwards. |
| `repeat` | `Repeat` | `"once"` | — | — | — | How it repeats. |
| `speed` | `number` | `0` | — | — | — | Its rate (0: 1). |
| `fromMs` | `number` | `0` | — | `ms` | — | Start this far in. |
| `fromLabel` | `string` | `""` | — | — | — | Start at a label of its first timeline (wins over fromMs). |
| `hold` | `boolean` | `false` | — | — | — | Keep the last values once it ends, until stopped. |
| `decorative` | `boolean` | `false` | — | — | — | Under reduced motion it does not play at all. |

```ogt
struct PlayDesc {
  doc "How a play runs."
  desc true
  field reverse bool "Plays backwards."
  field repeat Repeat "How it repeats."
  field speed f32 "Its rate (0: 1)."
  field fromMs f32 "Start this far in." {
    unit ms
  }
  field fromLabel String "Start at a label of its first timeline (wins over fromMs)."
  field hold bool "Keep the last values once it ends, until stopped."
  field decorative bool "Under reduced motion it does not play at all."
}
```

<sub>Module `ui` line 2514</sub>
