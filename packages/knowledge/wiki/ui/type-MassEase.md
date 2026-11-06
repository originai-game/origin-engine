<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MassEase"></a>

### `MassEase` (enum)

How a mass element's timeline moves into a key.

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | Evenly. |
| `"out"` | Fast then slow. |
| `"in"` | Slow then fast. |
| `"back"` | Out, past the key and back (a pop). |
| `"step"` | Holds the key before until this one. |

```ogt
enum MassEase {
  doc "How a mass element's timeline moves into a key."
  variant linear "Evenly."
  variant out "Fast then slow."
  variant in "Slow then fast."
  variant back "Out, past the key and back (a pop)."
  variant step "Holds the key before until this one."
}
```

<sub>`api.ogt` line 23248</sub>
