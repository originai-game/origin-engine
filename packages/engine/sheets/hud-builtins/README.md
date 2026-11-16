# HUD builtins

Default-style review for Pips, BossBar, Combo, PickupFeed and
PerfReadout. All input values and timed outcomes belong to the game.

Build the native addon and og first. Each page needs its page command:

```sh
og ui preview packages/engine/sheets/hud-builtins --screen 'resources=style:default;page:resources;0.1:quit' --settle 0.05 --breakpoints desktop,phonePortrait --locales game --scales 100 --out /tmp/hud-builtins
```

Pages: resources, resource-states, activity, activity-states. They cover
fractional/full/depleted/absent units, boss phases/invulnerable/defeated,
combo idle/active/broken, signed pickups and long names, unknown versus
zero metrics, connection states, and empty/loading feeds and reserves.

`bun test packages/engine/tests/hud-builtins.test.ts` uses the native host
to check every fixture on both widths, fractional resource values,
game-supplied phases and outcomes, and zero versus unknown measurements.
