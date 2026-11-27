<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgCombatLog"></a>
### `RpgCombatLog`

The combat log: a small band over a scroll of lines, the newest at the foot, each inked by its tone (damage red, heal green, crit warn; status reads plain, as the style's tone table words it). It scrolls inside the height it is given and fades at its foot. Read only.

`import { RpgCombatLog } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/combat-log.tsx`).

| Prop | Type | What |
|---|---|---|
| `lines?` | `ReadonlyArray<CombatLogLine>` | The log, oldest first. |

Each `CombatLogLine`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `text` | `string` | The line as the game words it (Sir Aldric uses Shield Bash on Cultist.). |
| `tone?` | `string` | damage, heal, status, crit or plain inks the line. |
| `detail?` | `string` | A quieter second line (→ 42 damage), empty for none. |

Pack slots: `Plate.frame`.

Fixtures: `default`.
