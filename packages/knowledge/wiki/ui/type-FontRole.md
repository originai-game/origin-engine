<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FontRole"></a>

### `FontRole` (enum)

Which configured font text uses; display and mono fall back to body, rounded to display, then body.

A string: one of the variants below. The zero value is `"body"`.

| Variant | Doc |
|---|---|
| `"body"` | The body font. |
| `"display"` | The display font. |
| `"mono"` | The monospaced font. |
| `"rounded"` | The rounded font, for levels and scores. |

```ogt
enum FontRole {
  doc "Which configured font text uses; display and mono fall back to body, rounded to display, then body."
  variant body "The body font."
  variant display "The display font."
  variant mono "The monospaced font."
  variant rounded "The rounded font, for levels and scores."
}
```

<sub>`api.ogt` line 15675</sub>
