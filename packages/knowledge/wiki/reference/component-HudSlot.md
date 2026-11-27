<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudSlot"></a>
### `HudSlot`

One slot of a HUD's skill bar, a block a bar is built from: the art in an octagon with a lit edge, its cooldown a dark sweep with a lit leading edge and the time left, a slot that cannot be paid for greyed with its reason on it, charge pips over a slot that holds charges, a consumable's count on a tag, and the ultimate in a hexagon inside a gold ring that fills with its charge; a key cap under it. Its side is --hud-slot-size. The game owns every cooldown, cost and charge; read only. The style cuts the cell with clip-path; here the cell is rounded and the sweep and the ring are pack art (a nine-disc wipe and a nine-ring wipe).

`import { HudSlot } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/slot.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | What a reader calls it (Ember slash). |
| `icon?` | `Texture \| 0` | Its art, a texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon of the atlas, drawn when there is no art; empty is none. |
| `key?` | `string` | The input as the game names it (1, Q), on a key cap under the slot; empty is none. |
| `kind?` | `string` | skill, item (a consumable, its count on a tag) or ultimate (a hexagon inside a ring, gold by default, that fills with its charge). |
| `cooldown?` | `number` | The share of its cooldown still to run, 0 to 1: a dark sweep with a lit leading edge; 0 is none. |
| `time?` | `string` | The cooldown left as the game words it (12), over the sweep; empty is none. |
| `count?` | `number` | How many are left, on a tag at the slot's corner; 0 shows none. |
| `ready?` | `boolean` | Ready and worth pressing now: a lit edge and a glow (an ultimate at full charge has them without it). |
| `unaffordable?` | `boolean` | It cannot be paid for: the art greyed under a veil. |
| `reason?` | `string` | Why it cannot be used (No mana), on the slot while it is unaffordable and not cooling; empty is none. |
| `charges?` | `number` | Charges held, the lit pips over the slot. |
| `chargesMax?` | `number` | Charges it holds; 0 draws no pips. |
| `charge?` | `number` | An ultimate's charge, 0 to 1: its ring's lit arc and, under 1, the percent on it. |
| `class?` | `string` | Extra classes for the root, to size it (NSH style). |

Pack slots: `Slot.frame`.

Fixtures: `default`, `cooling`, `item`, `ultimate`, `unaffordable`, `empty`, `overflow`.
