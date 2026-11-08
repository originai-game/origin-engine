<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-setBackend"></a>

### `og.motion.setBackend`

Where the motion model runs: a presentation setting of this engine, outside the world (a snapshot does not keep it). gpu, the default, runs it beside the renderer, cheapest per character. No tick waits for it: a result is applied when it comes back, as of the tick it was due, and until then a frame draws the poses it has, so live frames may vary run to run (what checks a frame waits for it, as the module's overview says). cpu runs it on the CPU across the lanes instead, each generation spread over the ticks before it is due (about 21 ms of CPU a generation natively, 23 ms in wasm), for a few characters. Set it before the first agent; a change later loads the model again on the other backend and every agent's generated motion starts again.

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
  doc "Where the motion model runs: a presentation setting of this engine, outside the world (a snapshot does not keep it). gpu, the default, runs it beside the renderer, cheapest per character. No tick waits for it: a result is applied when it comes back, as of the tick it was due, and until then a frame draws the poses it has, so live frames may vary run to run (what checks a frame waits for it, as the module's overview says). cpu runs it on the CPU across the lanes instead, each generation spread over the ticks before it is due (about 21 ms of CPU a generation natively, 23 ms in wasm), for a few characters. Set it before the first agent; a change later loads the model again on the other backend and every agent's generated motion starts again."
  param world World "The world."
  param backend MotionBackend "cpu or gpu."
  returns Result.Void "Refused for none."
}
```

<sub>Module `motion` line 330</sub>
