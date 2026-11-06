<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`time`](../time.md) · [index](../index.md)

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

<sub>`api.ogt` line 1653</sub>
