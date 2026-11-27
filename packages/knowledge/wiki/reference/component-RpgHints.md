<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgHints"></a>
### `RpgHints`

Input hints at the screen's foot: a row of key caps, each with what it does in small tracked capitals. The row only spaces them; a game lays a hint out anywhere else itself. Read only; hidden under a touch screen.

`import { RpgHints } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/hints.tsx`).

| Prop | Type | What |
|---|---|---|
| `hints?` | `ReadonlyArray<{ id: string; key: string; label: string }>` | The inputs worth a reminder, keyed rows of { id, key, label }: key the input as the game names it (Tab, Space) on a key cap, label what it does (Target). Under a touch screen the hints are hidden: the controls on screen say it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `overflow`.
