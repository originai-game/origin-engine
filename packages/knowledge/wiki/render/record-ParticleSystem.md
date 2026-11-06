<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-ParticleSystem"></a>

### `og.ParticleSystem` (component)

Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay; on an entity with og.TimeGroup they spawn, age and move at the group's scale (og.time.setGroupScale), read as the frame is drawn. They never feed back into the simulation. They simulate in compute with nothing read back, sprite-mode emitters included (emitters with sub-emitters on the CPU), identical either way. An emitter's `collision` line makes its particles bounce off the depth of what is drawn (`depth`, the default in 3D) or off the scene's og.Collider boxes, spheres, capsules and terrains (`colliders`, on or off screen; always in sprite mode), in compute either way. An emitter whose file has an `events` line (`events death collision`) is simulated in the tick instead, once the game names og.ParticleEvent: the og.particles system steps it with the engine's randomness and fixed math, sends an og.ParticleEvent for each of those events, and the renderer draws that simulation as it stands, so gameplay reads exactly the particles shown (snapshots and restores carry them).

Token `og.ParticleSystem`, registered under the schema name `og.ParticleSystem`; its value type is `ParticleSystem`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `emitter` | `ParticleEmitter` | `0` | — | — | — | The emitter asset; 0 draws nothing. |
| `seed` | `number` | `0` | — | — | — | The random seed: systems with different seeds differ. |
| `startTick` | `number` | `4294967295` | — | — | — | The tick emission started: write the tick (`time.tick`, `world.tick`) to fire a one-shot burst, or to restart one. Its default, PARTICLES_ARMED, is never written: a looping emitter emits as if started at tick 0, while a one-shot (`looping false`) stays armed and emits nothing until a tick is written, so effects spawned ahead of their first use wait. |
| `stopTick` | `number` | `0` | — | — | — | The tick emission stops (the particles live out); 0 keeps emitting while the emitter loops. |
| `visible` | `boolean` | `true` | — | — | — | Whether it is drawn (a hidden system loses its particles). |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Mesh.layers: its particles (billboards and 2D sprite particles alike) are drawn by the cameras whose masks share a bit with it (og.Camera, og.CameraView, og.Mirror). |

```ogt
component ParticleSystem {
  doc "Emits particles from an og.load.emitter asset at the entity's MatrixWorld. Particles are simulated by the renderer from (seed, startTick, the tick), so they are deterministic and replay; on an entity with og.TimeGroup they spawn, age and move at the group's scale (og.time.setGroupScale), read as the frame is drawn. They never feed back into the simulation. They simulate in compute with nothing read back, sprite-mode emitters included (emitters with sub-emitters on the CPU), identical either way. An emitter's `collision` line makes its particles bounce off the depth of what is drawn (`depth`, the default in 3D) or off the scene's og.Collider boxes, spheres, capsules and terrains (`colliders`, on or off screen; always in sprite mode), in compute either way. An emitter whose file has an `events` line (`events death collision`) is simulated in the tick instead, once the game names og.ParticleEvent: the og.particles system steps it with the engine's randomness and fixed math, sends an og.ParticleEvent for each of those events, and the renderer draws that simulation as it stands, so gameplay reads exactly the particles shown (snapshots and restores carry them)."
  field emitter ParticleEmitter "The emitter asset; 0 draws nothing."
  field seed u32 "The random seed: systems with different seeds differ."
  field startTick u32 "The tick emission started: write the tick (`time.tick`, `world.tick`) to fire a one-shot burst, or to restart one. Its default, PARTICLES_ARMED, is never written: a looping emitter emits as if started at tick 0, while a one-shot (`looping false`) stays armed and emits nothing until a tick is written, so effects spawned ahead of their first use wait." {
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

<sub>Module `render` line 1982</sub>
