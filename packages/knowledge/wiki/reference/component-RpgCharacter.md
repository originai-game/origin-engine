<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgCharacter"></a>
### `RpgCharacter`

A character sheet: a portrait with name and class, two columns of equipment slots either side of the full figure, and the attributes in two columns beneath. The game owns what fits where; the sheet reports choose (a slot's key) and unequip (X or pad Y on the focused slot, its key).

`import { RpgCharacter } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/character.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | A heading band over the figure (Equipment); empty shows the name header instead. |
| `name` | `string` | The character's name; with an empty title it heads the sheet beside the portrait. |
| `line` | `string` | The class and level line as the game words it (Rogue · Level 12). |
| `portrait` | `Texture \| 0` | A head-and-shoulders texture (og.load.texture); 0 hides it. |
| `figure` | `Texture \| 0` | A full-figure texture shown between the slot columns; 0 leaves the ground. |
| `slots?` | `ReadonlyArray<{ id: string; side: string; label: string; icon: string; rarity: string }>` | The equipment slots, keyed rows of { id, side, label, icon, rarity }: side is start or end (the column it stands in, mirrored right to left); an empty icon is an empty slot showing its label. |
| `selected` | `string` | The selected slot's id; empty is none. |
| `attributes?` | `ReadonlyArray<{ id: string; group: string; icon: string; label: string; value: string }>` | The attributes, keyed rows of { id, group, icon, label, value }: group a or b, the two columns; empty hides them (a page with RpgStats beside it). |
| `compact` | `boolean` | Its place is narrow (under about 300 px): the attributes (their split goes) tighten. The place decides, not the screen. |
| `dense` | `boolean` | A wide but short place (a laptop window): smaller slots and tighter attributes. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | The choose event, with the slot's key. |
| `onUnequip?` | `(e: UiEvent, id: string) => void` | Unequip, from the focused slot (X or pad Y, the details action), with the slot's key. |

Pack slots: `Button.plate`, `Panel.frame`.

Fixtures: `default`.
