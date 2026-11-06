<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TabBarDesc"></a>

### `TabBarDesc` (desc struct)

Tabs over the panel the game builds for the selected one.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tabs` | `readonly TabItem[]` | `[]` | — | — | — | The tabs. |
| `selected` | `number` | `0` | — | — | — | The selected tab (keep TabBarResult.selected). |
| `variant` | `TabVariant` | `"underline"` | — | — | — | How the selected tab is marked. |
| `bumpers` | `boolean` | `true` | — | — | — | The tabPrev and tabNext actions (Q and E, LB and RB) step through the tabs, their glyphs on the device used last (keyboard, Xbox, PlayStation, Switch) at the row's ends. |
| `wrap` | `boolean` | `true` | — | — | — | Stepping past the last tab comes back to the first. |
| `fill` | `boolean` | `false` | — | — | — | The tabs share the row's width. |
| `gap` | `number \| null` | `null` | — | `cssPx` | — | Space between the row and the panel; null is space.md. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | The panel's height; null fits. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the tab list. |

```ogt
struct TabBarDesc {
  doc "Tabs over the panel the game builds for the selected one."
  desc true
  field tabs List.TabItem "The tabs."
  field selected u32 "The selected tab (keep TabBarResult.selected)."
  field variant TabVariant "How the selected tab is marked."
  field bumpers bool "The tabPrev and tabNext actions (Q and E, LB and RB) step through the tabs, their glyphs on the device used last (keyboard, Xbox, PlayStation, Switch) at the row's ends." {
    default true
  }
  field wrap bool "Stepping past the last tab comes back to the first." {
    default true
  }
  field fill bool "The tabs share the row's width."
  field gap Option.f32 "Space between the row and the panel; null is space.md." {
    unit cssPx
  }
  field width Option.Size "Its width; null grows."
  field height Option.Size "The panel's height; null fits."
  field label String "What a screen reader calls the tab list."
}
```

<sub>`api.ogt` line 18015</sub>
