<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-ChosenView"></a>

### `ChosenView` (struct)

The choice made while a line was shown.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | Its ChoiceView.index. |
| `text` | `string` | `""` | — | — | — | Its text, resolved in the active locale. |

```ogt
struct ChosenView {
  doc "The choice made while a line was shown."
  field index u32 "Its ChoiceView.index."
  field text String "Its text, resolved in the active locale."
}
```

<sub>`api.ogt` line 14604</sub>
