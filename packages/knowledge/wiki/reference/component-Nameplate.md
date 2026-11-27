<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Nameplate"></a>
### `Nameplate`

A unit's plate over its head: the level chip and the name in its reaction's ink, its title in small capitals, its health as a bar cut in quarters (a cream ghost of a hit, the shield after the fill) and the cast row (the spell and its time over a thin bar), all on a HUD plate (compact, the bar alone, has none).

`import { Nameplate } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/nameplate.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | The unit it sits over, by its bits (0: use at). |
| `at?` | `string` | A world point it sits over without an entity (x y z). |
| `height?` | `number` | World units above the point (its head). |
| `name` | `string` | The name, in the reaction's ink; a long one scrolls on one line. |
| `title?` | `string` | A line under it in small capitals (a role, a guild); empty is none. |
| `level?` | `string` | Its level on a chip as the game words it (32); empty is none, a boss's empty level draws a skull. |
| `reaction?` | `string` | Its colour: neutral (empty), friendly, hostile or player. |
| `health?` | `number` | Health 0 to 1 for a skewed bar cut in quarters under the name; negative draws none (a friendly NPC that cannot be hurt). When it drops, a cream ghost of what was lost holds 400 ms and drains in 300 ms. At 0.25 or less the rim turns danger and breathes, and the percent shows. |
| `shield?` | `number` | An absorb after the health, 0 to 1, in cyan. |
| `castName?` | `string` | The spell it is casting; empty is nothing. A boss or a caster sits above the interaction prompt when plates overlap. |
| `castProgress?` | `number` | How far the cast is, 0 to 1 (the game's progress, read every tick). |
| `castTime?` | `string` | The cast's time left as the game words it (1.2); empty is none. |
| `castState?` | `string` | The cast: casting (empty), locked (it can't be interrupted: steel grey with a shield mark) or interrupted (a red hatch and the word; the game shows it 600 ms, then clears castName). |
| `icon?` | `string` | A class, quest or role icon before the name; empty is none. |
| `elite?` | `boolean` | An elite: a gold level chip, gold wings at the bar's ends and a gold rim. |
| `boss?` | `boolean` | A boss: as an elite, with its name in tracked capitals, a longer bar and the percent. |
| `targeted?` | `boolean` | The player's target: 1.14 times larger, its name and rim white, white brackets that close in 120 ms; drawn over the rest. |
| `dead?` | `boolean` | Dead: the name greys and the bars go. |
| `lootable?` | `boolean` | Dead with loot: Lootable in gold under the name. |
| `compact?` | `boolean` | Far or crowded: only the bar (no name, title or cast). |
| `maxDistance?` | `number` | Hidden past this many world units (80). |
| `scaleAt?` | `number` | Distance at which the nameplate is drawn at scale 1. |
| `scaleMin?` | `number` | Minimum world-distance scale. |
| `scaleMax?` | `number` | Maximum world-distance scale. |
| `occlusion?` | `"fade" \| "hide" \| "none"` | Behaviour when behind world geometry. Default "fade". |

Pack slots: `Plate.frame`.

Fixtures: `default`, `friendly`, `neutral`, `hostile`, `player`, `boss`, `targeted`, `low`, `casting`, `locked`, `interrupted`, `dead`, `compact`, `overflow`.
