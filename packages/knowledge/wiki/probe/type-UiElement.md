<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-UiElement"></a>

### `UiElement` (struct)

One UI element with its bounds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its stable id, as the command channel targets it. |
| `parent` | `string` | `""` | — | — | — | Its parent's id; empty at the root. |
| `kind` | `string` | `""` | — | — | — | What made it: `label`, `button`, `icon`, `image`, `panel` and the other widget names; empty when the UI module does not say. |
| `disabled` | `boolean` | `false` | — | — | — | A disabled control (its text is exempt from contrast rules, as WCAG exempts inactive controls). |
| `clipped` | `boolean` | `false` | — | — | — | Only part of it shows: a scroll container or the surface cuts it, and the bounds are the part that shows. |
| `text` | `string` | `""` | — | — | — | The text it shows, if any. |
| `role` | `string` | `""` | — | — | — | Its accessibility role (og.ui.a11y): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, alert, dialog, tooltip, separator, group; empty for a presentational element (a layout container, a spacer). |
| `name` | `string` | `""` | — | — | — | Its accessible name: its text, a name the widget gives it (an icon's name, a dialog's title, Decrease and Increase for a stepper's buttons), a control's own text gathered from its children, or else its id. |
| `value` | `string` | `""` | — | — | — | Its value as text: a slider's or stepper's number, a progress's or meter's amount, a field's text, a select's choice; empty for none. |
| `states` | `readonly string[]` | `[]` | — | — | — | Its states: disabled, focused, focusable (keys and pads reach it now, in this order), checked, selected, expanded, collapsed (expandable and closed), pressed, busy, current (`aria-current`). |
| `opaque` | `boolean` | `false` | — | — | — | It draws an opaque background over its whole box (its fill's alpha times its opacity is 1), hiding the scene under it. |
| `x` | `number` | `0` | — | `cssPx` | — | Left edge. |
| `y` | `number` | `0` | — | `cssPx` | — | Top edge. |
| `width` | `number` | `0` | — | `cssPx` | — | Width. |
| `height` | `number` | `0` | — | `cssPx` | — | Height. |

```ogt
struct UiElement {
  doc "One UI element with its bounds."
  field id String "Its stable id, as the command channel targets it."
  field parent String "Its parent's id; empty at the root."
  field kind String "What made it: `label`, `button`, `icon`, `image`, `panel` and the other widget names; empty when the UI module does not say."
  field disabled bool "A disabled control (its text is exempt from contrast rules, as WCAG exempts inactive controls)."
  field clipped bool "Only part of it shows: a scroll container or the surface cuts it, and the bounds are the part that shows."
  field text String "The text it shows, if any."
  field role String "Its accessibility role (og.ui.a11y): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, alert, dialog, tooltip, separator, group; empty for a presentational element (a layout container, a spacer)."
  field name String "Its accessible name: its text, a name the widget gives it (an icon's name, a dialog's title, Decrease and Increase for a stepper's buttons), a control's own text gathered from its children, or else its id."
  field value String "Its value as text: a slider's or stepper's number, a progress's or meter's amount, a field's text, a select's choice; empty for none."
  field states List.String "Its states: disabled, focused, focusable (keys and pads reach it now, in this order), checked, selected, expanded, collapsed (expandable and closed), pressed, busy, current (`aria-current`)."
  field opaque bool "It draws an opaque background over its whole box (its fill's alpha times its opacity is 1), hiding the scene under it."
  field x f32 "Left edge." {
    unit cssPx
  }
  field y f32 "Top edge." {
    unit cssPx
  }
  field width f32 "Width." {
    unit cssPx
  }
  field height f32 "Height." {
    unit cssPx
  }
}
```

<sub>Module `probe` line 352</sub>
