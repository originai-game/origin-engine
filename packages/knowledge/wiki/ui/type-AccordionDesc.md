<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AccordionDesc"></a>

### `AccordionDesc` (desc struct)

An accordion: sections whose headers open and close their bodies.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sections` | `readonly AccordionSection[]` | `[]` | — | — | — | The sections, in order. |
| `expanded` | `readonly number[]` | `[]` | — | — | — | The open sections' indices (keep AccordionResult.expanded). |
| `single` | `boolean` | `false` | — | — | — | Opening one closes the others. |
| `dividers` | `boolean` | `true` | — | — | — | Hairlines between sections. |
| `gap` | `number` | `0` | ≥ 0 | `cssPx` | — | Space between sections. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |

```ogt
struct AccordionDesc {
  doc "An accordion: sections whose headers open and close their bodies."
  desc true
  field sections List.AccordionSection "The sections, in order."
  field expanded List.u32 "The open sections' indices (keep AccordionResult.expanded)."
  field single bool "Opening one closes the others."
  field dividers bool "Hairlines between sections." {
    default true
  }
  field gap f32 "Space between sections." {
    unit cssPx
    min 0
  }
  field width Option.Size "Its width; null grows."
}
```

<sub>`api.ogt` line 17771</sub>
