<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `time` module

Simulation time. Game code never reads the wall clock; it reads Time.

Time groups slow or freeze some entities and not others (a bullet-time enemy beside a player at full speed, a frozen boss, a paused UI diorama): put an entity in a group with og.TimeGroup and set the group's scale with og.time.setGroupScale. og.anim, flipbooks, 2D skeletons, the entity's scaled tweens and its og.nav agent then advance by scaledDt times its group's scale, and its og.ParticleSystem's particles spawn, age and move at the group's scale (the renderer's particles do not follow the global scale or hit stop, so only the group's applies). Physics stays on the global scale (one solver steps every body together; slow a body's motion with its velocity, or move it kinematically), and timelines and the UI keep their own pace. A world that never names og.TimeGroup or og.TimeGroups keeps its saves and hash.

- **TypeScript**: functions are the namespace `og.time`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-feel`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module time {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1602</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Time`](#record-Time) | resource | The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers. The world keeps the time scale in a second resource, og.TimeScale (`scale`, `hitStop`, and `elapsed`, which is scaledElapsed), set through og.time.setScale and og.time.hitStop; read og.Time, which carries all of it. The probe's `get:og.Time` answers every field here, as a system's `time` does; `set:og.TimeScale.scale=0.5` slows time from a script. |
| [`og.TimeGroup`](#record-TimeGroup) | component | Which time group the entity is in (0 to 15): its animation (og.anim, flipbooks, 2D skeletons), its scaled tweens and its og.nav agent advance by scaledDt times og.time.setGroupScale's scale for the group, and its og.ParticleSystem's particles at the group's scale. Physics stays on the global scale: a body in a slowed group falls at the world's pace. An entity without one is at the global pace. |
| [`og.TimeGroups`](#record-TimeGroups) | resource | Each time group's scale on top of the global one, set with og.time.setGroupScale (every group starts at 1). Registered when a game first names og.TimeGroup or og.TimeGroups. Physics stays on the global scale. |
| [`og.time.setScale`](#fn-setScale) | function | Set the global time scale (slow motion): physics, og.anim, flipbooks and tweens advance by dt * scale from the system after the call; systems may call it. |
| [`og.time.hitStop`](#fn-hitStop) | function | Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. A longer hit stop in progress is never shortened. Typical values at 60 Hz: 2-3 ticks for a light hit, 4-6 medium, 8-12 heavy, 12-20 a finishing blow (paired with slow motion). |
| [`og.time.setGroupScale`](#fn-setGroupScale) | function | Set one time group's scale on top of the global one: 0.25 a quarter speed, 0 frozen, 1 the global pace. Entities with og.TimeGroup in it animate, tween and walk at the new pace from the system after the call; physics stays on the global scale. Outside a system the first call registers og.TimeGroups; inside one it must be registered already (name og.TimeGroup or og.TimeGroups in a system's reads or writes, or call it once in setup). |
| [`og.time.groupDt`](#fn-groupDt) | function | This tick's scaled length for an entity: scaledDt times its time group's scale (scaledDt for an entity in no group). Move an entity's own gameplay by it so a slowed group slows everything it does. |

## Functions

<a id="fn-setScale"></a>

### `og.time.setScale`

Set the global time scale (slow motion): physics, og.anim, flipbooks and tweens advance by dt * scale from the system after the call; systems may call it.

```ts
og.time.setScale(world: World, scale: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `scale` | `number` | no | 0 to 10: 1 is normal speed. |

**Returns** `Result<void>`: Refused outside 0..10.

Binding symbol `og_time_set_scale` · publish builds: kept

```ogt
fn setScale {
  doc "Set the global time scale (slow motion): physics, og.anim, flipbooks and tweens advance by dt * scale from the system after the call; systems may call it."
  param world World "The world."
  param scale f32 "0 to 10: 1 is normal speed."
  returns Result.Void "Refused outside 0..10."
}
```

<sub>`api.ogt` line 1657</sub>

<a id="fn-hitStop"></a>

### `og.time.hitStop`

Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. A longer hit stop in progress is never shortened. Typical values at 60 Hz: 2-3 ticks for a light hit, 4-6 medium, 8-12 heavy, 12-20 a finishing blow (paired with slow motion).

```ts
og.time.hitStop(world: World, ticks: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `ticks` | `number` | no | Ticks to hold. |

**Returns** `Result<void>`.

Binding symbol `og_time_hit_stop` · publish builds: kept

```ogt
fn hitStop {
  doc "Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. A longer hit stop in progress is never shortened. Typical values at 60 Hz: 2-3 ticks for a light hit, 4-6 medium, 8-12 heavy, 12-20 a finishing blow (paired with slow motion)."
  param world World "The world."
  param ticks u32 "Ticks to hold."
  returns Result.Void
}
```

<sub>`api.ogt` line 1664</sub>

<a id="fn-setGroupScale"></a>

### `og.time.setGroupScale`

Set one time group's scale on top of the global one: 0.25 a quarter speed, 0 frozen, 1 the global pace. Entities with og.TimeGroup in it animate, tween and walk at the new pace from the system after the call; physics stays on the global scale. Outside a system the first call registers og.TimeGroups; inside one it must be registered already (name og.TimeGroup or og.TimeGroups in a system's reads or writes, or call it once in setup).

```ts
og.time.setGroupScale(world: World, group: number, scale: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `number` | no | The group, 0 to 15. |
| `scale` | `number` | no | 0 to 10: 1 is the global pace. |

**Returns** `Result<void>`: Refused for a group past 15, a scale outside 0..10, or inside a system before og.TimeGroups is registered.

Binding symbol `og_time_set_group_scale` · publish builds: kept

```ogt
fn setGroupScale {
  doc "Set one time group's scale on top of the global one: 0.25 a quarter speed, 0 frozen, 1 the global pace. Entities with og.TimeGroup in it animate, tween and walk at the new pace from the system after the call; physics stays on the global scale. Outside a system the first call registers og.TimeGroups; inside one it must be registered already (name og.TimeGroup or og.TimeGroups in a system's reads or writes, or call it once in setup)."
  param world World "The world."
  param group u32 "The group, 0 to 15."
  param scale f32 "0 to 10: 1 is the global pace."
  returns Result.Void "Refused for a group past 15, a scale outside 0..10, or inside a system before og.TimeGroups is registered."
}
```

<sub>`api.ogt` line 1671</sub>

<a id="fn-groupDt"></a>

### `og.time.groupDt`

This tick's scaled length for an entity: scaledDt times its time group's scale (scaledDt for an entity in no group). Move an entity's own gameplay by it so a slowed group slows everything it does.

```ts
og.time.groupDt(world: World, entity: Entity): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity. |

**Returns** `number`.

Binding symbol `og_time_group_dt` · publish builds: kept

```ogt
fn groupDt {
  doc "This tick's scaled length for an entity: scaledDt times its time group's scale (scaledDt for an entity in no group). Move an entity's own gameplay by it so a slowed group slows everything it does."
  param world World "The world."
  param entity Entity "The entity."
  returns f32
}
```

<sub>`api.ogt` line 1679</sub>

## Components, resources and events

<a id="record-Time"></a>

### `og.Time` (resource)

The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers. The world keeps the time scale in a second resource, og.TimeScale (`scale`, `hitStop`, and `elapsed`, which is scaledElapsed), set through og.time.setScale and og.time.hitStop; read og.Time, which carries all of it. The probe's `get:og.Time` answers every field here, as a system's `time` does; `set:og.TimeScale.scale=0.5` slows time from a script.

Token `og.Time`, registered under the schema name `og.Time`; its value type is `Time`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | `ticks` | read only | Fixed steps taken since load. |
| `dt` | `number` | `0` | — | `s` | read only | The length of one tick: 1 / rate. |
| `elapsed` | `number` | `0` | — | `s` | read only | Simulation time since load: tick * dt. |
| `rate` | `number` | `0` | — | `hz` | read only | The tick rate, from GameDesc.tick. |
| `scale` | `number` | `1` | 0 … 10 | — | read only | Global slow motion, set with og.time.setScale: 1 is normal, 0.5 half speed, 0 stopped. Physics, animation, flipbooks and tweens advance by scaledDt; the tick itself, the UI, input and the probe keep their pace. |
| `hitStop` | `number` | `0` | — | `ticks` | read only | Ticks of hit stop left (og.time.hitStop): while above 0, scaledDt is 0. |
| `scaledDt` | `number` | `0` | — | `s` | read only | This tick's scaled length: 0 during a hit stop, else dt * scale. Move gameplay by it so slow motion and hit stop reach game code too. |
| `scaledElapsed` | `number` | `0` | — | `s` | read only | Scaled seconds since load: the sum of every finished tick's scaledDt. |

```ogt
resource Time {
  doc "The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers. The world keeps the time scale in a second resource, og.TimeScale (`scale`, `hitStop`, and `elapsed`, which is scaledElapsed), set through og.time.setScale and og.time.hitStop; read og.Time, which carries all of it. The probe's `get:og.Time` answers every field here, as a system's `time` does; `set:og.TimeScale.scale=0.5` slows time from a script."
  field tick u32 "Fixed steps taken since load." {
    readonly true
    unit ticks
  }
  field dt f32 "The length of one tick: 1 / rate." {
    readonly true
    unit s
  }
  field elapsed f64 "Simulation time since load: tick * dt." {
    readonly true
    unit s
  }
  field rate u32 "The tick rate, from GameDesc.tick." {
    readonly true
    unit hz
  }
  field scale f32 "Global slow motion, set with og.time.setScale: 1 is normal, 0.5 half speed, 0 stopped. Physics, animation, flipbooks and tweens advance by scaledDt; the tick itself, the UI, input and the probe keep their pace." {
    readonly true
    default 1
    min 0
    max 10
  }
  field hitStop u32 "Ticks of hit stop left (og.time.hitStop): while above 0, scaledDt is 0." {
    readonly true
    unit ticks
  }
  field scaledDt f32 "This tick's scaled length: 0 during a hit stop, else dt * scale. Move gameplay by it so slow motion and hit stop reach game code too." {
    readonly true
    unit s
  }
  field scaledElapsed f64 "Scaled seconds since load: the sum of every finished tick's scaledDt." {
    readonly true
    unit s
  }
}
```

<sub>`api.ogt` line 1607</sub>

<a id="record-TimeGroup"></a>

### `og.TimeGroup` (component)

Which time group the entity is in (0 to 15): its animation (og.anim, flipbooks, 2D skeletons), its scaled tweens and its og.nav agent advance by scaledDt times og.time.setGroupScale's scale for the group, and its og.ParticleSystem's particles at the group's scale. Physics stays on the global scale: a body in a slowed group falls at the world's pace. An entity without one is at the global pace.

Token `og.TimeGroup`, registered under the schema name `og.TimeGroup`; its value type is `TimeGroup`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `number` | `0` | ≤ 15 | — | — | The group, 0 to 15. |

```ogt
component TimeGroup {
  doc "Which time group the entity is in (0 to 15): its animation (og.anim, flipbooks, 2D skeletons), its scaled tweens and its og.nav agent advance by scaledDt times og.time.setGroupScale's scale for the group, and its og.ParticleSystem's particles at the group's scale. Physics stays on the global scale: a body in a slowed group falls at the world's pace. An entity without one is at the global pace."
  field group u32 "The group, 0 to 15." {
    max 15
  }
}
```

<sub>`api.ogt` line 1645</sub>

<a id="record-TimeGroups"></a>

### `og.TimeGroups` (resource)

Each time group's scale on top of the global one, set with og.time.setGroupScale (every group starts at 1). Registered when a game first names og.TimeGroup or og.TimeGroups. Physics stays on the global scale.

Token `og.TimeGroups`, registered under the schema name `og.TimeGroups`; its value type is `TimeGroups`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scales` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per group, its scale (0 to 10); past the list's length 1. |

```ogt
resource TimeGroups {
  doc "Each time group's scale on top of the global one, set with og.time.setGroupScale (every group starts at 1). Registered when a game first names og.TimeGroup or og.TimeGroups. Physics stays on the global scale."
  field scales List.f32.16 "Per group, its scale (0 to 10); past the list's length 1."
}
```

<sub>`api.ogt` line 1652</sub>
