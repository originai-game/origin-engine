<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`time`](../time.md) · [index](../index.md)

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

<sub>`api.ogt` line 1603</sub>
