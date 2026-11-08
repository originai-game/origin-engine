<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BurstEvent"></a>

### `BurstEvent` (struct)

Rays and sparks from the target.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many (0: 12). |
| `radius` | `number` | `0` | — | `cssPx` | — | How far they reach (0: the style's). |
| `color` | `Color \| null` | `null` | — | — | — | Their colour; null is the style's highlight. |

```ogt
struct BurstEvent {
  doc "Rays and sparks from the target."
  field count u32 "How many (0: 12)."
  field radius f32 "How far they reach (0: the style's)." {
    unit cssPx
  }
  field color Option.Color "Their colour; null is the style's highlight."
}
```

<sub>Module `ui` line 1984</sub>
