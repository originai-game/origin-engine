<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-UiPickElement"></a>

### `UiPickElement` (struct)

The UI element under a point (docs/ui.md §6): the topmost one that shows something or takes input; a layout container that draws nothing lets the point through.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id. |
| `path` | `string` | `""` | — | — | — | Its full path, as dump_ui and touch_move_to name it. |
| `kind` | `string` | `""` | — | — | — | What made it: label, button, panel, icon, a kit's own kind. |
| `text` | `string` | `""` | — | — | — | The text it shows; empty for none. |
| `rect` | `UiRect` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | — | Where it shows. |
| `layer` | `string \| null` | `null` | — | — | — | The screen whose layer it is in; null for code-built UI outside one. |
| `source` | `string \| null` | `null` | — | — | — | The document line that built it (`ui/shop.ogt:42`); null for code-built UI. |
| `part` | `string \| null` | `null` | — | — | — | The style part it is drawn as, with its variant and state flags (`Button.base[primary][hover]`); null for one no style dresses. |
| `rule` | `UiPickRule \| null` | `null` | — | — | — | The most specific rule that dresses it: the last of rules. |
| `rules` | `readonly UiPickRule[]` | `[]` | — | — | — | Every style rule that dresses its part, in the order they apply. |

```ogt
struct UiPickElement {
  doc "The UI element under a point (docs/ui.md §6): the topmost one that shows something or takes input; a layout container that draws nothing lets the point through."
  field id String "Its id."
  field path String "Its full path, as dump_ui and touch_move_to name it."
  field kind String "What made it: label, button, panel, icon, a kit's own kind."
  field text String "The text it shows; empty for none."
  field rect UiRect "Where it shows."
  field layer Option.String "The screen whose layer it is in; null for code-built UI outside one."
  field source Option.String "The document line that built it (`ui/shop.ogt:42`); null for code-built UI."
  field part Option.String "The style part it is drawn as, with its variant and state flags (`Button.base[primary][hover]`); null for one no style dresses."
  field rule Option.UiPickRule "The most specific rule that dresses it: the last of rules."
  field rules List.UiPickRule "Every style rule that dresses its part, in the order they apply."
}
```

<sub>Module `probe` line 30</sub>
