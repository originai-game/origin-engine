# HUD builtins

Default-style review for Pips, BossBar, Combo, PickupFeed, QtePrompt and
PerfReadout. All input values and timed outcomes belong to the game.

Build the native addon and og first. Each page needs its page command:

```sh
og ui preview packages/engine/sheets/hud-builtins --screen 'resources=style:default;page:resources;0.1:quit' --settle 0.05 --breakpoints desktop,phonePortrait --locales game --scales 100 --out /tmp/hud-builtins
```

Pages: resources, resource-states, activity, activity-states, prompts,
outcomes, waiting. They cover fractional/full/depleted/absent units,
boss phases/invulnerable/defeated/loading, combo idle/active/broken,
signed pickups and long names, unknown versus zero metrics, connection
states, and QTE press/hold/mash/waiting/disabled/success/failure/expired.

`bun test packages/engine/tests/hud-builtins.test.ts` uses the native host
to check every fixture on both widths, keyed intents through reorder,
inert resolved states, hold cancellation and keyboard activation.

Known core issue: `::-og-hold` paints against the control's larger hit
rectangle rather than its `::part(base)` visual box. Its wash can extend
below the visible hold button. The larger hit target is intentional; no
component-side sizing workaround is applied. The components lead has
carried the paint-box correction into the default-style redesign handoff.
