<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PlayInfo"></a>

### `PlayInfo` (struct)

Where a play is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `atMs` | `number` | `0` | — | `ms` | — | Its time (before a reverse's turn). |
| `lengthMs` | `number` | `0` | — | `ms` | — | Its length. |
| `paused` | `boolean` | `false` | — | — | — | Held. |
| `reverse` | `boolean` | `false` | — | — | — | Playing backwards. |
| `ended` | `boolean` | `false` | — | — | — | Finished (a held play keeps its values). |

```ogt
struct PlayInfo {
  doc "Where a play is."
  field atMs f32 "Its time (before a reverse's turn)." {
    unit ms
  }
  field lengthMs f32 "Its length." {
    unit ms
  }
  field paused bool "Held."
  field reverse bool "Playing backwards."
  field ended bool "Finished (a held play keeps its values)."
}
```

<sub>Module `ui` line 4331</sub>
