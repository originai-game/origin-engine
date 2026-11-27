<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgEnemyInfo"></a>
### `RpgEnemyInfo`

The targeted enemy: name, portrait, level, a health bar with its figure and its status effects. Read only.

`import { RpgEnemyInfo } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/enemy-info.tsx`).

| Prop | Type | What |
|---|---|---|
| `name` | `string` | The enemy's name. |
| `portrait` | `Texture \| 0` | Its portrait (og.load.texture); 0 is none. |
| `level` | `number` | Its level. |
| `health` | `number` | Health now. |
| `healthMax` | `number` | Most health. |
| `effects?` | `ReadonlyArray<{ id: string; icon: string; label: string; tone: string }>` | Status effects and traits, keyed rows of { id, icon, label, tone }: tone harm (a debuff on it), boon (a buff), weak (vulnerable to) or strong (resistant to). |

Pack slots: `Plate.frame`.

Fixtures: `default`.
