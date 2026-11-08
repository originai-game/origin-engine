<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SettingKind"></a>

### `SettingKind` (enum)

What a settings entry is: the control its row draws.

A string: one of the variants below. The zero value is `"toggle"`.

| Variant | Doc |
|---|---|
| `"toggle"` | On or off: a switch (`on`). |
| `"slider"` | A number in a range: a slider (`value`, min, max, step, unit). |
| `"choice"` | One of a few words: a segmented control up to four, else a dropdown (`index` of options). |
| `"binding"` | An input action's keys and buttons: og.ui.keyBindingRow, bound by physical key code and shown by the layout's label (the pattern's world). |
| `"action"` | A button that does something now (Reset tutorials, Calibrate). |
| `"custom"` | A row whose control the game builds: the custom callback, called with the entry's key inside the row. |

```ogt
enum SettingKind {
  doc "What a settings entry is: the control its row draws."
  variant toggle "On or off: a switch (`on`)."
  variant slider "A number in a range: a slider (`value`, min, max, step, unit)."
  variant choice "One of a few words: a segmented control up to four, else a dropdown (`index` of options)."
  variant binding "An input action's keys and buttons: og.ui.keyBindingRow, bound by physical key code and shown by the layout's label (the pattern's world)."
  variant action "A button that does something now (Reset tutorials, Calibrate)."
  variant custom "A row whose control the game builds: the custom callback, called with the entry's key inside the row."
}
```

<sub>Module `ui` line 4207</sub>
