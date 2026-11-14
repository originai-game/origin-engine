<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-ParticleSystem"></a>

### `og.ParticleSystem` (component)

Emits particles from an og.load.emitter asset at the entity's MatrixWorld. The renderer simulates them from (seed, startTick, the tick), so they are deterministic and replay, and they never feed back into the simulation; with og.TimeGroup they run at the group's scale, read as the frame is drawn. An emitter's `collision` line bounces them off the drawn depth or the scene's colliders. An emitter with an `events` line is simulated in the tick once the game names og.ParticleEvent, so gameplay reads the particles shown, and snapshots carry them.

See `looks §14`.

Token `og.ParticleSystem`, registered under the schema name `og.ParticleSystem`; its value type is `ParticleSystem`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `emitter` | `ParticleEmitter` | `0` | — | — | — | The emitter asset; 0 draws nothing. |
| `seed` | `number` | `0` | — | — | — | The random seed: systems with different seeds differ. |
| `startTick` | `number` | `4294967295` | — | — | — | The tick emission started: write the tick (`world.tick`) to fire a one-shot burst, or to restart one. Its default, PARTICLES_ARMED, is never written: a looping emitter emits as if started at tick 0, a one-shot (`looping false`) emits nothing until a tick is written. |
| `stopTick` | `number` | `0` | — | — | — | The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn (a hidden system loses its particles). |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Mesh.layers: its particles (billboards and 2D sprite particles alike) are drawn by the cameras whose masks share a bit with it (og.Camera, og.CameraView, og.Mirror). |

```ogt
component ParticleSystem {
  doc "Emits particles from an og.load.emitter asset at the entity's MatrixWorld. The renderer simulates them from (seed, startTick, the tick), so they are deterministic and replay, and they never feed back into the simulation; with og.TimeGroup they run at the group's scale, read as the frame is drawn. An emitter's `collision` line bounces them off the drawn depth or the scene's colliders. An emitter with an `events` line is simulated in the tick once the game names og.ParticleEvent, so gameplay reads the particles shown, and snapshots carry them."
  guide "looks §14"
  field emitter ParticleEmitter "The emitter asset; 0 draws nothing."
  field seed u32 "The random seed: systems with different seeds differ."
  field startTick u32 "The tick emission started: write the tick (`world.tick`) to fire a one-shot burst, or to restart one. Its default, PARTICLES_ARMED, is never written: a looping emitter emits as if started at tick 0, a one-shot (`looping false`) emits nothing until a tick is written." {
    default 4294967295
  }
  field stopTick u32 "The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops."
  field visible bool "Whether it is drawn (a hidden system loses its particles)." {
    default true
  }
  field layers u32 "Three.js layers, as Mesh.layers: its particles (billboards and 2D sprite particles alike) are drawn by the cameras whose masks share a bit with it (og.Camera, og.CameraView, og.Mirror)." {
    default 1
  }
}
```

<sub>Module `render` line 2178</sub>
