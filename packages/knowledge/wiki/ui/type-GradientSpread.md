<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GradientSpread"></a>

### `GradientSpread` (enum)

What a gradient does past its ends.

A string: one of the variants below. The zero value is `"pad"`.

| Variant | Doc |
|---|---|
| `"pad"` | Holds the end stops' colours. |
| `"repeat"` | Starts over. |
| `"reflect"` | Runs back and forth. |

```ogt
enum GradientSpread {
  doc "What a gradient does past its ends."
  variant pad "Holds the end stops' colours."
  variant repeat "Starts over."
  variant reflect "Runs back and forth."
}
```

<sub>Module `ui` line 2574</sub>
