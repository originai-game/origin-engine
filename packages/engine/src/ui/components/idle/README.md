# idle: built-in components

The resource bar, generators, the upgrade shop, prestige and the
welcome-back panel of an idle or incremental game. They draw what the game
tells them and report intents with stable row keys. They never compute
income, cycles, prices, bulk-buy counts, prestige gains or offline
earnings: every figure arrives from the game, formatted or as a number the
component only formats (`formatCompact`, `formatDuration`).

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `IdleGenerators` | `idle/generators` | The generator list of an idle game. |
| `IdleOffline` | `idle/offline` | The welcome-back panel of an idle game. |
| `IdlePrestige` | `idle/prestige` | The prestige panel of an idle game. |
| `IdlePurchases` | `idle/purchases` | The upgrade shop of an idle game. |
| `IdleResources` | `idle/resources` | The resource bar of an idle game. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only idle/` draws every member's fixtures under every pack.
