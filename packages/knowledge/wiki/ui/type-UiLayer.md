<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiLayer"></a>

### `UiLayer` (struct)

A layer of the stack (docs/ui.md §2.1), as its screen declares it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `serial` | `number` | `0` | — | — | — | Its number, unique for the UI's life: what push answered. |
| `screen` | `string` | `""` | — | — | — | The screen's name; also its layer's id, the first segment of its elements' paths. |
| `mount` | `string` | `""` | — | — | — | The store-driven mount that shows it, `scope:id` (a `type screen` document mounted with scope.mount is a layer while it is mounted); empty for a pushed screen. |
| `kind` | `string` | `""` | — | — | — | hud, menu, modal, popover, toast, tooltip or system. |
| `input` | `string` | `""` | — | — | — | game (input reaches gameplay), menu (input stays in the UI) or all. |
| `capture` | `boolean` | `false` | — | — | — | It takes the pointer on its gaps too. |
| `pause` | `boolean` | `false` | — | — | — | The game pauses while it is on the stack. |
| `under` | `string` | `""` | — | — | — | What happens to the layers under it: none, dim, blur or hide. |
| `back` | `string` | `""` | — | — | — | What Back does: pop, none or `emit <event>`. |
| `focus` | `string` | `""` | — | — | — | Where focus starts, a path below its root (empty: its first control). |
| `remember` | `boolean` | `false` | — | — | — | Focus returns where it was when it is the active layer again. |
| `top` | `boolean` | `false` | — | — | — | It is the topmost layer. |
| `active` | `boolean` | `false` | — | — | — | It is the active layer: the top one whose input is menu or all (the topmost with none). It takes Back, pop and focus. |
| `hidden` | `boolean` | `false` | — | — | — | A layer above hides it: it is not built. |
| `inert` | `boolean` | `false` | — | — | — | A menu layer above takes its input. |

```ogt
struct UiLayer {
  doc "A layer of the stack (docs/ui.md §2.1), as its screen declares it."
  field serial u32 "Its number, unique for the UI's life: what push answered."
  field screen String "The screen's name; also its layer's id, the first segment of its elements' paths."
  field mount String "The store-driven mount that shows it, `scope:id` (a `type screen` document mounted with scope.mount is a layer while it is mounted); empty for a pushed screen."
  field kind String "hud, menu, modal, popover, toast, tooltip or system."
  field input String "game (input reaches gameplay), menu (input stays in the UI) or all."
  field capture bool "It takes the pointer on its gaps too."
  field pause bool "The game pauses while it is on the stack."
  field under String "What happens to the layers under it: none, dim, blur or hide."
  field back String "What Back does: pop, none or `emit <event>`."
  field focus String "Where focus starts, a path below its root (empty: its first control)."
  field remember bool "Focus returns where it was when it is the active layer again."
  field top bool "It is the topmost layer."
  field active bool "It is the active layer: the top one whose input is menu or all (the topmost with none). It takes Back, pop and focus."
  field hidden bool "A layer above hides it: it is not built."
  field inert bool "A menu layer above takes its input."
}
```

<sub>Module `ui` line 3811</sub>
