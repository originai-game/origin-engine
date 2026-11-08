<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NumberStepperDesc"></a>

### `NumberStepperDesc` (desc struct)

A number stepper: minus, the value and plus.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number \| null` | `null` | — | — | — | The least value; null is unbounded. |
| `max` | `number \| null` | `null` | — | — | — | The most; null is unbounded. |
| `step` | `number` | `1` | — | — | — | What each press adds or takes. |
| `largeStep` | `number` | `0` | — | — | — | What a held press's repeats move by after a second; 0 is ten steps. |
| `label` | `string` | `""` | — | — | — | Its name, before it in its row; empty for none. |
| `format` | `ValueFormat` | `{}` | — | — | — | How its value reads. |
| `wrap` | `boolean` | `false` | — | — | — | Past max it goes to min and back (needs both). |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |

```ogt
struct NumberStepperDesc {
  doc "A number stepper: minus, the value and plus."
  desc true
  field min Option.f32 "The least value; null is unbounded."
  field max Option.f32 "The most; null is unbounded."
  field step f32 "What each press adds or takes." {
    default 1
  }
  field largeStep f32 "What a held press's repeats move by after a second; 0 is ten steps."
  field label String "Its name, before it in its row; empty for none."
  field format ValueFormat "How its value reads."
  field wrap bool "Past max it goes to min and back (needs both)."
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null fits."
}
```

<sub>Module `ui` line 3144</sub>
