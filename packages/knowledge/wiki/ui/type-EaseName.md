<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-EaseName"></a>

### `EaseName` (enum)

A named easing curve.

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | Constant speed. |
| `"out"` | How things arrive: fast, then settling (the default). |
| `"exit"` | How things leave: slow, then away. |
| `"inOut"` | Slow at both ends. |

```ogt
enum EaseName {
  doc "A named easing curve."
  variant linear "Constant speed."
  variant out "How things arrive: fast, then settling (the default)."
  variant exit "How things leave: slow, then away."
  variant inOut "Slow at both ends."
}
```

<sub>Module `ui` line 4449</sub>
