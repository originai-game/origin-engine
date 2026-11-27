<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgLockOn"></a>
### `RpgLockOn`

The lock-on reticle the page centres on the locked target: lit arcs round it with a glow in the HUD's accent. The game decides what is locked and where it stands on screen; read only.

`import { RpgLockOn } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/lock-on.tsx`).

| Prop | Type | What |
|---|---|---|
| `target` | `string` | The locked target's name for a reader; empty hides the reticle (nothing is locked). |

Fixtures: `default`, `empty`.
