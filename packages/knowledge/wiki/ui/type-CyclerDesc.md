<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CyclerDesc"></a>

### `CyclerDesc` (desc struct)

An option cycler: a setting's levels in one row.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its name, before it; empty for none. |
| `wrap` | `boolean` | `true` | — | — | — | Past the last it goes to the first. |
| `pips` | `boolean` | `true` | — | — | — | A pip per option under the value, the current lit. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `valueWidth` | `number \| null` | `null` | — | `cssPx` | — | The value's width; null fits the longest option (so the arrows never move). |

```ogt
struct CyclerDesc {
  doc "An option cycler: a setting's levels in one row."
  desc true
  field label String "Its name, before it; empty for none."
  field wrap bool "Past the last it goes to the first." {
    default true
  }
  field pips bool "A pip per option under the value, the current lit." {
    default true
  }
  field disabled bool "Shown but inert."
  field width Option.Size "Its width; null grows."
  field valueWidth Option.f32 "The value's width; null fits the longest option (so the arrows never move)." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 19615</sub>
