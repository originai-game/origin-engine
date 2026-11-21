# strategy: built-in components

The RTS and tower-defence HUD (the resource bar, the match clock, the wave,
alerts, control groups, the minimap, the selection panel, the production
queue, the command card) and the tower-upgrade inspector. They draw what
the game tells them and report intents with stable row keys. They never
compute costs, build times, cooldowns, legality, income, wave timing,
rewards, refunds or stats: every figure arrives formatted from the game,
and every flag (enabled, affordable, full, active, alert, can call early,
can sell) is the game's decision.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `StrategyAlerts` | `strategy/alerts` | Alerts and notices at the top left under the resources. |
| `StrategyClock` | `strategy/clock` | The match clock at the top right. |
| `StrategyCommands` | `strategy/commands` | The command card at the bottom right. |
| `StrategyGroups` | `strategy/groups` | The control groups over the selection. |
| `StrategyMinimap` | `strategy/minimap` | The minimap at the bottom left. |
| `StrategyQueue` | `strategy/queue` | A production queue as one row. |
| `StrategyResources` | `strategy/resources` | The resource bar at the top left. |
| `StrategySelection` | `strategy/selection` | The selection panel at the foot. |
| `StrategyTower` | `strategy/tower` | A tower's inspector. |
| `StrategyWave` | `strategy/wave` | The wave readout at the top right. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only strategy/` draws every member's fixtures under every pack.
