<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-parallel"></a>

### `og.tween.parallel`

Start tweens together; the group ends with the last one.

```ts
og.tween.parallel(world: World, tweens: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tweens` | `readonly TweenDesc[]` | no | The tweens. |
| `desc` | `TweenGroupDesc` | yes | Loops, scaling and the tag. |

**Returns** `Result<Tween>`.

Binding symbol `og_tween_parallel` · publish builds: kept

```ogt
fn parallel {
  doc "Start tweens together; the group ends with the last one."
  param world World "The world."
  param tweens List.TweenDesc "The tweens."
  param desc TweenGroupDesc "Loops, scaling and the tag." {
    optional true
  }
  returns Result.Tween
}
```

<sub>Module `tween` line 150</sub>
