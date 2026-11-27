<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-StrategyAlerts"></a>
### `StrategyAlerts`

Alerts and notices at the top left under the resources: one feed row each, newest first, lit from the edge it hangs on, a mark and an edge in danger, done or info, the game's words and the key that jumps to it; an older notice set back. The game owns which alerts stand and for how long; read only.

`import { StrategyAlerts } from "@origingame/engine/ui/components"` (kit `strategy`, `packages/engine/src/ui/components/strategy/alerts.tsx`).

| Prop | Type | What |
|---|---|---|
| `alerts?` | `ReadonlyArray<{ id: string; kind: string; text: string; key: string; faded: boolean }>` | The alerts and notices, newest first, keyed rows of { id, kind, text, key, faded }: kind danger (under attack), done (research complete) or info, on data-kind, each with its own mark and edge; text the game's words; key the input that jumps to it as the game names it (Space), on a key cap, empty none, hidden under a touch screen; faded an older notice, set back (data-faded). |

Fixtures: `default`, `info`, `overflow`.
