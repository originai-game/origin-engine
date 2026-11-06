<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-RollView"></a>

### `RollView` (struct)

An active check's roll.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stat` | `string` | `""` | — | — | — | The variable read as the stat. |
| `target` | `number` | `0` | — | — | — | What the roll plus the stat had to reach. |
| `roll` | `number` | `0` | — | — | — | The d20, 1 to 20. |
| `passed` | `boolean` | `false` | — | — | — | Whether it passed. |

```ogt
struct RollView {
  doc "An active check's roll."
  field stat String "The variable read as the stat."
  field target f32 "What the roll plus the stat had to reach."
  field roll u32 "The d20, 1 to 20."
  field passed bool "Whether it passed."
}
```

<sub>`api.ogt` line 14562</sub>
