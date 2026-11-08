<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Repeat"></a>

### `Repeat` (enum)

How a play repeats.

A string: one of the variants below. The zero value is `"once"`.

| Variant | Doc |
|---|---|
| `"once"` | Once (the default). |
| `"loop"` | Over and over. |
| `"pingPong"` | Forward then back, over and over. |

```ogt
enum Repeat {
  doc "How a play repeats."
  variant once "Once (the default)."
  variant loop "Over and over."
  variant pingPong "Forward then back, over and over."
}
```

<sub>Module `ui` line 4397</sub>
