<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChoreoCascade"></a>

### `ChoreoCascade` (struct)

Steps where each starts at the first `on` callback event of the one before (or its end): a score that counts chips, then multiplies, each step pitched higher.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `steps` | `readonly Choreo[]` | `[]` | — | — | — | The steps. |
| `on` | `string` | `""` | — | — | — | The callback that starts the next step. |

```ogt
struct ChoreoCascade {
  doc "Steps where each starts at the first `on` callback event of the one before (or its end): a score that counts chips, then multiplies, each step pitched higher."
  field steps List.Choreo "The steps."
  field on String "The callback that starts the next step."
}
```

<sub>`api.ogt` line 20828</sub>
