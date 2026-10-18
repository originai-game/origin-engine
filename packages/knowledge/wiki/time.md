<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `time` module

Simulation time. Game code never reads the wall clock; it reads Time.

- **TypeScript**: functions are the namespace `og.time`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module time {
  doc "Simulation time. Game code never reads the wall clock; it reads Time."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 541</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Time`](#record-Time) | resource | The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers. |
| [`og.time.setScale`](#fn-setScale) | function | Set the global time scale (slow motion): physics, og.anim, flipbooks and tweens advance by dt * scale from the system after the call; systems may call it. |
| [`og.time.hitStop`](#fn-hitStop) | function | Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. A longer hit stop in progress is never shortened. Typical values: 2-3 ticks for a light hit, 5-8 heavy, 10-15 a finishing blow. |

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

<sub>`api.ogt` line 582</sub>

<a id="fn-hitStop"></a>

### `og.time.hitStop`

Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. A longer hit stop in progress is never shortened. Typical values: 2-3 ticks for a light hit, 5-8 heavy, 10-15 a finishing blow.

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
  doc "Freeze scaled time for `ticks` ticks from now (this tick included): physics, og.anim, flipbooks and scaled tweens hold while input, the UI, unscaled tweens and camera shake keep moving. A longer hit stop in progress is never shortened. Typical values: 2-3 ticks for a light hit, 5-8 heavy, 10-15 a finishing blow."
  param world World "The world."
  param ticks u32 "Ticks to hold."
  returns Result.Void
}
```

<sub>`api.ogt` line 589</sub>

## Components, resources and events

<a id="record-Time"></a>

### `og.Time` (resource)

The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers.

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
  doc "The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers."
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

<sub>`api.ogt` line 544</sub>
