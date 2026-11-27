<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-BossBar"></a>
### `BossBar`

A boss's bar across the top of the screen, on a HUD plate: a Boss tag, the name and its epithet; a long health bar with a warm ghost behind the fill and a diamond at each phase boundary; under it the stagger gauge cut into cells, the elements the boss is weak to and its health's share. The game owns every value; read only. The style's lean (skewX), the gauge's cut mask (notch marks stand in) and the weakness tiles' cut corners (rounded ones stand in) have no TSX surface; the invulnerable fill is greyed in kind (a grey fill, since the greying filter is dropped).

`import { BossBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/boss-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `name?` | `string` | The boss name; empty uses Boss. |
| `epithet?` | `string` | The boss's title as the game words it (Warden of the Ninefold Gate), beside the name; empty hides it. |
| `value?` | `number` | Health, clamped to the supplied range for display; its share of max stands in figures at the bar's end. |
| `max?` | `number` | Full health; a nonpositive capacity shows no health instead of dividing by zero. |
| `ghost?` | `number` | Health before the latest loss, held as long as the game wants it shown: a light ghost behind the fill, the health just lost. Absent, the ghost rests at the fill and lags it on each loss by itself. |
| `phases?` | `ReadonlyArray<BossBarPhase>` | Keyed phases in the fight's order; the game owns phase progression. A diamond on the bar marks each boundary between phases (the first phase is the bar's right end), dimmed once health has passed it. |
| `phase?` | `string` | The current phase's stable id; read out with the health. |
| `stagger?` | `number` | The stagger gauge's fill, 0 to 1 (the boss breaks at 1); negative hides the gauge. |
| `staggerCells?` | `number` | Cells the stagger gauge is cut into (10). |
| `weaknesses?` | `ReadonlyArray<BossBarWeakness>` | The elements the boss is weak to, keyed rows. |
| `invulnerable?` | `boolean` | Health remains visible, greyed, and a worded invulnerability state is shown. |
| `defeated?` | `boolean` | The game has confirmed defeat; this does not follow automatically from zero health. |
| `loading?` | `boolean` | Waiting for boss data; hides stale health and phases. |

Each `BossBarPhase`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `label?` | `string` |  |
| `complete?` | `boolean` |  |
| `at?` | `number` | Optional health fraction (0 to 1) where this phase begins; omitted distributes phases evenly. |

Each `BossBarWeakness`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name?` | `string` | For a reader. |
| `icon?` | `Texture \| 0` | A texture (og.load.texture); 0 draws the glyph in its place. |
| `glyph?` | `string` | An icon name, in art's place. |

Pack slots: `Bar.fill`, `Bar.frame`, `Bar.frame.low`, `Bar.frame.thick`, `Bar.ghost`, `Bar.tick`, `Bar.tone`, `Bar.tone.boss`, `Bar.tone.invulnerable`, `Bar.tone.stamina`, `Bar.trail`, `Boss.figure`, `Boss.phase`, `Boss.status`, `Plate.frame`.

Fixtures: `default`, `critical`, `invulnerable`, `defeated`, `loading`, `overflow`.
