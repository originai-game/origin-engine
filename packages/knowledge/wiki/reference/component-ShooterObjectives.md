<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterObjectives"></a>
### `ShooterObjectives`

The objectives at the screen's top corner: a heading in the HUD's accent, the objective in play lit with its progress on a tag, the others dimmed under it with theirs (a phone shows the one in play alone). Read only. The tags' scrolling overflow (the svelte's -og-scroll marquee) has no TSX surface: long values truncate.

`import { ShooterObjectives } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/objectives.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The block's heading as the game words it; empty takes the component's Objective. |
| `goals?` | `ReadonlyArray<ShooterGoal>` | The objectives in the game's order. The one in play is lit with its progress on a tag, the others dimmed under it with theirs (a phone shows the one in play alone). The game owns progress and completion. Read only. |

Each `ShooterGoal`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `text` | `string` | The objective as the game words it. |
| `tag?` | `string` | Its progress or time on a tag (2 / 3, 0:45), empty for none. |
| `current?` | `boolean` | The one in play (lit; the others are dimmed). |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `one`, `empty`, `overflow`.
