<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyValueRow"></a>

### `KeyValueRow` (desc struct)

A row of key-value rows.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | The key. |
| `value` | `string` | `""` | — | — | — | The value. |
| `hint` | `string` | `""` | — | — | — | A line under the key; empty is none. |
| `icon` | `string` | `""` | — | — | — | An icon before the key; empty is none. |
| `tone` | `string` | `""` | — | — | — | The value's ink: success, warn, danger, info, primary, gold, health, mana, xp; empty is ink. |
| `mono` | `boolean \| null` | `null` | — | — | — | The value in the mono face; null is when it reads as a number. |

```ogt
struct KeyValueRow {
  doc "A row of key-value rows."
  desc true
  field label String "The key."
  field value String "The value."
  field hint String "A line under the key; empty is none."
  field icon String "An icon before the key; empty is none."
  field tone String "The value's ink: success, warn, danger, info, primary, gold, health, mana, xp; empty is ink."
  field mono Option.bool "The value in the mono face; null is when it reads as a number."
}
```

<sub>Module `ui` line 2685</sub>
