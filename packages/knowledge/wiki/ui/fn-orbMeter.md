<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-orbMeter"></a>

### `og.ui.orbMeter`

An orb (Diablo's health and mana globes): a glass sphere filled from the bottom with a liquid whose surface sways, a highlight on the glass, a frame ring, the value in the middle; the level moves on the ui spring and a drop flashes it. Critical (under 15 %) pulses.

Parts: OrbMeter.frame, OrbMeter.glass, OrbMeter.liquid[<kind>], OrbMeter.surface, OrbMeter.value, OrbMeter.label. States: group level (normal, low, critical). Accessibility: meter. Probe ids: <id>, <id>/Orb/Liquid, <id>/Value.

```ts
og.ui.orbMeter(id: string, desc?: OrbMeterDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `OrbMeterDesc` | yes | The value and the look. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn orbMeter {
  doc "An orb (Diablo's health and mana globes): a glass sphere filled from the bottom with a liquid whose surface sways, a highlight on the glass, a frame ring, the value in the middle; the level moves on the ui spring and a drop flashes it. Critical (under 15 %) pulses."
  doc "Parts: OrbMeter.frame, OrbMeter.glass, OrbMeter.liquid[<kind>], OrbMeter.surface, OrbMeter.value, OrbMeter.label. States: group level (normal, low, critical). Accessibility: meter. Probe ids: <id>, <id>/Orb/Liquid, <id>/Value."
  host false
  param id String "Its id."
  param desc OrbMeterDesc "The value and the look."
  returns Void
}
```

<sub>Module `ui` line 5975</sub>
