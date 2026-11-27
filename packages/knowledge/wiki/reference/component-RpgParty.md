<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgParty"></a>
### `RpgParty`

The party as a row of portrait cards: a level seal, the portrait, the name, and a health and a mana HudBar with their figures, filled in the page's health and mana colours. Reports choose (a member's key). The cards share the width evenly.

`import { RpgParty } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/party.tsx`).

| Prop | Type | What |
|---|---|---|
| `members?` | `ReadonlyArray<{ id: string; name: string; portrait: Texture \| 0; level: number; health: number; healthMax: number; mana: number; manaMax: number; down: boolean }>` | Keyed rows of { id, name, portrait, level, health, healthMax, mana, manaMax, down }: portrait a texture (og.load.texture), 0 none; down greys the card and marks it. |
| `selected` | `string` | The member chosen; empty is none. |
| `dense` | `boolean` | A wide but short place (a laptop window): wider, shorter portraits. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose (a member's key). |

Pack slots: `Chip.frame`.

Fixtures: `default`, `empty`.
