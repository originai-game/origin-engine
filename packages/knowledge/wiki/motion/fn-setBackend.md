<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-setBackend"></a>

### `og.motion.setBackend`

Where the motion model runs: gpu (the default, beside the renderer, cheapest per character) or cpu (across the lanes, each generation spread over the ticks before it is due, for a few characters). A presentation setting of this engine, outside the world: a snapshot does not keep it. No tick waits for it; results apply when they come back. A change after the first agent loads the model again and every agent's generated motion starts again.

See `animation §10`.

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
  doc "Where the motion model runs: gpu (the default, beside the renderer, cheapest per character) or cpu (across the lanes, each generation spread over the ticks before it is due, for a few characters). A presentation setting of this engine, outside the world: a snapshot does not keep it. No tick waits for it; results apply when they come back. A change after the first agent loads the model again and every agent's generated motion starts again."
  guide "animation §10"
  param world World "The world."
  param backend MotionBackend "cpu or gpu."
  returns Result.Void "Refused for none."
}
```

<sub>Module `motion` line 330</sub>
