<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MenuDesc"></a>

### `MenuDesc` (desc struct)

A menu hanging from an element, or at a point (a context menu).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entries` | `readonly MenuEntry[]` | `[]` | — | — | — | Its entries, submenus' too. |
| `anchor` | `string` | `""` | — | — | — | The path of the element it hangs from. |
| `at` | `Vector2 \| null` | `null` | — | — | — | A point it opens at instead (CSS px): where a right press or a long press happened. |
| `side` | `FloatSide` | `"below"` | — | — | — | The side of the element it prefers. |
| `align` | `FloatAlign` | `"start"` | — | — | — | How it lines up with the element. |
| `minWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | Its least width; 0 is 200. |
| `maxWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | Its greatest width; 0 is 360 at 100 % text, growing with the player's text size. Never past the screen: past it a label wraps. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it. |

```ogt
struct MenuDesc {
  doc "A menu hanging from an element, or at a point (a context menu)."
  desc true
  field entries List.MenuEntry "Its entries, submenus' too."
  field anchor String "The path of the element it hangs from."
  field at Option.Vector2 "A point it opens at instead (CSS px): where a right press or a long press happened."
  field side FloatSide "The side of the element it prefers."
  field align FloatAlign "How it lines up with the element."
  field minWidth f32 "Its least width; 0 is 200." {
    unit cssPx
    min 0
  }
  field maxWidth f32 "Its greatest width; 0 is 360 at 100 % text, growing with the player's text size. Never past the screen: past it a label wraps." {
    unit cssPx
    min 0
  }
  field label String "What a screen reader calls it."
}
```

<sub>Module `ui` line 2195</sub>
