<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `time` module

Simulation time. Game code never reads the wall clock; it reads Time.

Time groups slow or freeze some entities and not others (a bullet-time enemy beside a player at full speed, a frozen boss, a paused UI diorama): put an entity in a group with og.TimeGroup and set the group's scale with og.time.setGroupScale. Physics stays on the global scale, and timelines and the UI keep their own pace.

- **TypeScript**: functions are the namespace `og.time`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module time {
  doc "Simulation time. Game code never reads the wall clock; it reads Time."
  doc "Time groups slow or freeze some entities and not others (a bullet-time enemy beside a player at full speed, a frozen boss, a paused UI diorama): put an entity in a group with og.TimeGroup and set the group's scale with og.time.setGroupScale. Physics stays on the global scale, and timelines and the UI keep their own pace."
  # declarations follow, each on its own page
```

<sub>Module `time` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`og.Time`](time/record-Time.md) | resource |  | The fixed tick's clock. |
| [`og.TimeGroup`](time/record-TimeGroup.md) | component |  | Which time group the entity is in (0 to 15): its animation (og.anim, flipbooks, 2D skeletons), its scaled tweens and its og.nav agent advance by scaledDt times og.time.setGroupScale's scale for the g… |
| [`og.TimeGroups`](time/record-TimeGroups.md) | resource |  | Each time group's scale on top of the global one, set with og.time.setGroupScale (every group starts at 1). |
| [`og.time.setScale`](time/fn-setScale.md) | function | `(world: World, scale: number): Result<void>` | Set the global time scale (slow motion): physics, og.anim, flipbooks and tweens advance by dt * scale from the system after the call; systems may call it. |
| [`og.time.hitStop`](time/fn-hitStop.md) | function | `(world: World, ticks: number): Result<void>` | Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. |
| [`og.time.setGroupScale`](time/fn-setGroupScale.md) | function | `(world: World, group: number, scale: number): Result<void>` | Set one time group's scale on top of the global one: 0.25 a quarter speed, 0 frozen, 1 the global pace. |
| [`og.time.groupDt`](time/fn-groupDt.md) | function | `(world: World, entity: Entity): number` | This tick's scaled length for an entity: scaledDt times its time group's scale (scaledDt for an entity in no group). |
