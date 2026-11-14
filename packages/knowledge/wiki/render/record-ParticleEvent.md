<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-ParticleEvent"></a>

### `og.ParticleEvent` (event)

A particle of an og.ParticleSystem was born, died or hit something in the last tick (read in the tick after, like every event): a bullet's spark that lands, a firework's shell that bursts, debris that rings a bell. Sent only for systems whose emitter names the kind on its `events` line; such systems are simulated in the tick (og.particles), with the engine's randomness, so the events replay. Registered when a game first names it: a world that never does keeps its hash. Sub-emitters' particles send none.

Token `og.ParticleEvent`, registered under the schema name `og.ParticleEvent`; its value type is `ParticleEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.ParticleSystem. |
| `kind` | `ParticleEventKind` | `"birth"` | — | — | — | What happened. |
| `index` | `number` | `0` | — | — | — | The particle's index in its system's spawn order (counting from 0 at startTick). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where, in the world. |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | — | Its velocity then, in the world (before a bounce). |

```ogt
event ParticleEvent {
  doc "A particle of an og.ParticleSystem was born, died or hit something in the last tick (read in the tick after, like every event): a bullet's spark that lands, a firework's shell that bursts, debris that rings a bell. Sent only for systems whose emitter names the kind on its `events` line; such systems are simulated in the tick (og.particles), with the engine's randomness, so the events replay. Registered when a game first names it: a world that never does keeps its hash. Sub-emitters' particles send none."
  field entity Entity "The entity with the og.ParticleSystem."
  field kind ParticleEventKind "What happened."
  field index u32 "The particle's index in its system's spawn order (counting from 0 at startTick)."
  field position Vector3 "Where, in the world." {
    unit m
    worldspace world
  }
  field velocity Vector3 "Its velocity then, in the world (before a bounce)." {
    unit mps
  }
}
```

<sub>Module `render` line 2220</sub>
