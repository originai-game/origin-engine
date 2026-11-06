<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AccordionSection"></a>

### `AccordionSection` (desc struct)

A section of an accordion: its header's title, subtitle and badge.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | The header's title. |
| `subtitle` | `string` | `""` | — | — | — | A line under it; empty is none. |
| `badge` | `string` | `""` | — | — | — | A badge at the header's end; empty is none. |
| `badgeTone` | `Tone` | `"neutral"` | — | — | — | The badge's tone. |
| `disabled` | `boolean` | `false` | — | — | — | The header is shown but does not open. |

```ogt
struct AccordionSection {
  doc "A section of an accordion: its header's title, subtitle and badge."
  desc true
  field title String "The header's title."
  field subtitle String "A line under it; empty is none."
  field badge String "A badge at the header's end; empty is none."
  field badgeTone Tone "The badge's tone."
  field disabled bool "The header is shown but does not open."
}
```

<sub>Module `ui` line 2122</sub>
