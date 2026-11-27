<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CombatHud"></a>
### `CombatHud`

The shooter's whole HUD: the reticle and the hit directions centred, the killfeed at the top's end, the vitals at the bottom's start and the ammunition at the bottom's end. The edges sit inside a fixed inset, so a phone's notch and home bar are the caller's (the svelte's safe-area insets and its right-to-left mirror are dropped), and a phone's compact reflow keeps the corners.

`import { CombatHud } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/combat-hud.tsx`).

| Prop | Type | What |
|---|---|---|
| `health?` | `number` | Health now. |
| `healthMax?` | `number` | Most health. Default 1. |
| `shield?` | `number` | Shield now, a strip over the health bar. |
| `shieldMax?` | `number` | Most shield; 0 hides the strip. |
| `spread?` | `number` | Crosshair spread, 0 to 1 (Crosshair). |
| `hit?` | `string` | Crosshair hit mark: none, hit, headshot, kill or shieldBreak. Default none. |
| `hitAt?` | `number` | The hit's id; a new one replays the mark. |
| `aimed?` | `boolean` | The aim rests on an enemy. |
| `shape?` | `string` | Crosshair shape: cross, dot or circle. Default cross. |
| `weapon?` | `string` | Weapon name (AmmoCounter). |
| `mode?` | `string` | Fire mode: auto, burst or single. Default auto. |
| `magazine?` | `number` | Rounds in the magazine. |
| `capacity?` | `number` | Magazine size. |
| `reserve?` | `number` | Rounds in reserve. |
| `reloading?` | `number` | Reload progress 0 to 1; negative when not reloading. Default -1. |
| `hits?` | `ReadonlyArray<DamageDirectionHit>` | Incoming hits (DamageDirection). |
| `feed?` | `ReadonlyArray<KillfeedRow>` | Killfeed rows, newest first (Killfeed). |

Pack slots: `Plate.frame`.

Fixtures: `default`.
