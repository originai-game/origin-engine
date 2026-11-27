<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HoldButton"></a>
### `HoldButton`

A confirm that must be held (Delete save): its icon and words, what it says while it is held, and a line under it saying what to do. Holding it for `duration` seconds confirms; the hold's progress runs along the button's foot and the words swap while held.

`import { HoldButton } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hold-button.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | Its words and accessible name (Delete save). |
| `holding?` | `string` | What it says while it is held (Keep holding…); empty keeps its label. |
| `hint?` | `string` | A line under it, what to do (Hold to delete); empty for none. |
| `icon?` | `string` | An icon of the atlas before the label; empty for none. |
| `action?` | `string` | An og-input action whose glyph is shown as a hint; empty for none. |
| `duration?` | `number` | Hold duration in seconds. Default 1.2. |
| `variant?` | `string` | Its role: danger (the default, a destructive confirm), primary or secondary. Default danger. |
| `disabled?` | `boolean` | Shown but inert. |
| `onConfirm?` | `(e: UiEvent) => void` | Confirmed, by holding it for duration seconds. |

Pack slots: `Hold.fill`, `Hold.fill.…`, `Plate.frame`.

Fixtures: `default`, `label-only`, `disabled`, `overflow`.
