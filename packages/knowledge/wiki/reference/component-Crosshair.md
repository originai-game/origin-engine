<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Crosshair"></a>
### `Crosshair`

The reticle: four arms and a dot (or a ring and a dot), opening with the weapon's spread, the target's ink when the aim rests on an enemy, an optional reload progress ring, and four symmetric diagonal ticks confirming the last hit (red with a glow for a headshot and a kill, the shield's blue for a broken one).

`import { Crosshair } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/crosshair.tsx`).

| Prop | Type | What |
|---|---|---|
| `spread` | `number` | The weapon's current spread, 0 (settled) to 1 (widest): the arms move out from the centre. |
| `hit?` | `string \| undefined` | The last hit to confirm: none, hit, headshot, kill or shieldBreak. The game clears it to none when its mark should go. Default none. |
| `hitAt?` | `number \| undefined` | The id of that hit (its tick): a new id plays the mark again. |
| `aimed?` | `boolean \| undefined` | The aim rests on an enemy: the reticle takes the target ink. |
| `shape?` | `string \| undefined` | cross (four arms and a dot), dot, or circle (a ring and a dot). Default cross. |
| `reload?` | `number \| undefined` | Optional reload progress (0 to 1); draws a subtle progress ring around the reticle while reloading. |

Fixtures: `default`, `headshot`, `kill`, `circle`.
