<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SemanticsDesc"></a>

### `SemanticsDesc` (desc struct)

What a component tells the accessibility tree about an element it built from containers (docs/ui.md §2): a role other than its kind's, a name, a value and states. Absent fields leave what the element's kind gives it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `role` | `string` | `""` | — | — | — | Its role: button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, tabpanel, radiogroup, progressbar, meter, timer, status, alert, log (a feed whose new lines are read: a killfeed, subtitles), dialog, alertdialog, tooltip, separator, group, region, navigation, toolbar, menu, menubar, menuitem, menuitemcheckbox, menuitemradio, list, listitem, tree, treeitem, table, grid, row, rowheader, columnheader, cell, gridcell, figure or scrollbar; empty keeps its kind's. |
| `name` | `string \| null` | `null` | — | — | — | Its accessible name; null keeps what it shows. |
| `description` | `string \| null` | `null` | — | — | — | Read after the name; null is none. |
| `value` | `string \| null` | `null` | — | — | — | Its value as people read it (`3 of 12`, `Sorted ascending`); null is none. |
| `checked` | `boolean \| null` | `null` | — | — | — | Checked (a menu item's check); null is not checkable. |
| `selected` | `boolean \| null` | `null` | — | — | — | Selected (a tab, a row, an option); null is not selectable. |
| `expanded` | `boolean \| null` | `null` | — | — | — | Expanded (an accordion header, a tree item, a menu button); null is not expandable. |
| `busy` | `boolean` | `false` | — | — | — | Loading. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert (a disabled menu item, a locked tab): disabled in the tree, and its interaction state group's disabled state. |
| `valueMin` | `number \| null` | `null` | — | — | — | A range's smallest value. |
| `valueMax` | `number \| null` | `null` | — | — | — | A range's largest value. |
| `valueNow` | `number \| null` | `null` | — | — | — | A range's value as a number (a scroll position, a page); null is no range. |

```ogt
struct SemanticsDesc {
  doc "What a component tells the accessibility tree about an element it built from containers (docs/ui.md §2): a role other than its kind's, a name, a value and states. Absent fields leave what the element's kind gives it."
  desc true
  field role String "Its role: button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, tabpanel, radiogroup, progressbar, meter, timer, status, alert, log (a feed whose new lines are read: a killfeed, subtitles), dialog, alertdialog, tooltip, separator, group, region, navigation, toolbar, menu, menubar, menuitem, menuitemcheckbox, menuitemradio, list, listitem, tree, treeitem, table, grid, row, rowheader, columnheader, cell, gridcell, figure or scrollbar; empty keeps its kind's."
  field name Option.String "Its accessible name; null keeps what it shows."
  field description Option.String "Read after the name; null is none."
  field value Option.String "Its value as people read it (`3 of 12`, `Sorted ascending`); null is none."
  field checked Option.bool "Checked (a menu item's check); null is not checkable."
  field selected Option.bool "Selected (a tab, a row, an option); null is not selectable."
  field expanded Option.bool "Expanded (an accordion header, a tree item, a menu button); null is not expandable."
  field busy bool "Loading."
  field disabled bool "Shown but inert (a disabled menu item, a locked tab): disabled in the tree, and its interaction state group's disabled state."
  field valueMin Option.f32 "A range's smallest value."
  field valueMax Option.f32 "A range's largest value."
  field valueNow Option.f32 "A range's value as a number (a scroll position, a page); null is no range."
}
```

<sub>Module `ui` line 3681</sub>
