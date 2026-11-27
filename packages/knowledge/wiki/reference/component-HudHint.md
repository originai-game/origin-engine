<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudHint"></a>
### `HudHint`

One input hint of a HUD, a block a hint row is built from: a key cap and what it does in small tracked capitals. Read only; hidden under a touch screen, where the controls on screen say it. No strings of its own; the svelte file carries none either.

`import { HudHint } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hint.tsx`).

| Prop | Type | What |
|---|---|---|
| `key?` | `string` | The input as the game names it (Tab, Space), on a key cap; empty is none. |
| `label?` | `string` | What it does (Target), in small tracked capitals. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `empty`, `overflow`.
