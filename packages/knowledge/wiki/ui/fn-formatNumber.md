<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-formatNumber"></a>

### `og.ui.formatNumber`

A number written in a notation: grouped, compact, idle letters, scientific or engineering, with a sign, prefix and suffix and the locale's separators, and past 1e308 through its exponent. Deterministic: the same text on every host.

```ts
og.ui.formatNumber(value: number, format?: NumberFormatDesc): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `value` | `number` | no | The number (times 10^format.exponent). |
| `format` | `NumberFormatDesc` | yes | How to write it. |

**Returns** `string`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn formatNumber {
  doc "A number written in a notation: grouped, compact, idle letters, scientific or engineering, with a sign, prefix and suffix and the locale's separators, and past 1e308 through its exponent. Deterministic: the same text on every host."
  host false
  param value f64 "The number (times 10^format.exponent)."
  param format NumberFormatDesc "How to write it."
  returns String
}
```

<sub>Module `ui` line 3162</sub>
