<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`time`](../time.md) · [index](../index.md)

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

<sub>`api.ogt` line 1667</sub>
