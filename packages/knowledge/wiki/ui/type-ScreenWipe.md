<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ScreenWipe"></a>

### `ScreenWipe` (enum)

How a screen transition covers the old screen and reveals the new one.

A string: one of the variants below. The zero value is `"fade"`.

| Variant | Doc |
|---|---|
| `"fade"` | The cover fades in and out. |
| `"wipe"` | An edge sweeps across. |
| `"iris"` | A circle closes on the centre and opens again. |
| `"dissolve"` | Cells appear in a fixed scattered order. |
| `"slide"` | A sheet slides over and away. |

```ogt
enum ScreenWipe {
  doc "How a screen transition covers the old screen and reveals the new one."
  variant fade "The cover fades in and out."
  variant wipe "An edge sweeps across."
  variant iris "A circle closes on the centre and opens again."
  variant dissolve "Cells appear in a fixed scattered order."
  variant slide "A sheet slides over and away."
}
```

<sub>Module `ui` line 1873</sub>
