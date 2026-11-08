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
}
```

<sub>Module `ui` line 3733</sub>
