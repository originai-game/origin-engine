<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-UiPickRule"></a>

### `UiPickRule` (struct)

A style rule that dresses a picked element's part.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selector` | `string` | `""` | — | — | — | Its selector, as the style writes it (`Button.base[primary]`). |
| `source` | `string` | `""` | — | — | — | Its file and line (`styles/fantasy/style.ogt:210`); empty for the engine's default style. |

```ogt
struct UiPickRule {
  doc "A style rule that dresses a picked element's part."
  field selector String "Its selector, as the style writes it (`Button.base[primary]`)."
  field source String "Its file and line (`styles/fantasy/style.ogt:210`); empty for the engine's default style."
}
```

<sub>`api.ogt` line 24063</sub>
