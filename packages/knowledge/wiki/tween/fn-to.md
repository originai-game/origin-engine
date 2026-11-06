<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-to"></a>

### `og.tween.to`

Start a tween. Systems may call it; it first moves in this tick's og.tween run.

Also known as: animate a value, DOTween, Godot Tween, LeanTween, GSAP, lerp over time, ease

```ts
og.tween.to(world: World, desc?: TweenDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TweenDesc` | yes | The field, the values and the timing. |

**Returns** `Result<Tween>`: Refused for a dead entity, a component it lacks, a field that is not numeric, or values of the wrong length.

Binding symbol `og_tween_to` · publish builds: kept

```ogt
fn to {
  doc "Start a tween. Systems may call it; it first moves in this tick's og.tween run."
  aliases "animate a value, DOTween, Godot Tween, LeanTween, GSAP, lerp over time, ease"
  param world World "The world."
  param desc TweenDesc "The field, the values and the timing."
  returns Result.Tween "Refused for a dead entity, a component it lacks, a field that is not numeric, or values of the wrong length."
}
```

<sub>`api.ogt` line 12492</sub>
