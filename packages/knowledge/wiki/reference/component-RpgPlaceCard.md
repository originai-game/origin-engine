<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgPlaceCard"></a>
### `RpgPlaceCard`

A place picked on the map, on dark glass: its picture at the start fading into the glass, its name in the HUD's title type over a gold rule, a line about it, and two buttons, Mark (a lit ghost plate) and Fast travel (the gold primary plate). Reports mark (marking or unmarking) and travel (fast travel). Narrow the line about it is left out, so the name and the buttons keep their room.

`import { RpgPlaceCard } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/place-card.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The place's name. |
| `about` | `string` | A line or two about it; empty hides it. |
| `picture` | `Texture \| 0` | Its picture (og.load.texture); 0 hides it. |
| `marked` | `boolean` | The player has marked it: the button offers Unmark. |
| `canTravel` | `boolean` | Fast travel to it is possible: the button is enabled. |
| `onMark?` | `(e: UiEvent) => void` | Mark or unmark. |
| `onTravel?` | `(e: UiEvent) => void` | Fast travel. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Panel.frame`.

Fixtures: `default`, `marked`.
