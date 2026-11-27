<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgMenuBar"></a>
### `RpgMenuBar`

The menu's top bar: its screens as tabs and a close button. Reports choose (a screen's key), tabPrev, tabNext and close (the button, or the tabPrev, tabNext and back input actions).

`import { RpgMenuBar } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/menu-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `tabs?` | `ReadonlyArray<{ id: string; label: string }>` | The menu's screens, keyed rows of { id, label } (Character, Inventory, Skills, Quests, Map). |
| `tab` | `string` | The screen shown. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a screen (its key). |
| `onTabPrev?` | `(e: UiEvent) => void` | Previous screen. |
| `onTabNext?` | `(e: UiEvent) => void` | Next screen. |
| `onClose?` | `(e: UiEvent) => void` | Close (the button, Escape or B). |

Pack slots: `Button.plate`, `Tab.indicator`.

Fixtures: `default`.
