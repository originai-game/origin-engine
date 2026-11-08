<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Edges"></a>

### `Edges` (desc struct)

Insets of an absolute or anchored element from its parent's edges; an absent one is not set. start and end follow the writing direction: a right-to-left layout mirrors them.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `start` | `number \| null` | `null` | — | `cssPx` | — | From the start edge. |
| `top` | `number \| null` | `null` | — | `cssPx` | — | From the top. |
| `end` | `number \| null` | `null` | — | `cssPx` | — | From the end edge. |
| `bottom` | `number \| null` | `null` | — | `cssPx` | — | From the bottom. |
| `centerX` | `boolean` | `false` | — | — | — | Centred across the parent (start, if given, moves it). |
| `centerY` | `boolean` | `false` | — | — | — | Centred down the parent (top, if given, moves it). |

```ogt
struct Edges {
  doc "Insets of an absolute or anchored element from its parent's edges; an absent one is not set. start and end follow the writing direction: a right-to-left layout mirrors them."
  desc true
  field start Option.f32 "From the start edge." {
    unit cssPx
  }
  field top Option.f32 "From the top." {
    unit cssPx
  }
  field end Option.f32 "From the end edge." {
    unit cssPx
  }
  field bottom Option.f32 "From the bottom." {
    unit cssPx
  }
  field centerX bool "Centred across the parent (start, if given, moves it)."
  field centerY bool "Centred down the parent (top, if given, moves it)."
}
```

<sub>Module `ui` line 2285</sub>
