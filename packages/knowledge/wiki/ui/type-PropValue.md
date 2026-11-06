<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PropValue"></a>

### `PropValue` (struct)

A value for an animatable prop.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `prop` | `UiProp` | `"x"` | — | — | — | The prop. |
| `value` | `number` | `0` | — | — | — | Its value. |

```ogt
struct PropValue {
  doc "A value for an animatable prop."
  field prop UiProp "The prop."
  field value f32 "Its value."
}
```

<sub>`api.ogt` line 20391</sub>
