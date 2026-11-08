<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-setBackend"></a>

### `og.motion.setBackend`

Where the motion model runs: a presentation setting of this engine, outside the world (a snapshot does not keep it). cpu, the default, poses every agent with the same bits on every host, which replays, forks, captures and cross-host checks rely on; its lanes split the agents' generations. gpu is an explicit opt-in for big presentation-only crowds: cheaper per character at scale, but its poses are identical only on one host and GPU (each backend's shader compiler rounds its own way), and on the web a tick whose batch has not come back waits for it. Set it before the first agent; a change later loads the model again on the other backend and every agent's generated motion starts again.

```ts
og.motion.setBackend(world: World, backend: MotionBackend): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `backend` | `MotionBackend` | no | cpu or gpu. |

**Returns** `Result<void>`: Refused for none.

Binding symbol `og_motion_set_backend` · publish builds: kept

```ogt
fn setBackend {
  doc "Where the motion model runs: a presentation setting of this engine, outside the world (a snapshot does not keep it). cpu, the default, poses every agent with the same bits on every host, which replays, forks, captures and cross-host checks rely on; its lanes split the agents' generations. gpu is an explicit opt-in for big presentation-only crowds: cheaper per character at scale, but its poses are identical only on one host and GPU (each backend's shader compiler rounds its own way), and on the web a tick whose batch has not come back waits for it. Set it before the first agent; a change later loads the model again on the other backend and every agent's generated motion starts again."
  param world World "The world."
  param backend MotionBackend "cpu or gpu."
  returns Result.Void "Refused for none."
}
```

<sub>Module `motion` line 330</sub>
