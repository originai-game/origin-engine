<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ListItem"></a>
### `ListItem`

One line of a list: what it is, its name over a supporting line, then what the game says about it at the trailing end (a value over a badge), then its state (chosen or refreshing). The trailing end stays on the row while the words keep a measure beside it; a long value on a narrow row drops under the words instead, with its state, so the name is not squeezed into a column.

`import { ListItem } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/list-item.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The row's primary text. |
| `subtitle?` | `string` | Optional secondary text, wrapping below the title. |
| `icon?` | `string` | Optional leading icon, used when image is zero. |
| `image?` | `Texture \| 0 \| undefined` | Optional loaded portrait or item texture. |
| `value?` | `string` | The game's formatted trailing value. |
| `badge?` | `string` | Optional trailing badge, such as New or 3. |
| `selected?` | `boolean` | Chosen by the game; marked with a check and aria-pressed. Default false. |
| `disabled?` | `boolean` | Unavailable; cannot choose. Default false. |
| `loading?` | `boolean` | Refreshing this row; prevents choosing and shows a spinner without hiding its content. Default false. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

Pack slots: `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `disabled`, `loading`, `overflow`.

Actions: `choose` (Choose this row (the enclosing collection's stable key)).
