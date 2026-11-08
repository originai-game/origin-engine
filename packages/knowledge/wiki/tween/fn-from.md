<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-from"></a>

### `og.tween.from`

Animate from desc.from to the field's value captured at this call, before the delay. The first og.tween tick writes from and holds it through the delay. Unlike GSAP immediateRender, the call itself never writes the field. A system must declare read access to the target component for capture. Snapshot/restore retains the captured endpoint; later writes during the delay cannot change it.

```ts
og.tween.from(world: World, desc?: TweenFromDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TweenFromDesc` | yes | The start value and timing. |

**Returns** `Result<Tween>`: Refused for an unreadable target or invalid values.

Binding symbol `og_tween_from` · publish builds: kept

```ogt
fn from {
  doc "Animate from desc.from to the field's value captured at this call, before the delay. The first og.tween tick writes from and holds it through the delay. Unlike GSAP immediateRender, the call itself never writes the field. A system must declare read access to the target component for capture. Snapshot/restore retains the captured endpoint; later writes during the delay cannot change it."
  param world World "The world."
  param desc TweenFromDesc "The start value and timing."
  returns Result.Tween "Refused for an unreadable target or invalid values."
}
```

<sub>Module `tween` line 410</sub>
