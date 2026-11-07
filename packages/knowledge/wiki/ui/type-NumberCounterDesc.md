<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NumberCounterDesc"></a>

### `NumberCounterDesc` (desc struct)

A big number that counts to its value.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `format` | `NumberFormatDesc` | `{}` | — | — | — | How it is written. |
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is xl3. |
| `font` | `FontRole` | `"mono"` | — | — | — | The font; the default is mono (tabular digits that do not jitter). |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is ink. |
| `label` | `string` | `""` | — | — | — | A caption under it (Score, Gold); empty is none. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas before it (coin); empty is none. |
| `roll` | `boolean` | `true` | — | — | — | Each digit rolls to its next like an odometer while it counts (the default); off, the number counts in place. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long a change takes to count; 0 is 600 ms (a rise of more than a tenth takes up to 1.2 s). |
| `delta` | `boolean` | `true` | — | — | — | A rise throws its +n off the number, rising and fading (the default). |
| `ticks` | `boolean` | `false` | — | — | — | Play counterTick as it counts, pitched higher through the count. |
| `anchor` | `Anchor \| null` | `null` | — | — | — | Opens its own anchored group there (on the HUD chip); null builds it in place. |
| `align` | `TextAlign` | `"start"` | — | — | — | How it aligns in its column. |

```ogt
struct NumberCounterDesc {
  doc "A big number that counts to its value."
  desc true
  field format NumberFormatDesc "How it is written."
  field step Option.TypeStepName "The type step; null is xl3."
  field font FontRole "The font; the default is mono (tabular digits that do not jitter)." {
    default mono
  }
  field color Option.Paint "The ink; null is ink."
  field label String "A caption under it (Score, Gold); empty is none."
  field icon String "An icon of the atlas before it (coin); empty is none."
  field roll bool "Each digit rolls to its next like an odometer while it counts (the default); off, the number counts in place." {
    default true
  }
  field durationMs f32 "How long a change takes to count; 0 is 600 ms (a rise of more than a tenth takes up to 1.2 s)." {
    unit ms
    min 0
  }
  field delta bool "A rise throws its +n off the number, rising and fading (the default)." {
    default true
  }
  field ticks bool "Play counterTick as it counts, pitched higher through the count."
  field anchor Option.Anchor "Opens its own anchored group there (on the HUD chip); null builds it in place."
  field align TextAlign "How it aligns in its column."
}
```

<sub>Module `ui` line 5627</sub>
