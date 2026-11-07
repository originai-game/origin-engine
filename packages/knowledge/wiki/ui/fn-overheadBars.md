<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-overheadBars"></a>

### `og.ui.overheadBars`

Overhead bars to the bar of RTS and MOBA crowds (StarCraft II, League of Legends): one mass element per unit, thousands a frame, each a dark track with its fill in its reaction's colour and a pale ghost that holds after a hit then drains, scaled by distance and faded behind the scene.

Parts: none per bar (mass elements); the template is named <id>. Accessibility: none (the units' frames carry it). Probe: og.ui.massCount.

```ts
og.ui.overheadBars(id: string, desc?: OverheadBarsDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id (its mass template's name). |
| `desc` | `OverheadBarsDesc` | yes | The bars. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn overheadBars {
  doc "Overhead bars to the bar of RTS and MOBA crowds (StarCraft II, League of Legends): one mass element per unit, thousands a frame, each a dark track with its fill in its reaction's colour and a pale ghost that holds after a hit then drains, scaled by distance and faded behind the scene."
  doc "Parts: none per bar (mass elements); the template is named <id>. Accessibility: none (the units' frames carry it). Probe: og.ui.massCount."
  host false
  param id String "Its id (its mass template's name)."
  param desc OverheadBarsDesc "The bars."
  returns Void
}
```

<sub>Module `ui` line 6496</sub>
