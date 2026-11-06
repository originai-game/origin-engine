<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadioGroupDesc"></a>

### `RadioGroupDesc` (desc struct)

A radio group: its choices and layout.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `options` | `readonly RadioOption[]` | `[]` | — | — | — | The choices. |
| `selected` | `number` | `-1` | — | — | — | The chosen one from game state; -1 is none. |
| `label` | `string` | `""` | — | — | — | The group's name: shown above it and read by a screen reader; empty for none. |
| `variant` | `RadioVariant` | `"list"` | — | — | — | How it draws. |
| `row` | `boolean` | `false` | — | — | — | Lay the choices across (they wrap) rather than down. |
| `disabled` | `boolean` | `false` | — | — | — | Every choice inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |

```ogt
struct RadioGroupDesc {
  doc "A radio group: its choices and layout."
  desc true
  field options List.RadioOption "The choices."
  field selected i32 "The chosen one from game state; -1 is none." {
    default -1
  }
  field label String "The group's name: shown above it and read by a screen reader; empty for none."
  field variant RadioVariant "How it draws."
  field row bool "Lay the choices across (they wrap) rather than down."
  field disabled bool "Every choice inert."
  field width Option.Size "Its width; null fits."
}
```

<sub>`api.ogt` line 19452</sub>
