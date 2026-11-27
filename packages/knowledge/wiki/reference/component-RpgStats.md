<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgStats"></a>
### `RpgStats`

A character's figures in two columns of grouped rows, each an icon, a label and a value. Read only; fills its height and scrolls inside it.

`import { RpgStats } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/stats.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading (Stats). |
| `groups?` | `ReadonlyArray<{ id: string; column: string }>` | The groups, keyed rows of { id, column }: column a or b; a rule parts one group from the next in a column. |
| `rows?` | `ReadonlyArray<{ id: string; group: string; icon: string; label: string; value: string }>` | The figures, keyed rows of { id, group, icon, label, value }: group a group's id; value as the game words it. |

Pack slots: `Panel.frame`.

Fixtures: `default`.
