<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-radialMeter"></a>

### `og.ui.radialMeter`

A radial stamina wheel (Breath of the Wild, Elden Ring's spirit gauge): a ring that drains clockwise, outer rings for extra wheels, warn under a third, red and pulsing when exhausted until it refills, fading out once full.

Parts: RadialMeter.track, RadialMeter.fill[normal|low|exhausted], RadialMeter.wheel#i. States: group level (full, normal, low, exhausted). Accessibility: meter. Probe ids: <id>, <id>/Wheel#i.

```ts
og.ui.radialMeter(id: string, desc?: RadialMeterDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `RadialMeterDesc` | yes | The value and the look. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn radialMeter {
  doc "A radial stamina wheel (Breath of the Wild, Elden Ring's spirit gauge): a ring that drains clockwise, outer rings for extra wheels, warn under a third, red and pulsing when exhausted until it refills, fading out once full."
  doc "Parts: RadialMeter.track, RadialMeter.fill[normal|low|exhausted], RadialMeter.wheel#i. States: group level (full, normal, low, exhausted). Accessibility: meter. Probe ids: <id>, <id>/Wheel#i."
  host false
  param id String "Its id."
  param desc RadialMeterDesc "The value and the look."
  returns Void
}
```

<sub>Module `ui` line 5929</sub>
