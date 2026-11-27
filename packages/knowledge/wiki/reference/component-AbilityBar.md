<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-AbilityBar"></a>
### `AbilityBar`

A strip of AbilitySlot blocks, and nothing else: the slot draws itself, the strip places them. Each place is --ability-bar-slot-size wide and, where the strip is given less, shrinks alike down to --ability-bar-slot-min; under that the strip breaks into lines, so it never overflows. A game that wants another arrangement (two rows, groups, an ultimate at the end) lays AbilitySlot out itself in the same way.

`import { AbilityBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/ability-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `slots` | `ReadonlyArray<{ id: string; icon: string; name: string; action: string; hotkey: string; cooldown: number; cooldownMax: number; cost: string; unaffordable: boolean; locked: boolean; active: boolean; count?: number }>` | The slots, in order: each { id, icon, name, action, hotkey, cooldown, cooldownMax, cost, unaffordable, locked, active, count }, cooldown in seconds (0: ready); count above 0 shows how many are left (a quick item: potions, scrolls). A slot's full set of states is the AbilitySlot component. |
| `column?` | `boolean` | The slots run top to bottom; false is a row. |
| `onUse?` | `(e: UiEvent, id: string) => void` | Use. |

Fixtures: `default`.
