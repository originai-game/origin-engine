<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StatusGrid"></a>
### `StatusGrid`

Buffs and debuffs as a grid of tiles, the debuffs in their own row bordered by type, and the effects past the shown ones folded into a +n tile. Read only.

`import { StatusGrid } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/status-grid.tsx`).

| Prop | Type | What |
|---|---|---|
| `buffs?` | `ReadonlyArray<StatusGridEffect>` | The buffs, in the game's order. |
| `debuffs?` | `ReadonlyArray<StatusGridEffect>` | The debuffs, in their own row and bordered by type. |
| `more?` | `number` | Effects past the shown ones, folded into a +n tile (0: none). |

Each `StatusGridEffect`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `icon` | `string` | An icon of the atlas. |
| `name` | `string` | What a reader calls it (Haste). |
| `stacks?` | `number` | How many are stacked; 1 is plain. |
| `remaining?` | `number` | Seconds left (0 lasts until removed). |
| `duration?` | `number` | Seconds it runs (0 lasts until removed; past it, an extended effect, sweeps nothing). |
| `expiring?` | `boolean` | In its last five seconds. |
| `type?` | `string` | magic, curse, poison, disease or bleed (debuffs only). |

Fixtures: `default`.
