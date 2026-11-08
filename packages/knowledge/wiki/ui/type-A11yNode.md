<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-A11yNode"></a>

### `A11yNode` (struct)

One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Its path, as touch_move_to and og.ui.focus name it. |
| `parent` | `string` | `""` | — | — | — | The path of the nearest node above it in the tree; empty at the top. |
| `role` | `string` | `""` | — | — | — | Its role (probe.UiElement.role): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, dialog, tooltip, separator, group, list, listitem, and the landmarks navigation, main, banner, contentinfo, complementary, article, region. |
| `name` | `string` | `""` | — | — | — | Its accessible name: what a screen reader says it is. A text field or area is named by the label before it in its row, else by its placeholder, else by its id; never by the text in it (that is its value). |
| `description` | `string` | `""` | — | — | — | Read after the name (aria-description): a text field's placeholder when a label names it; empty for none. |
| `value` | `string` | `""` | — | — | — | Its value as text (a slider's number, a progress's percent, a field's text); empty for none. |
| `states` | `readonly string[]` | `[]` | — | — | — | Its states: disabled, focused, focusable, checked, selected, expanded, collapsed (expandable and closed), pressed, busy, offscreen (a scroll container cuts it out of view). |
| `valueMin` | `number \| null` | `null` | — | — | — | A range's smallest value (aria-valuemin): a slider's min, a bounded stepper's, 0 for a progress (in percent) and a meter; null for a stepper without bounds and a control that is no range. |
| `valueMax` | `number \| null` | `null` | — | — | — | A range's largest value (aria-valuemax): a slider's max, a bounded stepper's, 100 for a progress, a meter's max; null as valueMin is. |
| `valueNow` | `number \| null` | `null` | — | — | — | A range's value as a number (aria-valuenow; value says it as people read it): a slider's, a stepper's, a meter's, a progress's percent; null for a control that is no range. |
| `level` | `number \| null` | `null` | — | — | — | A heading's level, 1 to 6 (aria-level; <h1> to <h6>); null for anything else. |
| `x` | `number` | `0` | — | `cssPx` | — | Left edge of the whole element. |
| `y` | `number` | `0` | — | `cssPx` | — | Top edge of the whole element. |
| `width` | `number` | `0` | — | `cssPx` | — | Width. |
| `height` | `number` | `0` | — | `cssPx` | — | Height. |

```ogt
struct A11yNode {
  doc "One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out)."
  field path String "Its path, as touch_move_to and og.ui.focus name it."
  field parent String "The path of the nearest node above it in the tree; empty at the top."
  field role String "Its role (probe.UiElement.role): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, dialog, tooltip, separator, group, list, listitem, and the landmarks navigation, main, banner, contentinfo, complementary, article, region."
  field name String "Its accessible name: what a screen reader says it is. A text field or area is named by the label before it in its row, else by its placeholder, else by its id; never by the text in it (that is its value)."
  field description String "Read after the name (aria-description): a text field's placeholder when a label names it; empty for none."
  field value String "Its value as text (a slider's number, a progress's percent, a field's text); empty for none."
  field states List.String "Its states: disabled, focused, focusable, checked, selected, expanded, collapsed (expandable and closed), pressed, busy, offscreen (a scroll container cuts it out of view)."
  field valueMin Option.f32 "A range's smallest value (aria-valuemin): a slider's min, a bounded stepper's, 0 for a progress (in percent) and a meter; null for a stepper without bounds and a control that is no range."
  field valueMax Option.f32 "A range's largest value (aria-valuemax): a slider's max, a bounded stepper's, 100 for a progress, a meter's max; null as valueMin is."
  field valueNow Option.f32 "A range's value as a number (aria-valuenow; value says it as people read it): a slider's, a stepper's, a meter's, a progress's percent; null for a control that is no range."
  field level Option.u32 "A heading's level, 1 to 6 (aria-level; <h1> to <h6>); null for anything else."
  field x f32 "Left edge of the whole element." {
    unit cssPx
  }
  field y f32 "Top edge of the whole element." {
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

<sub>Module `ui` line 1737</sub>
