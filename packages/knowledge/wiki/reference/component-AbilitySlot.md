<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-AbilitySlot"></a>
### `AbilitySlot`

One ability or quick item: its icon on its face, the hotkey's cap, a cooldown sweep, charges, a count, its cost, ranks as pips and its states (unaffordable, out of range, active, locked, disabled, an ultimate's charge ring with its percent, then Ready). Every part is a pack slot of the Ability component, so a theme sets its size, shape, art, ring, glow, sweep and type. It reports onUse; the game decides whether it fires.

`import { AbilitySlot } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/ability-slot.tsx`).

| Prop | Type | What |
|---|---|---|
| `icon` | `string` | An icon of the atlas, drawn large on a gradient. |
| `name` | `string` | What a screen reader calls it (Fireball). |
| `action` | `string` | The action whose glyph the hotkey cap shows on the device used last (it follows a rebinding); empty is none. |
| `hotkey` | `string` | A label for the cap instead of the action's glyph (Q, LB); empty shows the action's. |
| `cooldown` | `number` | Seconds of cooldown left; 0 is ready. |
| `cooldownMax?` | `number` | The cooldown's length; the sweep shows cooldown over it. Default 1. |
| `charges` | `number` | Charges ready, in a corner badge when maxCharges is over 1. |
| `maxCharges` | `number` | Charges it holds; 0 or 1 is none. |
| `count` | `number` | How many are left (a quick item: potions, scrolls), at the bottom's end; 0 shows none. |
| `cost` | `string` | Its cost (40) at the bottom left while ready; empty is none. |
| `unaffordable?` | `boolean` | Not enough resource: the art tinted red and the cost in danger. |
| `outOfRange?` | `boolean` | The target is out of range: the art tinted red. |
| `active?` | `boolean` | Toggled on or channelling: a breathing glow and a primary edge. |
| `locked?` | `boolean` | Not learnt yet: dark art under a lock. |
| `disabled?` | `boolean` | Cannot be used now: the art washed out under a red slash. |
| `ultimate?` | `number` | An ultimate's charge 0 to 1: larger, a gold ring filling round it and a percent; negative is a plain slot. Default -1. |
| `level` | `number` | Ranks learnt (pips right under it). |
| `fit?` | `boolean` | It fills the place it is given (a strip's), rather than standing at the theme's ability size. |
| `maxLevel` | `number` | Ranks it has; 0 draws no pips. |
| `onUse?` | `(e: UiEvent) => void` | Use. |

Pack slots: `Ability.box`, `Ability.box.ultimate`, `Ability.chip.charges`, `Ability.chip.cost`, `Ability.chip.count`, `Ability.chip.key`, `Ability.chip.ready`, `Ability.chip.short`, `Ability.chip.ult`, `Ability.count`, `Ability.face`, `Ability.face.ultimate`, `Ability.frame`, `Ability.frame.disabled`, `Ability.frame.ready`, `Ability.frame.ultimate`, `Ability.icon`, `Ability.icon.disabled`, `Ability.pip`, `Ability.pip.on`, `Ability.ring`, `Ability.ring.fill`, `Ability.ring.ready`, `Ability.slash`, `Ability.sweep`, `Ability.sweep.ultimate`, `Ability.tint`.

Fixtures: `default`, `ultimate`, `ready`.
