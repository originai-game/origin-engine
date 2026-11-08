<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ValueFormat"></a>

### `ValueFormat` (desc struct)

How a control shows a number.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `decimals` | `number` | `-1` | — | — | — | Decimals; -1 follows the step (1 → 0, 0.5 → 1, 0.05 → 2). |
| `unit` | `string` | `""` | — | — | — | Written after it (`%`, ` m`, `×`); empty for none. |
| `percent` | `boolean` | `false` | — | — | — | Show 0–1 as a whole percent. |
| `labels` | `readonly string[]` | `[]` | — | — | — | Words for whole values from min (`Off`, `Low`, `Medium`, `High`): a setting's levels. |
| `digits` | `number` | `0` | — | — | — | Whole digits shown, zero-padded and not grouped (a clock's `05`, a year's `2026`); 0 groups thousands (`1,234`). |

```ogt
struct ValueFormat {
  doc "How a control shows a number."
  desc true
  field decimals i32 "Decimals; -1 follows the step (1 → 0, 0.5 → 1, 0.05 → 2)." {
    default -1
  }
  field unit String "Written after it (`%`, ` m`, `×`); empty for none."
  field percent bool "Show 0–1 as a whole percent."
  field labels List.String "Words for whole values from min (`Off`, `Low`, `Medium`, `High`): a setting's levels."
  field digits u32 "Whole digits shown, zero-padded and not grouped (a clock's `05`, a year's `2026`); 0 groups thousands (`1,234`)."
}
```

<sub>Module `ui` line 1906</sub>
