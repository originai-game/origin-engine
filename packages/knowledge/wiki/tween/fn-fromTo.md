<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-fromTo"></a>

### `og.tween.fromTo`

Animate from explicit desc.from to desc.to; both are required. The first og.tween tick writes from, including during the delay. No endpoint is captured from the live field.

```ts
og.tween.fromTo(world: World, desc?: TweenDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TweenDesc` | yes | The explicit start, end and timing. |

**Returns** `Result<Tween>`: Refused when from is missing or the target or values are invalid.

Binding symbol `og_tween_from_to` · publish builds: kept

```ogt
fn fromTo {
  doc "Animate from explicit desc.from to desc.to; both are required. The first og.tween tick writes from, including during the delay. No endpoint is captured from the live field."
  param world World "The world."
  param desc TweenDesc "The explicit start, end and timing."
  returns Result.Tween "Refused when from is missing or the target or values are invalid."
}
```

<sub>Module `tween` line 417</sub>
