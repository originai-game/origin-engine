<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Cooldown"></a>
### `Cooldown`

A scene cell a game lays in a strip: its icon, the share still to run swept dark over it and the seconds over that, nudging when it comes ready. It draws the Ability component's pack slots, so it takes the theme's size, shape, art and sweep.

`import { Cooldown } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/cooldown.tsx`).

| Prop | Type | What |
|---|---|---|
| `remaining` | `number` | Seconds left; 0 is ready. |
| `total?` | `number` | The whole cooldown in seconds. Default 1. |
| `icon` | `string` | An icon of the atlas under the sweep. |
| `label` | `string` | Its accessible name (Dash). |
| `onUse?` | `(e: UiEvent) => void` | Use. |

Pack slots: `Ability.box`, `Ability.count`, `Ability.face`, `Ability.frame`, `Ability.icon`, `Ability.sweep`.

Fixtures: `default`, `ready`.
