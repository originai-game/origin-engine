<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-sequence"></a>

### `og.tween.sequence`

Start tweens one after another (each with its own delay, duration and repeat); the time one leaves over goes to the next.

```ts
og.tween.sequence(world: World, steps: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `steps` | `readonly TweenDesc[]` | no | The tweens, in order. |
| `desc` | `TweenGroupDesc` | yes | Repeat, scaling and the tag. |

**Returns** `Result<Tween>`.

Binding symbol `og_tween_sequence` · publish builds: kept

```ogt
fn sequence {
  doc "Start tweens one after another (each with its own delay, duration and repeat); the time one leaves over goes to the next."
  param world World "The world."
  param steps List.TweenDesc "The tweens, in order."
  param desc TweenGroupDesc "Repeat, scaling and the tag." {
    optional true
  }
  returns Result.Tween
}
```

<sub>Module `tween` line 181</sub>
